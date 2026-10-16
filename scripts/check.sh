#!/usr/bin/env bash
set -euo pipefail
# Missing local tools must fail rather than trigger rustup downloads.
export RUSTUP_AUTO_INSTALL=0

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

python3 scripts/roadmap.py validate
python3 scripts/roadmap.py render --check
python3 -m unittest discover -s scripts -p 'test_*.py' -v
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo build --locked --release
