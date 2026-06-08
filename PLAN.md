# Plan: `vhdx` crate — Pure-Rust VHDX Reader

**Implementer**: another Claude Code session  
**Goal**: Create `~/src/vhdx/` as a Cargo workspace containing `vhdx` (reader crate)
and `vhdx-cli` (CLI tool), mirroring the `~/src/ewf/` structure exactly.
Then update `~/src/vhdx-forensic/` to depend on the new `vhdx` crate instead of
carrying its own reader implementation.

**Do not start implementing before reading this plan in full.**

---

## Context

`~/src/vhdx-forensic/` is a single Rust crate containing both:
- A clean VHDX reader (`VhdxReader` — `Read + Seek` over virtual sectors)
- A forensic integrity analyser (`VhdxIntegrity`) and in-memory repair tool (`VhdxRepair`)

The ecosystem pattern (see `~/src/ewf/` and `~/src/ewf-forensic/`) separates the two
concerns into distinct crates:

```
ewf            (reader only, thin deps)        ~/src/ewf/ewf/
ewf-forensic   (integrity audit + repair)      ~/src/ewf-forensic/
```

This plan creates the same split for VHDX:

```
vhdx           (reader only, thin deps)        ~/src/vhdx/vhdx/      ← NEW
vhdx-cli       (CLI tool)                      ~/src/vhdx/vhdx-cli/  ← NEW
vhdx-forensic  (integrity audit + repair)      ~/src/vhdx-forensic/  ← REFACTORED
```

---

## TDD requirement — MANDATORY

**Every task requires two separate commits: RED (failing tests) then GREEN (passing).**

The reader code already exists in `vhdx-forensic`. The TDD approach here is:

