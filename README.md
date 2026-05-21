# vhdx

[![Crates.io](https://img.shields.io/crates/v/vhdx.svg)](https://crates.io/crates/vhdx)
[![Docs.rs](https://docs.rs/vhdx/badge.svg)](https://docs.rs/vhdx)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/SecurityRonin/vhdx/actions/workflows/ci.yml/badge.svg)](https://github.com/SecurityRonin/vhdx/actions/workflows/ci.yml)

Pure-Rust read-only VHDX container reader. Decodes the MS-VHDX outer container format (Windows 8+ / Hyper-V) and exposes a `Read + Seek` interface over the virtual sector stream — ready to hand to any filesystem analysis crate.

## Quick start

```rust
use vhdx::VhdxReader;
use std::io::{Read, Seek, SeekFrom};

let mut reader = VhdxReader::open("disk.vhdx")?;
println!("Virtual disk size: {} bytes", reader.virtual_disk_size());
println!("Sector size:       {} bytes", reader.logical_sector_size());

// Read sector 0
let mut buf = [0u8; 512];
reader.seek(SeekFrom::Start(0))?;
reader.read_exact(&mut buf)?;
```

## Supported formats

- VHDX Version 1 (Windows 8 / Server 2012 and later)
- Dynamic disks (sparse, BAT-addressed data blocks)
- Fixed disks

**Not supported:** differencing disks (require parent chain resolution).

## CLI

The `vhdx-cli` crate provides a `vhdx info <path>` command:

```
$ vhdx info disk.vhdx
File:              disk.vhdx
Format:            VHDX v1 (dynamic)
Virtual disk size: 16,777,216 bytes (16.00 MiB)
Logical sectors:   512 bytes
```

## Related crates

- [`vhdx-forensic`](https://github.com/SecurityRonin/vhdx-forensic) — forensic integrity analyser and in-memory repair tool built on top of this crate
- [`ewf`](https://github.com/SecurityRonin/ewf) — equivalent reader for E01/EWF forensic disk images
- [`ewf-forensic`](https://github.com/SecurityRonin/ewf-forensic) — forensic analyser for E01 images

## License

MIT
