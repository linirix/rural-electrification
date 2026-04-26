# Electrification

A terminal-first economic and management simulator about running a small electric utility in a newly expanding power market.

The first playable loop is deliberately local-market focused: raise capital, borrow, build generation, extend distribution capacity, buy competitors, set rates, market to customers, maintain service quality, and try to become the dominant utility before the market review ends.

The market regulator will not approve a purchase of the last independent rival, and larger deals now require diligence before the purchase command will close. Acquisitions can build a dominant utility, but they carry information costs, public-interest concessions, integration strain, and rival countermeasures.

## Run

```sh
cargo run
```

Useful commands inside the game:

```text
status
build gen
build gen 300
build lines
build lines 500
marketing 4000
issue 20000
stock issue 20000
buyback 10000
stock buyback 10000
borrow 20000
borrow max
repay 10000
repay max
debt repay 10000
rate 10.0
rate up 2
rate down 1
maint 4500
diligence 1
buy 1
preview borrow 10000 diligence 3 buy 3
rivals
board
n
help
quit
```

Capital action limits:

- `issue [amount]` has no fixed proceeds cap. Larger issues are possible, but they face steeper issue discounts, fees, dilution, reputation pressure, and equity-market fatigue.
- `buyback [amount]` has no fixed command cap, but it cannot spend cash you do not have and cannot retire the last public float. Material buybacks retire shares at a premium, reprice the remaining float, and are dampened if they leave the company short on cash.
- `borrow [amount|max]` is floating-rate and limited by borrowing room, which is based on the asset base, current debt, and current credit conditions.
- `repay [amount|max]` is limited by cash on hand and outstanding debt.
- `diligence <number>` reveals exact acquisition terms for a rival for three quarters. Without diligence, the rival screen shows public estimates rather than exact cash, debt, closing cost, and post-deal leverage.
- `buy <number>` requires current diligence, available cash for the closing price, and no active integration cooldown.
- `maintenance [amount]` has diminishing reliability and reputation impact as the asset base grows, so larger systems need larger maintenance budgets.
- `stock issue`, `stock buyback`, `debt`, and `debt repay` remain supported aliases.

Macroeconomic inputs change each quarter. Base rates, credit spreads, demand conditions, and cost pressure affect debt service, borrowing room, growth, operating costs, and equity financing appetite. Higher leverage now raises the annual floating rate paid on outstanding debt, so leveraged firms are more exposed when credit tightens.

## Test And Playtest

```sh
cargo test
cargo run --bin playtest -- --seeds 50
cargo run --bin playtest -- --sweep-starts --seeds 10000
```

Normal games start with seeded variance around the opening market, macro conditions, company balance sheet, service quality, rivals, and capacity position. The playtest sweep compares fixed, light, moderate, wide, volatile, and maximum starting variance profiles.

The core simulation is separated from terminal rendering so the same model can later support a richer TUI, save files, or a web adaptation.
