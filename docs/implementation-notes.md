# VHDX Implementation Notes

Developer notes capturing format quirks, spec contradictions, and empirically verified
behaviour. Intended for future contributors and as a basis for upstream spec clarifications.

---

## 1. Dual-header resiliency (sequence number arbitration)

VHDX stores **two** header copies at fixed offsets:
- Header 1: byte offset `0x0001_0000` (64 KiB)
- Header 2: byte offset `0x0002_0000` (128 KiB)

Each is a 4 KiB block with a CRC32C at bytes 4–7 (computed with those bytes zeroed).

**Selection rule:** the valid header (passing CRC32C) with the **higher sequence number**
is the active one. If only one is valid, use it regardless of sequence number.

```rust
match (h1, h2) {
    (Ok(a), Ok(b)) => if a.sequence_number >= b.sequence_number { a } else { b },
    (Ok(a), Err(_)) => a,
    (Err(_), Ok(b)) => b,
    (Err(_), Err(_)) => Err(VhdxError::NoValidHeader),
}
```

**Common pitfall:** reading only the first header without checking the second. After
a crash mid-write, the first header may be newer but invalid; the second is the last
committed state.

---

## 2. Log replay before any structure parsing

**Critical:** if the active header's `LogGuid` is non-zero and `LogLength > 0`,
the image is **dirty** — an in-progress write was interrupted. The transaction log
must be replayed before parsing regions, metadata, or the BAT.

Without replay, you may read stale BAT entries or corrupted metadata that point to
the wrong file offsets.

### Log entry structure

Each log entry (`loge` cookie) contains:
- **Data descriptors** (`desc`): copy 4096 bytes from an embedded sector to `FileOffset`
- **Zero descriptors** (`zero`): fill `ZeroLength` bytes at `FileOffset` with zeros

Entries must be sorted by sequence number and applied in ascending order. Entries whose
`LogGuid` differs from the active header's `LogGuid` are skipped (they belong to a
prior, already-committed transaction).

### Clean image shortcut

If `log_guid == [0u8; 16]` or `log_length == 0`, skip log processing entirely.

---

## 3. BAT interleaving with sector bitmap entries

The Block Allocation Table is **not** a flat array of data-block entries. Every
`chunk_ratio` data-block entries is followed by one **sector-bitmap entry**:

```
BAT index 0:             data block 0
BAT index 1:             data block 1
...
BAT index chunk_ratio-1: data block (chunk_ratio - 1)
BAT index chunk_ratio:   sector bitmap block 0   ← interleaved
BAT index chunk_ratio+1: data block chunk_ratio
...
```

The formula to convert a virtual data block index to its BAT index:

```rust
let bat_index = data_block_index + data_block_index / chunk_ratio;
```

The `chunk_ratio` is derived from metadata:

```
chunk_ratio = (2^23 * logical_sector_size) / block_size
```

For default geometry (512-byte sectors, 32 MiB block size):
`chunk_ratio = (2^23 * 512) / (32 * 1024 * 1024) = 128`

**Common pitfall:** treating BAT as a flat array and indexing by `data_block_index`
directly — reads the wrong entry for any block beyond the first sector-bitmap slot.

---

## 4. File offset encoding (20-bit shift)

The BAT entry stores the file offset of a payload block in **1-MiB units** (bits 63..20).
The lower 20 bits are the state field:

```
bits 63..20: file_offset_mb  (shift right 20 to get MiB count)
bits 2..0:   block state
```

To recover the byte offset:

```rust
let file_offset_mb = bat_entry >> 20;
let file_offset = file_offset_mb * 0x0010_0000 + offset_within_block;
```

This means all payload blocks are aligned to 1 MiB boundaries. The spec calls this
"payload block offset" and uses the term "Payload Block Offset in units of 1 MB" — but
the unit is **1 MiB (2^20 bytes)**, not 1 MB (10^6 bytes).

---

## 5. Block states

The 3-bit state field in BAT entries:

| Value | Name | Meaning |
|-------|------|---------|
| 0 | `PAYLOAD_BLOCK_NOT_PRESENT` | Sparse / unwritten — read as zeros |
| 2 | `PAYLOAD_BLOCK_UNDEFINED` | Allocation attempted but incomplete |
| 3 | `PAYLOAD_BLOCK_ZERO` | Present but all zeros (optimisation) |
| 5 | `PAYLOAD_BLOCK_UNMAPPED` | Block was trimmed/unmapped |
| 6 | `PAYLOAD_BLOCK_FULLY_PRESENT` | Fully written — read from file offset |
| 7 | `PAYLOAD_BLOCK_PARTIALLY_PRESENT` | Some sectors present (used with sector bitmaps) |

Our implementation only handles 0 (return zeros) and 6 (read from file). States 3 and 5
should also return zeros. State 7 requires per-sector bitmap consultation.

---

## 6. CRC32C everywhere

VHDX uses CRC-32C (Castagnoli polynomial `0x82F63B78`) for all integrity checks:
- File identifier block (first 512 bytes)
- Both header copies
- Both region table copies
- Every log entry

The stored CRC occupies bytes 4–7 of each structure; compute with those bytes zeroed,
then compare.

Do **not** confuse CRC32C (`0x82F63B78`) with the more common CRC32 used in zlib/PNG
(`0xEDB88320`). Using the wrong polynomial yields silently incorrect checksums.

---

## 7. Differencing / child disks

VHDX supports differencing disks where the parent is referenced by a GUID in the
virtual disk parameters metadata. This implementation rejects differencing disks.

Implementing correctly requires:
- Locating the parent via `ParentLocationBuffer` (relative/absolute paths, volume GUID)
- Applying the child's BAT on top of the parent's virtual address space
- Reading from parent for blocks not present in the child

---

## Upstream PR candidates

| Project | File | Suggested change |
|---------|------|-----------------|
| MS-VHDX spec | §2.5 (BAT) | Add worked example of `bat_index` formula with explicit `chunk_ratio` calculation for default geometry |
| MS-VHDX spec | §2.3.4 | Clarify "1 MB" in file offset description is 2^20 bytes (MiB), not 10^6 bytes |
| QEMU | `block/vhdx.c` | Add comment explaining why `log_replay` must precede BAT parsing |