1. Copy the test files into the new `vhdx` crate first.
2. Run `cargo test` — they fail (the code doesn't exist yet).
3. Copy/move the source modules.
4. Run `cargo test` — they pass.
5. Commit RED (tests only), then GREEN (code that passes).

For the `vhdx-cli`, write a test that asserts the `vhdx info` command produces expected
output, then implement the command.

The `vhdx-forensic` refactor is simpler: after updating import paths from `crate::` to
`vhdx::`, the existing test suite must still pass — no tests change, only the import
paths and Cargo.toml.

---

## Phase 1: Initialise `~/src/vhdx/` workspace

### 1.1 Git init

```bash
cd ~/src/vhdx
git init
git checkout -b main
```

### 1.2 Workspace `Cargo.toml`

Mirror `~/src/ewf/Cargo.toml` exactly, adapting names:

```toml
[workspace]
members = ["vhdx", "vhdx-cli"]
resolver = "2"

[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
pedantic = { level = "warn", priority = -1 }
correctness = { level = "deny", priority = -1 }
suspicious = { level = "deny", priority = -1 }
module_name_repetitions = { level = "allow", priority = 1 }
must_use_candidate = { level = "allow", priority = 1 }
missing_errors_doc = { level = "allow", priority = 1 }
missing_panics_doc = { level = "allow", priority = 1 }
cast_possible_truncation = { level = "allow", priority = 1 }
cast_possible_wrap = { level = "allow", priority = 1 }
cast_sign_loss = { level = "allow", priority = 1 }
cast_precision_loss = { level = "allow", priority = 1 }
format_push_string = { level = "allow", priority = 1 }
format_collect = { level = "allow", priority = 1 }
too_many_lines = { level = "allow", priority = 1 }
items_after_statements = { level = "allow", priority = 1 }
similar_names = { level = "allow", priority = 1 }
manual_let_else = { level = "allow", priority = 1 }
```

### 1.3 `deny.toml`

Copy `~/src/ewf/deny.toml` verbatim (cargo-deny config).

### 1.4 `rustfmt.toml`

Copy `~/src/ewf/rustfmt.toml` verbatim.

### 1.5 `.gitignore`

Standard Rust gitignore (`/target`).

---

## Phase 2: Create the `vhdx` reader crate

### 2.1 Directory structure

```
vhdx/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── error.rs
│   ├── header.rs
│   ├── region.rs
│   ├── metadata.rs
│   ├── bat.rs
│   └── reader.rs
└── tests/
    ├── data/           ← symlink or copy from ~/src/vhdx-forensic/tests/data/
    └── compat.rs       ← adapted from vhdx-forensic tests
```

### 2.2 `vhdx/Cargo.toml`

```toml
[package]
name = "vhdx"
version = "0.1.0"
edition = "2021"
rust-version = "1.85"
description = "Pure-Rust read-only VHDX container reader"
license = "MIT"
repository = "https://github.com/SecurityRonin/vhdx-core"
homepage = "https://github.com/SecurityRonin/vhdx-core"
documentation = "https://docs.rs/vhdx"
authors = ["Albert Hui"]
keywords = ["forensics", "vhdx", "disk-image", "windows", "hyper-v"]
categories = ["parser-implementations", "filesystem"]
readme = "../README.md"
exclude = ["tests/data/"]

[dependencies]
thiserror = "2"

[dev-dependencies]
tempfile = "3"

[lints]
workspace = true
```

### 2.3 Source files — copy and adapt from `~/src/vhdx-forensic/src/`

Copy these files verbatim (they contain no forensic-specific code):
- `src/error.rs`   → `vhdx/src/error.rs`      (no changes needed)
- `src/header.rs`  → `vhdx/src/header.rs`      (visibility changes — see §2.4)
- `src/region.rs`  → `vhdx/src/region.rs`      (visibility changes — see §2.4)
- `src/metadata.rs`→ `vhdx/src/metadata.rs`    (visibility changes — see §2.4)
- `src/bat.rs`     → `vhdx/src/bat.rs`         (visibility changes — see §2.4)
- `src/reader.rs`  → `vhdx/src/reader.rs`      (update `crate::` references)

### 2.4 Visibility changes required

`vhdx-forensic`'s `integrity.rs` and `repair.rs` currently import these items from
the local crate modules. After the split they will import them from the `vhdx` crate,
so these items must be `pub` (not `pub(crate)`) in the new `vhdx` crate.

**From `header.rs`** — make these `pub`:
- `fn crc32c(data: &[u8]) -> u32`
- `const HEADER1_OFFSET: u64`
- `const HEADER2_OFFSET: u64`
- `const HEADER_SIGNATURE: &[u8; 4]`
- `const HEADER_SIZE: usize`
- `const REGION_TABLE1_OFFSET: u64`
- `const REGION_TABLE2_OFFSET: u64`

**From `region.rs`** — make these `pub`:
- `const BAT_GUID: [u8; 16]`
- `const MB: u64`
- `const METADATA_GUID: [u8; 16]`
- `const REGION_ENTRY_SIZE: usize`
- `const REGION_TABLE_CRC_COVERAGE: usize`
- `const REGION_TABLE_SIGNATURE: &[u8; 4]`

**From `metadata.rs`** — make these `pub`:
- `const GUID_FILE_PARAMETERS: [u8; 16]`
- `const GUID_LOGICAL_SECTOR_SIZE: [u8; 16]`
- `const GUID_PARENT_LOCATOR: [u8; 16]`
- `const GUID_PHYSICAL_SECTOR_SIZE: [u8; 16]`
- `const GUID_VIRTUAL_DISK_ID: [u8; 16]`
- `const GUID_VIRTUAL_DISK_SIZE: [u8; 16]`
- `const METADATA_TABLE_SIGNATURE: &[u8; 8]`

Everything else in these modules can remain `pub(crate)`.

### 2.5 `vhdx/src/lib.rs`

```rust
//! Pure-Rust read-only VHDX container reader.
//!
//! Decodes the MS-VHDX outer container format and exposes a `Read + Seek`
//! interface over the virtual sector stream.
//!
//! # Supported formats
//! - VHDX Version 1 (Windows 8+ / Server 2012+)
//! - Dynamic disks
//! - Fixed disks
//!
//! # Layer
//! CONTAINER — equivalent role to `ewf` for E01 images.

mod bat;
mod error;
pub mod header;
pub mod metadata;
pub mod region;
mod reader;

pub use error::{Result, VhdxError};
pub use reader::VhdxReader;

/// Well-known VHDX file magic (first 8 bytes of every VHDX file).
pub const FILE_MAGIC: &[u8; 8] = b"vhdxfile";
```

Note: `header`, `metadata`, `region` are `pub mod` so `vhdx-forensic` can import their
constants and functions. `bat` and `reader` stay `pub(crate)`.

### 2.6 Test data

The test images live at `~/src/vhdx-forensic/tests/data/`. Do NOT copy them — symlink:

```bash
ln -s ../vhdx-forensic/tests/data ~/src/vhdx/vhdx/tests/data
```

If the symlink approach causes problems, copy the images instead, but do not commit
them to both repos — the forensic repo is the source of truth.

### 2.7 `vhdx/tests/compat.rs`

Write reader-focused tests adapted from `~/src/vhdx-forensic/tests/libvhdi_compat.rs`
and `~/src/vhdx-forensic/tests/real_images.rs`. Include:

- `ext2_vhdx_opens` — `VhdxReader::from_bytes(data("ext2.vhdx"))` succeeds
- `ext2_vhdx_virtual_disk_size` — 4 MiB
- `ext2_vhdx_sector_0_readable` — read_exact 512 bytes succeeds
- `fat_parent_vhdx_opens` — succeeds
- `fat_parent_vhdx_virtual_disk_size` — 4 MiB
- `fat_differential_vhdx_refused` — `from_bytes` returns `Err` (DifferencingNotSupported)
- `ext2_vhd_rejected` — `from_bytes` returns `Err` (not VHDX magic)
- `qemu_empty_dynamic_opens` — succeeds
- `qemu_empty_dynamic_virtual_disk_size` — 16 MiB
- `qemu_fixed_opens` — succeeds
- `qemu_fixed_virtual_disk_size` — 8 MiB

TDD: write these tests first → confirm they fail (crate doesn't exist) → copy source →
confirm they pass.

---

## Phase 3: Create the `vhdx-cli` crate

### 3.1 `vhdx-cli/Cargo.toml`

```toml
[package]
name = "vhdx-cli"
version = "0.1.0"
edition = "2021"
rust-version = "1.85"
description = "CLI tool for inspecting VHDX disk images"
license = "MIT"
repository = "https://github.com/SecurityRonin/vhdx-core"
keywords = ["forensics", "vhdx", "disk-image"]
categories = ["command-line-utilities"]

[[bin]]
name = "vhdx"
path = "src/main.rs"

[dependencies]
vhdx = { version = "0.1.0", path = "../vhdx" }
clap = { version = "4", features = ["derive"] }

[lints]
workspace = true
```

### 3.2 `vhdx-cli/src/main.rs` — `vhdx info` command

Minimum viable CLI: one subcommand `info <path>` that prints:

```
File:              disk.vhdx
Format:            VHDX v1 (dynamic)
Virtual disk size: 16,777,216 bytes (16.00 MiB)
Logical sectors:   512 bytes
```

Use `clap` with a `derive`-based API, matching `ewf-cli` style. No MCP server for v0.1.

```rust
#[derive(Parser)]
#[command(name = "vhdx", version, about = "CLI tool for VHDX disk images")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Display image metadata
    Info { path: PathBuf },
}
```

### 3.3 CLI integration test

Create `vhdx-cli/tests/cli.rs`:

```rust
#[test]
fn info_shows_virtual_disk_size() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_vhdx"))
        .args(["info", "tests/data/qemu_empty_dynamic.vhdx"])
        .output()
        .expect("vhdx binary must run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("16,777,216") || stdout.contains("16 MiB"),
        "expected virtual disk size in output, got: {stdout}");
}
```

TDD: write test → `cargo test --test cli` fails (binary not built) → implement main.rs
→ test passes.

---

## Phase 4: Refactor `~/src/vhdx-forensic/` to depend on `vhdx`

This phase transforms `vhdx-forensic` from a standalone crate into a forensic analysis
layer that uses `vhdx` for its parser infrastructure.

### 4.1 Add `vhdx` dependency to `vhdx-forensic/Cargo.toml`

During development, use a path dependency. Switch to crates.io version after `vhdx` is
published.

```toml
[dependencies]
thiserror = "2"
vhdx = { path = "../vhdx/vhdx" }   # temporary; change to version = "0.1" after publish
```

### 4.2 Update import paths in `integrity.rs`

Change these `use crate::` statements:
```rust
// BEFORE
use crate::header::{crc32c, HEADER1_OFFSET, HEADER2_OFFSET, HEADER_SIGNATURE,
    HEADER_SIZE, REGION_TABLE1_OFFSET, REGION_TABLE2_OFFSET};
use crate::metadata::{GUID_FILE_PARAMETERS, GUID_LOGICAL_SECTOR_SIZE,
    GUID_PARENT_LOCATOR, GUID_PHYSICAL_SECTOR_SIZE, GUID_VIRTUAL_DISK_ID,
    GUID_VIRTUAL_DISK_SIZE, METADATA_TABLE_SIGNATURE};
use crate::region::{BAT_GUID, MB, METADATA_GUID, REGION_ENTRY_SIZE,
    REGION_TABLE_CRC_COVERAGE, REGION_TABLE_SIGNATURE};
use crate::FILE_MAGIC;

// AFTER
use vhdx::header::{crc32c, HEADER1_OFFSET, HEADER2_OFFSET, HEADER_SIGNATURE,
    HEADER_SIZE, REGION_TABLE1_OFFSET, REGION_TABLE2_OFFSET};
use vhdx::metadata::{GUID_FILE_PARAMETERS, GUID_LOGICAL_SECTOR_SIZE,
    GUID_PARENT_LOCATOR, GUID_PHYSICAL_SECTOR_SIZE, GUID_VIRTUAL_DISK_ID,
    GUID_VIRTUAL_DISK_SIZE, METADATA_TABLE_SIGNATURE};
use vhdx::region::{BAT_GUID, MB, METADATA_GUID, REGION_ENTRY_SIZE,
    REGION_TABLE_CRC_COVERAGE, REGION_TABLE_SIGNATURE};
use vhdx::FILE_MAGIC;
```

### 4.3 Update import paths in `repair.rs`

```rust
// BEFORE
use crate::header::{crc32c, HEADER1_OFFSET, HEADER2_OFFSET, HEADER_SIZE,
    REGION_TABLE1_OFFSET, REGION_TABLE2_OFFSET};
use crate::region::REGION_TABLE_CRC_COVERAGE;

// AFTER
use vhdx::header::{crc32c, HEADER1_OFFSET, HEADER2_OFFSET, HEADER_SIZE,
    REGION_TABLE1_OFFSET, REGION_TABLE2_OFFSET};
use vhdx::region::REGION_TABLE_CRC_COVERAGE;
```

### 4.4 Update `vhdx-forensic/src/lib.rs`

```rust
//! VHDX forensic integrity analyser and in-memory repair tool.
//!
//! Depends on the `vhdx` crate for container parsing. Exports `VhdxIntegrity`,
//! `VhdxRepair`, and all anomaly/severity types.
//!
//! # Layer
//! FORENSIC AUDIT — equivalent role to `ewf-forensic` for E01 images.

pub mod integrity;
pub mod repair;

// Re-export the reader so callers don't need two crates for basic use.
pub use vhdx::{Result, VhdxError, VhdxReader, FILE_MAGIC};
pub use integrity::{AnalysisSummary, Severity, VhdxIntegrity, VhdxIntegrityAnomaly};

/// Return references to all anomalies whose severity is at or above `min`.
pub fn anomalies_at_least<'a>(
    anomalies: &'a [VhdxIntegrityAnomaly],
    min: Severity,
) -> Vec<&'a VhdxIntegrityAnomaly> {
    anomalies.iter().filter(|a| a.severity() >= min).collect()
}
pub use repair::{CannotRepair, RepairAction, RepairReport, VhdxRepair};
```

### 4.5 Delete removed modules from `vhdx-forensic/src/`

After the import paths are updated and `cargo test` passes:

```bash
cd ~/src/vhdx-forensic
git rm src/bat.rs src/error.rs src/header.rs src/metadata.rs src/region.rs src/reader.rs
```

These modules now live in `vhdx`. The forensic crate no longer maintains its own copy.

### 4.6 Verify tests still pass

```bash
cargo test    # all 138 tests must still pass
cargo clippy --all-targets -- -D warnings
```

No test changes should be needed — the public API (VhdxIntegrity, VhdxRepair, VhdxReader,
anomaly types, Severity, anomalies_at_least) is unchanged.

---

## Phase 5: CI and project config

### 5.1 `~/src/vhdx/.github/workflows/ci.yml`

Mirror `~/src/ewf/.github/workflows/ci.yml`:

```yaml
name: CI
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: -Dwarnings

jobs:
  fmt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683
      - uses: dtolnay/rust-toolchain@stable
        with: { components: rustfmt }
      - run: cargo fmt --check

  clippy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683
      - uses: dtolnay/rust-toolchain@stable
        with: { components: clippy }
      - uses: Swatinem/rust-cache@9d47c6ad4b02e050fd481d890b2ea34778fd09d6
      - run: cargo clippy --all-targets -- -D warnings

  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@9d47c6ad4b02e050fd481d890b2ea34778fd09d6
      - run: cargo test

  deny:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683
      - uses: EmbarkStudios/cargo-deny-action@3fd3802e88374d3fe9159b834c7714ec57d6c979

  msrv:
    name: MSRV (1.85)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683
      - uses: dtolnay/rust-toolchain@1.85
      - uses: Swatinem/rust-cache@9d47c6ad4b02e050fd481d890b2ea34778fd09d6
      - run: cargo test

  secrets:
    name: Secret Scan (gitleaks)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683
        with: { fetch-depth: 0 }
      - uses: gitleaks/gitleaks-action@v2
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

**Important**: the test images in `tests/data/` are part of the repo (see §2.6). CI
`actions/checkout` will include them. Do not add a step to download them.

### 5.2 No release workflow for v0.1

Skip the release/binary/winget workflow for now. Add it when the crate stabilises.

---

## Phase 6: README

Write `~/src/vhdx/README.md` with:
- Badges (crates.io, docs.rs, MIT, CI)
- One-paragraph description: pure-Rust VHDX reader, `Read + Seek` over virtual sectors
- Quick-start usage block (same as the `## Usage → VhdxReader` section in vhdx-forensic README)
- "For forensic integrity analysis, see `vhdx-forensic`" pointer
- Related crates section
- License

---

## Commit sequence

```
# Phase 2 TDD
git commit -m "test(RED): vhdx reader compat tests — all fail, crate empty"
git commit -m "feat(GREEN): vhdx reader crate — copied from vhdx-forensic, tests pass"

# Phase 3 TDD
git commit -m "test(RED): vhdx-cli info command test — fails, binary missing"
git commit -m "feat(GREEN): vhdx-cli info command — prints virtual disk size and format"

# Phase 5
git commit -m "ci: add CI workflow (fmt, clippy, test, deny, msrv, gitleaks)"

# Phase 6
git commit -m "docs: README with usage and related-crates section"
```

Separate repo (`~/src/vhdx-forensic/`):
```
git commit -m "test(RED): import vhdx dep — integrity/repair imports broken"
git commit -m "refactor(GREEN): vhdx-forensic depends on vhdx crate, removes reader modules"
```

---

## Checklist before declaring done

- [ ] `cargo test` passes in `~/src/vhdx/` (all compat tests green)
- [ ] `cargo test` passes in `~/src/vhdx-forensic/` (all 138 tests still pass)
- [ ] `cargo clippy --all-targets -- -D warnings` clean in both repos
- [ ] `cargo fmt --check` clean in both repos
- [ ] `vhdx-forensic/src/{bat,error,header,metadata,region,reader}.rs` are deleted
- [ ] `vhdx-forensic` Cargo.toml has `vhdx = { path = "..." }` dependency
- [ ] `vhdx/src/lib.rs` re-exports: `VhdxReader`, `VhdxError`, `Result`, `FILE_MAGIC`
- [ ] `vhdx/src/{header,metadata,region}.rs` are `pub mod` with all listed items `pub`
- [ ] `vhdx-cli` `vhdx info` command works on all 6 test images
- [ ] CI workflow file present and syntactically valid

---

## Key files to read before starting

```
~/src/ewf/Cargo.toml                     ← workspace structure reference
~/src/ewf/ewf/Cargo.toml                 ← reader crate Cargo.toml reference
~/src/ewf/ewf-cli/Cargo.toml            ← CLI crate Cargo.toml reference
~/src/ewf/.github/workflows/ci.yml      ← CI workflow reference
~/src/vhdx-forensic/src/lib.rs          ← current public API
~/src/vhdx-forensic/src/reader.rs       ← VhdxReader implementation
~/src/vhdx-forensic/src/header.rs       ← header constants and crc32c
~/src/vhdx-forensic/src/region.rs       ← region constants
~/src/vhdx-forensic/src/metadata.rs     ← metadata GUIDs and constants
~/src/vhdx-forensic/src/integrity.rs    ← uses crate::header/metadata/region imports
~/src/vhdx-forensic/src/repair.rs       ← uses crate::header/region imports
~/src/vhdx-forensic/tests/libvhdi_compat.rs   ← reader tests to adapt
~/src/vhdx-forensic/tests/real_images.rs      ← reader tests to adapt
```
