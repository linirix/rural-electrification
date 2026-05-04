# Electrification

Electrification is a terminal-first strategy game about turning a fragile rural
electric company into a regional power. You set rates, build generation and
lines, borrow, issue stock, hire managers, pursue acquisitions, defend
reliability, and answer to public reviews that care about more than market
share.

The game is local, deterministic, and text-driven: every quarter turns finance,
rates, reliability, reputation, customer churn, rivals, and public tolerance
into a new management problem. Organic growth is reliable but close-run; M&A
can be powerful but brittle; balanced play is usually best.

Although the setting implies the early electrification era, the game intentionally avoids period-piece presentation. The core decisions are timeless management tradeoffs around dollars, watts, customers, reliability, rates, capital structure, and market power.

```text
more corgi's rural electrification!

Year 3 Q2> preview borrow 50000 diligence 2 buy 2
Year 3 Q2> maint 8000
Year 3 Q2> next
```

## Download

Download the latest platform archive from
[GitHub Releases](https://github.com/linirix/rural-electrification/releases/latest):

- macOS Apple Silicon: `electrification-0.1.5-macos-arm64.tar.gz`
- Linux x64: `electrification-0.1.5-linux-x64.tar.gz`
- Windows x64: `electrification-0.1.5-windows-x64.tar.gz`

Each archive has a matching `.sha256` checksum file.

## Quick Start

On macOS:

```sh
tar -xzf electrification-0.1.5-macos-arm64.tar.gz
cd electrification-0.1.5-macos-arm64
./bin/electrification
```

On Linux:

```sh
tar -xzf electrification-0.1.5-linux-x64.tar.gz
cd electrification-0.1.5-linux-x64
./bin/electrification
```

On Windows PowerShell:

```powershell
tar -xzf electrification-0.1.5-windows-x64.tar.gz
cd electrification-0.1.5-windows-x64
.\bin\electrification.exe
```

For a forgiving first campaign, start the curated release seed:

```sh
./bin/electrification --seed 106318
```

The same game can also be started from the playtest seed noted in
`PLAYTEST_SEEDS.md`:

```sh
./bin/electrification --playtest-seed 106
```

Useful launch flags:

```text
--seed <game-seed>          start a deterministic game seed
--playtest-seed <seed>      start the game seed used by a playtest seed
--sandbox                   disable board review constraints
```

## Verify Downloads

On macOS:

```sh
shasum -a 256 -c electrification-0.1.5-macos-arm64.tar.gz.sha256
```

On Linux:

```sh
sha256sum -c electrification-0.1.5-linux-x64.tar.gz.sha256
```

On Windows PowerShell, compare the printed hash with the `.sha256` file:

```powershell
Get-FileHash electrification-0.1.5-windows-x64.tar.gz -Algorithm SHA256
Get-Content electrification-0.1.5-windows-x64.tar.gz.sha256
```

If macOS blocks the downloaded binary, remove the quarantine attribute from the
extracted directory:

```sh
xattr -dr com.apple.quarantine .
```

## Terminal Notes And Known Issues

- Use a modern UTF-8 terminal with ANSI color support: Terminal.app, iTerm2,
  GNOME Terminal, Konsole, or Windows Terminal.
- A width of 120 columns or more gives the dashboard its intended layout.
- If colors or box drawing are hard to read, run with `NO_COLOR=1`.
- On Windows, prefer Windows Terminal over the legacy console.
- Save files live under `~/.electrification/` by default.
- The game is intentionally terminal-first; there is no graphical frontend.

## Run From Source

```sh
cargo run
```

`Cargo.toml` sets `electrification` as the default binary. To be explicit:

```sh
cargo run --bin electrification
```

For a faster release build:

```sh
cargo run --release --bin electrification
```

Set `NO_COLOR=1` or use `TERM=dumb` to disable ANSI color.

## Core Commands

```text
status / s
next / n / end
preview <command>
quote <command>

build gen [MWh]
build lines [customers]
expand
marketing [amount]
maint [amount]
rate 10.0
rate up 2
rate down 1

issue [amount]
buyback [amount]
dividend [amount]
borrow [amount|max]
repay [amount|max]
diligence <rival-number>
buy <rival-number>

rivals
board
sandbox
save [name]
load [name]
help / ?
quit / exit
```

Examples:

```text
preview borrow 10000 diligence 3 buy 3
borrow max
repay max
marketing 4
maint 6
save campaign1
load campaign1
```

For `marketing` and `maint`, a single digit means thousands of dollars: `marketing 4`
spends $4,000 and `maint 6` spends $6,000. Other money commands stay literal unless
you use `k` notation.

## Game Systems

- `issue [amount]` has no fixed proceeds cap. Larger issues are possible, but they face steeper discounts, fees, dilution, reputation pressure, and equity-market fatigue.
- `buyback [amount]` has no fixed command cap, but it cannot spend cash you do not have or retire the last public float. Material buybacks retire shares at a premium and reprice the remaining float.
- `dividend [amount]` pays a company-wide dividend pro rata to all shareholders. Company cash falls by the full amount, while founder wealth receives only the founder-owned share.
- `borrow [amount|max]` is floating-rate and limited by borrowing room, which depends on asset base, current debt, and credit conditions.
- `repay [amount|max]` is limited by cash on hand and outstanding debt.
- `diligence <number>` reveals exact acquisition terms for a rival for three quarters and freezes that target's acquisition quote during the diligence window. It can leak to the target, triggering defensive financing, retention, rate, and capacity moves.
- `buy <number>` can close with or without diligence, but no-diligence deals rely on public estimates and can surprise you at closing. You still need available cash for the closing price and no active integration cooldown.
- `maintenance [amount]` has diminishing reliability and reputation impact as the asset base grows. Larger systems need larger upkeep budgets; the dashboard will flag when asset scale makes each maintenance dollar less responsive. After adjacent expansion, maintenance also helps work down regional integration burden.
- `expand` starts adjacent-territory entry once the core platform is mature enough. Up to three adjacent entries are allowed, with each follow-on expansion costing more and creating temporary regional integration work.
- After the formal Year 5 review, continued games receive a Year 10 regional mandate: hold broad regional share, enter multiple adjacent territories, keep reliability high, and stay inside board leverage limits.
- `sandbox` disables board reviews, mandates, and checkpoint penalties for the current run. Operating failures still apply, including market-access loss, receivership, and hostile takeover.

Macroeconomic inputs change each quarter. Base rates, credit spreads, demand conditions, cost pressure, public rate tolerance, and active shocks affect debt service, borrowing room, growth, operating costs, customer churn, startup formation, and equity financing appetite.

Rivals are not passive. They can cut rates, market, build capacity, consolidate with each other, and counterattack after the player becomes dominant. The regulator will not approve a purchase of the final independent rival.

## Saves

Save files are pretty-printed JSON and live outside the repo by default:

```text
~/.electrification/<name>.json
```

If no name is supplied, the game uses `autosave`:

```text
save
load
```

Save names are restricted to letters, numbers, `-`, and `_`.

## Test And Playtest

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt -- --check
```

The strategy harness is the main balance tool:

```sh
cargo run --bin playtest -- --seeds 50
cargo run --bin playtest -- --strategy organic --seeds 500
cargo run --bin playtest -- --strategy regional --seeds 500
cargo run --bin playtest -- --strategy raider --seeds 500
cargo run --bin playtest -- --strategy glonzo --seeds 500
cargo run --bin playtest -- --strategy dividend --seeds 500
cargo run --bin playtest -- --strategy manager --seeds 500
cargo run --bin playtest -- --all-strategies --seeds 500
cargo run --bin playtest -- --sweep-starts --all-strategies --seeds 1000
cargo run --bin stress_scan -- --strategy raider --seeds 2000
```

Useful playtest flags:

- `--strategy naive|organic|balanced|regional|ma|raider|landshark|costanza|glonzo|dividend|manager|austerity|junkbond|ratehawk|discountrunner|distressedbuyer|all`
- `--all-strategies`
- `--seeds <count>`
- `--variance <amplitude>`
- `--sweep-starts`
- `--verbose`

Playtest summaries include peak acquisition stress so reckless roll-up strategies can be evaluated separately from disciplined M&A. `stress_scan` accepts `--strategy`, `--seeds`, and `--variance`, and adds cross-strategy edge-case and defeat-mode coverage output. There is also a strategy regression test in `tests/strategy_balance.rs` that keeps the major automated strategies inside expected win-rate bands.

For deeper balance context, see `PLAYTEST_MEMO_1000008.md` for the million-game stress memo and `PLAYTEST_SEEDS.md` for interesting or useful seed notes. The main balance constants live near the top of `src/sim.rs`, grouped by domain so tuning changes are easy to audit.

## Release

Before cutting a release, run the full local gate:

```sh
cargo fmt --check
cargo clippy --release --all-targets -- -D warnings
cargo test --release
cargo run --release --bin playtest -- --all-strategies --seeds 1000
```

Build a distributable package for the current machine:

```sh
./scripts/package_release.sh
./scripts/smoke_release_bundle.sh "$(ls -t dist/electrification-0.1.5-*.tar.gz | head -n 1)"
```

The package includes:

- `bin/electrification` for the interactive game
- `bin/playtest` for balance sweeps
- `bin/stress_scan` for edge-case scanning
- `README.md`, `RELEASE_NOTES.md`, `PLAYTEST_SEEDS.md`, `Cargo.lock`, and `RELEASE.txt`
- a matching `.sha256` checksum file for the archive

GitHub Actions are configured for:

- `CI`: format, clippy, release tests, and a 1000-seed all-strategy smoke run on pushes and pull requests.
- `Release`: package and smoke-test macOS, Linux, and Windows tarballs on `v*` tags or manual dispatch.

On a `v*` tag push, the release workflow also creates or updates the matching GitHub Release and uploads all platform tarballs as release assets.

Suggested release flow:

```sh
git status
./scripts/package_release.sh
./scripts/smoke_release_bundle.sh "$(ls -t dist/electrification-0.1.5-*.tar.gz | head -n 1)"
git tag v0.1.5
git push origin main --tags
```

## Project Layout

```text
src/lib.rs              crate exports
src/main.rs             interactive terminal entry point
src/terminal.rs         terminal entry point and shared terminal constants
src/terminal/command.rs command parsing, aliases, save/load
src/terminal/preview.rs command preview and chained transaction preview
src/terminal/screens.rs dashboard, board, notices, and report screens
src/terminal/rivals.rs  rival overview/detail rendering
src/terminal/render.rs  ANSI styling, box rendering, wrapping, tones
src/terminal/tests.rs   terminal command and rendering tests
src/sim.rs              core game state, decisions, quarter advancement
src/sim/economics.rs    settlement, churn, project costs, maintenance
src/sim/competitors.rs  rival behavior, startups, rival mergers
src/sim/attribution.rs  quarter-result explanations
src/strategy.rs         automated strategy policies and summaries
src/bin/playtest.rs     strategy harness CLI
src/bin/stress_scan.rs  broad edge-case scan CLI
scripts/package_release.sh
scripts/smoke_release_bundle.sh
tests/strategy_balance.rs
```

`Cargo.lock` is intentionally tracked because this is an application/game, not a library-only crate.
