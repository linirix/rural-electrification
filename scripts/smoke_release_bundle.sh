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

GAME_BIN="$PACKAGE_DIR/bin/electrification"
PLAYTEST_BIN="$PACKAGE_DIR/bin/playtest"
STRESS_BIN="$PACKAGE_DIR/bin/stress_scan"

for binary in "$GAME_BIN" "$PLAYTEST_BIN" "$STRESS_BIN"; do
    if [[ ! -x "$binary" ]]; then
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
