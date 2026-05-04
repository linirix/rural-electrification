#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

VERSION="$(awk -F '"' '/^version =/ { print $2; exit }' Cargo.toml)"
HOST="$(rustc -vV | awk '/^host:/ { host = $2 } END { print host }')"
case "$HOST" in
    aarch64-apple-darwin) PLATFORM="macos-arm64" ;;
    x86_64-apple-darwin) PLATFORM="macos-x64" ;;
    x86_64-unknown-linux-gnu) PLATFORM="linux-x64" ;;
    x86_64-pc-windows-msvc) PLATFORM="windows-x64" ;;
    *) PLATFORM="$HOST" ;;
esac
PACKAGE_NAME="electrification-${VERSION}-${PLATFORM}"
DIST_DIR="$ROOT/dist"
STAGE_DIR="$DIST_DIR/$PACKAGE_NAME"
ARCHIVE="$DIST_DIR/$PACKAGE_NAME.tar.gz"
CHECKSUM="$ARCHIVE.sha256"

rm -rf "$STAGE_DIR" "$ARCHIVE" "$CHECKSUM"
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
if [[ -f "$ROOT/RELEASE_NOTES.md" ]]; then
    cp "$ROOT/RELEASE_NOTES.md" "$STAGE_DIR/"
fi

{
    printf 'Electrification %s\n' "$VERSION"
    printf 'Platform: %s\n' "$PLATFORM"
    printf 'Rust target: %s\n' "$HOST"
    printf '\nRun the game:\n'
    if [[ -n "$EXE_SUFFIX" ]]; then
        printf '  .\\bin\\electrification.exe\n'
    else
        printf '  ./bin/electrification\n'
    fi
    printf '\nRecommended first campaign:\n'
    if [[ -n "$EXE_SUFFIX" ]]; then
        printf '  .\\bin\\electrification.exe --seed 106318\n'
    else
        printf '  ./bin/electrification --seed 106318\n'
    fi
    printf '\nDeveloper balance tools included:\n'
    if [[ -n "$EXE_SUFFIX" ]]; then
        printf '  .\\bin\\playtest.exe --all-strategies --seeds 1000\n'
        printf '  .\\bin\\stress_scan.exe --seeds 10000\n'
    else
        printf '  ./bin/playtest --all-strategies --seeds 1000\n'
        printf '  ./bin/stress_scan --seeds 10000\n'
    fi
} > "$STAGE_DIR/RELEASE.txt"

tar -C "$DIST_DIR" -czf "$ARCHIVE" "$PACKAGE_NAME"

ARCHIVE_BASENAME="$(basename "$ARCHIVE")"
if command -v sha256sum >/dev/null 2>&1; then
    (cd "$DIST_DIR" && sha256sum "$ARCHIVE_BASENAME") > "$CHECKSUM"
else
    (cd "$DIST_DIR" && shasum -a 256 "$ARCHIVE_BASENAME") > "$CHECKSUM"
fi

printf 'Packaged %s\n' "$ARCHIVE"
printf 'Checksum %s\n' "$CHECKSUM"
