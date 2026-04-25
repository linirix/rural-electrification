# Electrification

A terminal-first economic and management simulator about running a small electric utility in a newly expanding power market.

The first playable loop is deliberately local-market focused: raise capital, borrow, build generation, extend distribution capacity, buy competitors, set rates, market to customers, maintain service quality, and try to become the dominant utility before the market review ends.

The market regulator will not approve a purchase of the last independent rival, so acquisitions can build a dominant utility but do not remove competitive pressure entirely.

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
stock 20000
stock issue 20000
buyback 10000
stock buyback 10000
debt 20000
repay 10000
debt repay 10000
rate up 2
rate down 1
maintenance 4500
buy 1
advance
help
quit
```

Capital action limits:

- `stock issue [amount]` has no fixed proceeds cap. Larger issues are possible, but they face steeper issue discounts, fees, dilution, and reputation pressure.
- `buyback [amount]` has no fixed command cap, but it cannot spend cash you do not have and cannot retire the last public float.
- `debt [amount]` is floating-rate and limited by borrowing room, which is based on the asset base, current debt, and current credit conditions.
- `repay [amount]` is limited by cash on hand and outstanding debt.

Macroeconomic inputs change each quarter. Base rates, credit spreads, demand conditions, and cost pressure affect debt service, borrowing room, growth, operating costs, and equity financing appetite. Higher leverage now raises the annual floating rate paid on outstanding debt, so leveraged firms are more exposed when credit tightens.

## Test And Playtest

```sh
cargo test
cargo run --bin playtest -- --seeds 50
```

The core simulation is separated from terminal rendering so the same model can later support a richer TUI, save files, or a web adaptation.
