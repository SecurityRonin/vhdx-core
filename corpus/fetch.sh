#!/usr/bin/env bash
# Generate VHDX corpus images using qemu-img.
# Requires: qemu-utils (sudo apt-get install -y qemu-utils)
set -euo pipefail

DEST="$(cd "$(dirname "$0")" && pwd)"

# Dynamic VHDX (Hyper-V / WSL2 format)
qemu-img create -f vhdx "${DEST}/dynamic.vhdx" 10M
