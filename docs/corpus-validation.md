# VHDX Corpus Validation

Byte-level differential tests comparing `VhdxReader` output against
`qemu-img convert -O raw` (QEMU 11.0.0, macOS/Apple Silicon).

## Test Environment

| Component | Version |
|-----------|---------|
| QEMU | 11.0.0 (Homebrew, `/opt/homebrew/bin/qemu-img`) |
| OS | macOS (Apple Silicon) |
| Rust | (see `rust-toolchain.toml`) |

## Corpus Files

Full provenance (SHA-256, source URLs, regeneration commands) is in
`tests/data/SOURCES.md`.

| File | Source | Virtual size | Notes |
|------|--------|--------------|-------|
| `ext2.vhdx` | log2timeline/dfvfs (Apache-2.0) | 4 MiB | QEMU v5.2, ext2 filesystem, dynamic |
| `fat-parent.vhdx` | log2timeline/dfvfs (Apache-2.0) | 4 MiB | FAT filesystem, standalone parent |
| `fat-differential.vhdx` | log2timeline/dfvfs (Apache-2.0) | 4 MiB | FAT differencing disk (DifferencingDisk warning) |
| `qemu_empty_dynamic.vhdx` | Generated locally (QEMU 11.0.0) | 16 MiB | Empty dynamic |
| `qemu_fixed.vhdx` | Generated locally (QEMU 11.0.0) | 8 MiB | Fixed provisioning |
| `ext2.vhd` | log2timeline/dfvfs (Apache-2.0) | 2 MiB | Legacy VHD — must be rejected |

## Test Results

### `corpus_ext2_vhdx_matches_qemu_raw`

Full byte scan of `ext2.vhdx` at 64 KiB stride + near-end read.
**PASS** — bytes match qemu-img.

Exercises: dynamic BAT, sector bitmap, 512-byte logical sectors, real ext2
filesystem layout.

### `corpus_qemu_fixed_vhdx_matches_qemu_raw`

Full byte scan of `qemu_fixed.vhdx` at 64 KiB stride + near-end read.
**PASS** — bytes match qemu-img.

Exercises: fixed-provisioning BAT (all entries in FULLY PRESENT state), all
sector bitmap blocks allocated — structurally different BAT from dynamic images.

### `corpus_fat_parent_vhdx_matches_qemu_raw`

Full byte scan of `fat-parent.vhdx` at 64 KiB stride + near-end read.
**PASS** — bytes match qemu-img.

Exercises: FAT filesystem layout, 2 MiB block size, third-party creator tool.

### `fat-differential.vhdx` (negative test)

`qemu-img` cannot open this image ("Operation not supported" — created by a
non-QEMU tool). Our reader opens it successfully but emits a `DifferencingDisk`
warning and reads data from its own blocks. No qemu-img differential test.

## Validation Coverage

| Feature | Covered | Notes |
|---------|---------|-------|
| Dynamic BAT (absent blocks → zeros) | Yes | ext2.vhdx, fat-parent.vhdx |
| Fixed BAT (all blocks present) | Yes | qemu_fixed.vhdx |
| Dual-header sequence arbitration | Yes | all images parsed |
| Log replay | Yes | images with non-empty log |
| BAT interleave formula | Yes | validated by byte-identical reads |
| 512-byte logical sectors | Yes | ext2.vhdx, qemu images |
| Differencing disk parsing | Yes | fat-differential.vhdx (no byte diff) |
| Large block size variations | Yes | 8 MiB and 2 MiB blocks tested |
| ext4 / NTFS filesystems | No | not in current corpus |
| Parent disk resolution | No | differencing disk parent lookup not implemented |

## Reproducing

```sh
# See tests/data/SOURCES.md for per-file download / generation commands

# Run validation tests
cargo test --test corpus_differential
```
