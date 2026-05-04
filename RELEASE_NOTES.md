# Electrification v0.1.5

Electrification is a terminal-first strategy game about turning a small rural
electric utility into a durable regional power. You manage rates, reliability,
capital structure, customer growth, acquisitions, public reviews, and rival
responses quarter by quarter.

## What To Download

- macOS Apple Silicon: `electrification-0.1.5-macos-arm64.tar.gz`
- Linux x64: `electrification-0.1.5-linux-x64.tar.gz`
- Windows x64: `electrification-0.1.5-windows-x64.tar.gz`

Each archive includes the interactive game plus the `playtest` and
`stress_scan` balance tools. Matching `.sha256` files are attached for checksum
verification.

## Quick Start

macOS:

```sh
tar -xzf electrification-0.1.5-macos-arm64.tar.gz
cd electrification-0.1.5-macos-arm64
./bin/electrification
```

Linux:

```sh
tar -xzf electrification-0.1.5-linux-x64.tar.gz
cd electrification-0.1.5-linux-x64
./bin/electrification
```

Windows PowerShell:

```powershell
tar -xzf electrification-0.1.5-windows-x64.tar.gz
cd electrification-0.1.5-windows-x64
.\bin\electrification.exe
```

For a forgiving first campaign:

```sh
./bin/electrification --seed 106318
```

## Release Highlights

- Launch-facing README and release notes with clearer install, verification,
  terminal, and first-run guidance.
- A usable curated-seed entry point via `--seed` and `--playtest-seed`.
- Release bundle smoke tests now exercise save/load, not only startup and quit.
- Public release workflow now publishes this release-note text instead of a
  generic placeholder.

## Terminal Notes

Use a modern UTF-8 terminal with ANSI support. A width of 120 columns or more is
recommended. Set `NO_COLOR=1` or use `TERM=dumb` if colors or box drawing are
hard to read. On Windows, prefer Windows Terminal.
