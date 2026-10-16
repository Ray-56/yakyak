#!/usr/bin/env bash
set -euo pipefail
export RUSTUP_AUTO_INSTALL=0

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
failed=0

required() {
    local label="$1"
    shift
    local output
    if output="$("$@" 2>&1)"; then
        printf 'required OK: %s — %s\n' "$label" "$output"
    else
        printf 'required MISSING/ERROR: %s — %s\n' "$label" "$output" >&2
        failed=1
    fi
}

required Git git --version
required Python python3 -c 'import sys; print(sys.version.split()[0]); sys.exit(0 if sys.version_info >= (3, 10) else "Python >=3.10 is required")'

toolchain="$(sed -n 's/^channel = "\([0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*\)"$/\1/p' rust-toolchain.toml)"
if [[ -z "$toolchain" ]] || [[ "$toolchain" == *$'\n'* ]]; then
    printf 'required ERROR: rust-toolchain.toml must declare one exact stable channel\n' >&2
    failed=1
elif ! command -v rustup >/dev/null 2>&1; then
    printf 'required MISSING: rustup (cannot inspect repository toolchain %s)\n' "$toolchain" >&2
    failed=1
else
    # rustup run fails for an absent toolchain instead of installing one.
    if rust_version="$(rustup run "$toolchain" rustc --version 2>&1)"; then
        if [[ "$rust_version" == "rustc $toolchain "* ]]; then
            printf 'required OK: Rust — %s\n' "$rust_version"
        else
            printf 'required ERROR: Rust version mismatch; expected %s, got %s\n' "$toolchain" "$rust_version" >&2
            failed=1
        fi
    else
        printf 'required MISSING/ERROR: Rust %s — %s\n' "$toolchain" "$rust_version" >&2
        failed=1
    fi
    required Cargo rustup run "$toolchain" cargo --version
    required rustfmt rustup run "$toolchain" rustfmt --version
    required Clippy rustup run "$toolchain" cargo clippy --version
fi

for optional_tool in sipp tcpdump tshark; do
    if command -v "$optional_tool" >/dev/null 2>&1; then
        printf 'optional AVAILABLE: %s (presence only; lab readiness not verified)\n' "$optional_tool"
    else
        printf 'optional MISSING: %s\n' "$optional_tool"
    fi
done

exit "$failed"
