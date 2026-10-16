#!/usr/bin/env bash
set -euo pipefail
# Missing local tools must fail rather than trigger rustup downloads.
export RUSTUP_AUTO_INSTALL=0

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

if [[ $# -ne 1 ]]; then
    printf 'Usage: bash scripts/test.sh bootstrap|network|e2e\n' >&2
    exit 2
fi

case "$1" in
    bootstrap)
        listing="$(cargo test --locked --test bootstrap -- --list)"
        count="$(printf '%s\n' "$listing" | awk '/: test$/ { count++ } END { print count+0 }')"
        if [[ "$count" -eq 0 ]]; then
            printf 'ERROR: bootstrap selected 0 tests; refusing an empty pass\n' >&2
            exit 1
        fi
        printf 'bootstrap: %s tests selected\n' "$count"
        cargo test --locked --test bootstrap
        ;;
    network|e2e)
        printf 'NOT_IMPLEMENTED: %s test group\n' "$1" >&2
        exit 2
        ;;
    *)
        printf 'ERROR: unknown test group: %s\n' "$1" >&2
        exit 2
        ;;
esac
