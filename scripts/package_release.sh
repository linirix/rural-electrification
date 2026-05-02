#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

VERSION="$(awk -F '"' '/^version =/ { print $2; exit }' Cargo.toml)"
HOST="$(rustc -vV | awk '/^host:/ { host = $2 } END { print host }')"
PACKAGE_NAME="electrification-${VERSION}-${HOST}"
DIST_DIR="$ROOT/dist"
STAGE_DIR="$DIST_DIR/$PACKAGE_NAME"
ARCHIVE="$DIST_DIR/$PACKAGE_NAME.tar.gz"

rm -rf "$STAGE_DIR" "$ARCHIVE"
mkdir -p "$STAGE_DIR/bin"

cargo build --release --locked --bins

BIN_DIR="$ROOT/target/release"
EXE_SUFFIX=""
if [[ -f "$BIN_DIR/electrification.exe" ]]; then
    EXE_SUFFIX=".exe"
fi

copy_binary() {
    local name="$1"
    local source="$BIN_DIR/${name}${EXE_SUFFIX}"
    if [[ ! -f "$source" ]]; then
        printf 'missing release binary: %s\n' "$source" >&2
        exit 1
    fi
    cp "$source" "$STAGE_DIR/bin/"
}

copy_binary electrification
copy_binary playtest
copy_binary stress_scan
cp "$ROOT/README.md" "$STAGE_DIR/"
cp "$ROOT/PLAYTEST_SEEDS.md" "$STAGE_DIR/"
cp "$ROOT/Cargo.lock" "$STAGE_DIR/"

{
    printf 'Electrification %s\n' "$VERSION"
    printf 'Target: %s\n' "$HOST"
    printf '\nRun the game:\n'
    if [[ -n "$EXE_SUFFIX" ]]; then
        printf '  .\\bin\\electrification.exe\n'
    else
        printf '  ./bin/electrification\n'
    fi
    printf '\nDeveloper balance tools included:\n'
    if [[ -n "$EXE_SUFFIX" ]]; then
        printf '  .\\bin\\playtest.exe --all-strategies --seeds 500\n'
        printf '  .\\bin\\stress_scan.exe --seeds 10000\n'
    else
        printf '  ./bin/playtest --all-strategies --seeds 500\n'
        printf '  ./bin/stress_scan --seeds 10000\n'
    fi
} > "$STAGE_DIR/RELEASE.txt"

tar -C "$DIST_DIR" -czf "$ARCHIVE" "$PACKAGE_NAME"

printf 'Packaged %s\n' "$ARCHIVE"
