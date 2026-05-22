[![Crates.io](https://img.shields.io/crates/v/vhdx.svg)](https://crates.io/crates/vhdx)
[![Docs.rs](https://img.shields.io/docsrs/vhdx)](https://docs.rs/vhdx)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![CI](https://github.com/SecurityRonin/vhdx/actions/workflows/ci.yml/badge.svg)](https://github.com/SecurityRonin/vhdx/actions/workflows/ci.yml)
[![Sponsor](https://img.shields.io/badge/sponsor-h4x0r-ea4aaa?logo=github-sponsors)](https://github.com/sponsors/h4x0r)

**Pure-Rust read-only VHDX container reader.**

Decodes the Microsoft VHDX container format (Hyper-V, Windows 8+) and exposes a `Read + Seek` interface over the virtual sector stream — zero unsafe code, no C bindings, no external tools required.

```toml
[dependencies]
vhdx = "0.1"
```

---

## Usage

### Open a VHDX and read sectors

```rust
use vhdx::VhdxReader;
use std::io::{Read, Seek, SeekFrom};

let mut reader = VhdxReader::open("disk.vhdx")?;

println!("Virtual disk size: {} bytes", reader.virtual_disk_size());
println!("Logical sector size: {} bytes", reader.logical_sector_size());

// Read the first sector
let mut sector = vec![0u8; reader.logical_sector_size() as usize];
reader.seek(SeekFrom::Start(0))?;
reader.read_exact(&mut sector)?;
```

### Pass to a filesystem crate

`VhdxReader` implements `Read + Seek`, so it drops directly into any crate that accepts a reader:

```rust
use vhdx::VhdxReader;

let reader = VhdxReader::open("disk.vhdx")?;
// e.g. ext4fs_forensic::Filesystem::open(reader)?;
```

### Read from an in-memory buffer

```rust
use vhdx::VhdxReader;

let data: Vec<u8> = std::fs::read("disk.vhdx")?;
let reader = VhdxReader::from_bytes(data)?;
```

---

## CLI

The `vhdx-cli` crate (included in this workspace) provides a `vhdx info` command:

```
$ vhdx info disk.vhdx
File:              disk.vhdx
Format:            VHDX v1 (dynamic)
Virtual disk size: 16,777,216 bytes (16.00 MiB)
Logical sectors:   512 bytes
```

---

## Supported formats

| Format | Supported |
|--------|:---------:|
| VHDX Version 1 (Windows 8 / Server 2012+) | ✓ |
| Dynamic disks (sparse, BAT-addressed) | ✓ |
| Fixed disks (pre-allocated) | ✓ |
| Differencing disks (require parent chain) | — |
| Log replay | — |

Read-only. Offline forensic use — log replay is not performed (snapshots are typically clean; mount on a running Hyper-V host for automatic replay if needed).

---

## Related crates

| Crate | Role |
|-------|------|
| [`vhdx-forensic`](https://github.com/SecurityRonin/vhdx-forensic) | Forensic integrity analyser and in-memory repair tool built on this crate |
| [`ewf`](https://github.com/SecurityRonin/ewf) | Equivalent reader for E01/EWF forensic disk images |
| [`ewf-forensic`](https://github.com/SecurityRonin/ewf-forensic) | Forensic analyser for E01 images |

---

[Privacy Policy](https://securityronin.github.io/vhdx/privacy/) · [Terms of Service](https://securityronin.github.io/vhdx/terms/) · © 2026 Security Ronin Ltd
