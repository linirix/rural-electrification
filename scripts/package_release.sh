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

cp "$ROOT/target/release/electrification" "$STAGE_DIR/bin/"
cp "$ROOT/target/release/playtest" "$STAGE_DIR/bin/"
cp "$ROOT/target/release/stress_scan" "$STAGE_DIR/bin/"
cp "$ROOT/README.md" "$STAGE_DIR/"
cp "$ROOT/PLAYTEST_SEEDS.md" "$STAGE_DIR/"
cp "$ROOT/Cargo.lock" "$STAGE_DIR/"

{
    printf 'Electrification %s\n' "$VERSION"
    printf 'Target: %s\n' "$HOST"
    printf '\nRun the game:\n'
    printf '  ./bin/electrification\n'
    printf '\nDeveloper balance tools included:\n'
    printf '  ./bin/playtest --all-strategies --seeds 500\n'
    printf '  ./bin/stress_scan --seeds 10000\n'
} > "$STAGE_DIR/RELEASE.txt"

tar -C "$DIST_DIR" -czf "$ARCHIVE" "$PACKAGE_NAME"

printf 'Packaged %s\n' "$ARCHIVE"
