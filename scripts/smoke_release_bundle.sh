#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
    printf 'usage: %s <dist/electrification-version-target.tar.gz | extracted-directory>\n' "$0" >&2
    exit 2
fi

INPUT="$1"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [[ -d "$INPUT" ]]; then
    PACKAGE_DIR="$INPUT"
else
    case "$INPUT" in
        *.tar.gz | *.tgz)
            CHECKSUM_FILE="$INPUT.sha256"
            if [[ -f "$CHECKSUM_FILE" ]]; then
                expected="$(awk '{ print $1; exit }' "$CHECKSUM_FILE")"
                if command -v sha256sum >/dev/null 2>&1; then
                    actual="$(sha256sum "$INPUT" | awk '{ print $1 }')"
                else
                    actual="$(shasum -a 256 "$INPUT" | awk '{ print $1 }')"
                fi
                if [[ "$actual" != "$expected" ]]; then
                    printf 'checksum mismatch for %s\n' "$INPUT" >&2
                    printf 'expected: %s\nactual:   %s\n' "$expected" "$actual" >&2
                    exit 1
                fi
            fi
            tar -C "$TMP_DIR" -xzf "$INPUT"
            ;;
        *)
            printf 'unsupported release bundle: %s\n' "$INPUT" >&2
            exit 2
            ;;
    esac
    PACKAGE_DIR="$(find "$TMP_DIR" -mindepth 1 -maxdepth 1 -type d -print)"
    PACKAGE_DIR="$(printf '%s\n' "$PACKAGE_DIR" | sed -n '1p')"
fi

EXE_SUFFIX=""
if [[ -f "$PACKAGE_DIR/bin/electrification.exe" ]]; then
    EXE_SUFFIX=".exe"
fi

GAME_BIN="$PACKAGE_DIR/bin/electrification${EXE_SUFFIX}"
PLAYTEST_BIN="$PACKAGE_DIR/bin/playtest${EXE_SUFFIX}"
STRESS_BIN="$PACKAGE_DIR/bin/stress_scan${EXE_SUFFIX}"

for binary in "$GAME_BIN" "$PLAYTEST_BIN" "$STRESS_BIN"; do
    if [[ ! -f "$binary" ]]; then
        printf 'missing executable: %s\n' "$binary" >&2
        exit 1
    fi
done

if [[ ! -f "$PACKAGE_DIR/README.md" || ! -f "$PACKAGE_DIR/RELEASE.txt" ]]; then
    printf 'release package is missing README.md or RELEASE.txt\n' >&2
    exit 1
fi

printf 'quit\n' | NO_COLOR=1 TERM=dumb "$GAME_BIN" > "$TMP_DIR/game.out"
grep -q "more corgi's rural electrification" "$TMP_DIR/game.out"
grep -q "Summary:" "$TMP_DIR/game.out"

"$PLAYTEST_BIN" --strategy organic --seeds 5 > "$TMP_DIR/playtest.out"
grep -q "strategy: organic" "$TMP_DIR/playtest.out"

"$STRESS_BIN" --strategy glonzo --seeds 5 > "$TMP_DIR/stress.out"
grep -q "stress scan" "$TMP_DIR/stress.out"
grep -q "defeat-mode coverage" "$TMP_DIR/stress.out"

printf 'Smoke test passed for %s\n' "$INPUT"
