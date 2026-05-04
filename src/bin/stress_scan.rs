use std::collections::BTreeMap;

use electrification::sim::money;
use electrification::{
    Game, InitialVariance, OutcomeKind, QuarterReport, Strategy, StrategySummary,
    run_strategy_batch_with_observer,
};

fn main() {
    let options = Options::parse();
    let variance = InitialVariance::new(options.variance);
    println!(
        "stress scan; seeds: {}; variance amplitude {:.2}; strategies: {}",
        options.seeds,
        variance.amplitude,
        options
            .strategies()
            .iter()
            .map(|strategy| strategy.name())
            .collect::<Vec<_>>()
            .join(", ")
    );

    let mut scans = Vec::new();
    for strategy in options.strategies() {
        let scan = scan_strategy(options.seeds, variance, strategy);
        print_strategy_line(&scan.summary, strategy);
        scans.push(scan);
    }

    println!();
    print_defeat_mode_coverage(&scans);
    println!();
    print_cross_strategy_findings(&scans);
    println!();
    print_edge_cases(&scans);
}

struct Options {
    seeds: u32,
    variance: f64,
    strategy: Option<Strategy>,
}

impl Options {
    fn parse() -> Self {
        let mut args = std::env::args().skip(1);
        let mut seeds = 10_000;
        let mut variance = 1.0;
        let mut strategy = None;
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--seeds" => {
                    if let Some(value) = args.next().and_then(|raw| raw.parse::<u32>().ok()) {
                        seeds = value.max(1);
                    }
                }
                "--variance" => {
                    if let Some(value) = args.next().and_then(|raw| raw.parse::<f64>().ok()) {
                        variance = value.clamp(0.0, 2.0);
                    }
                }
                "--strategy" => {
                    if let Some(raw) = args.next() {
                        strategy = parse_strategy(&raw);
                        if strategy.is_none() {
                            eprintln!("unknown strategy '{raw}'; scanning all strategies");
                        }
                    }
                }
                _ => {}
            }
        }
        Self {
            seeds,
            variance,
            strategy,
        }
    }

    fn strategies(&self) -> Vec<Strategy> {
        self.strategy
            .map(|strategy| vec![strategy])
            .unwrap_or_else(|| Strategy::all().to_vec())
    }
}

fn parse_strategy(raw: &str) -> Option<Strategy> {
    match raw.to_ascii_lowercase().as_str() {
        "naive" | "baseline" => Some(Strategy::Naive),
        "organic" | "growth" => Some(Strategy::Organic),
        "balanced" | "hybrid" => Some(Strategy::Balanced),
        "regional" | "expansion" | "platform" => Some(Strategy::Regional),
        "ma" | "m&a" | "acquire" | "acquisition" | "expert" => Some(Strategy::Ma),
        "raider" | "rollup" | "reckless" => Some(Strategy::Raider),
        "landshark" | "optimizer" | "ev" => Some(Strategy::Landshark),
        "costanza" | "opposite" | "contrarian" => Some(Strategy::Costanza),
        "glonzo" | "random" | "chaos" => Some(Strategy::Glonzo),
        "dividend" | "dividends" | "income" => Some(Strategy::Dividend),
        "manager" | "managers" | "autopilot" => Some(Strategy::Manager),
        "austerity" | "bootstrapped" | "cashfunded" => Some(Strategy::Austerity),
        "junkbond" | "junk" | "leveredorganic" => Some(Strategy::Junkbond),
        "ratehawk" | "pricing" | "highrate" => Some(Strategy::Ratehawk),
        "discountrunner" | "discount" | "lowrate" => Some(Strategy::Discountrunner),
        "distressedbuyer" | "distressed" | "distress" => Some(Strategy::Distressedbuyer),
        _ => None,
    }
}

struct StrategyScan {
    summary: StrategySummary,
    traces: Vec<Trace>,
}

#[derive(Clone)]
struct Trace {
    seed: u32,
    game_seed: u64,
    strategy: Strategy,
    final_quarter: u32,
    outcome: String,
    outcome_details: String,
    victory: bool,
    final_share: f64,
    final_cash: f64,
    final_debt: f64,
    final_reliability: f64,
    final_profit: f64,
    max_share: f64,
    min_cash: f64,
    min_reliability: f64,
    min_profit: f64,
    max_profit: f64,
    max_debt_to_assets: f64,
    max_acquisition_stress: f64,
}

impl Trace {
    fn new(seed: u32, strategy: Strategy) -> Self {
        Self {
            seed,
            game_seed: seed as u64 * 1_003,
            strategy,
            final_quarter: 0,
            outcome: "unfinished".to_string(),
            outcome_details: String::new(),
            victory: false,
            final_share: 0.0,
            final_cash: 0.0,
            final_debt: 0.0,
            final_reliability: 0.0,
            final_profit: 0.0,
            max_share: 0.0,
            min_cash: f64::INFINITY,
            min_reliability: f64::INFINITY,
            min_profit: f64::INFINITY,
            max_profit: f64::NEG_INFINITY,
            max_debt_to_assets: 0.0,
            max_acquisition_stress: 0.0,
        }
    }

    fn observe(&mut self, game: &Game, report: &QuarterReport) {
        let share = game.market_share();
        self.final_quarter = game.quarter;
        self.final_share = share;
        self.final_cash = game.player.cash;
        self.final_debt = game.player.debt;
        self.final_reliability = game.player.reliability;
        self.final_profit = report.profit;
        self.max_share = self.max_share.max(share);
        self.min_cash = self.min_cash.min(game.player.cash);
        self.min_reliability = self.min_reliability.min(game.player.reliability);
        self.min_profit = self.min_profit.min(report.profit);
        self.max_profit = self.max_profit.max(report.profit);
        self.max_debt_to_assets = self.max_debt_to_assets.max(game.player.debt_to_assets());
        self.max_acquisition_stress = self.max_acquisition_stress.max(game.acquisition_stress);
        if let Some(outcome) = &game.outcome {
            self.outcome = outcome.headline.clone();
            self.outcome_details = outcome.details.clone();
            self.victory = outcome.kind == OutcomeKind::Victory;
        }
    }
}

fn scan_strategy(seeds: u32, variance: InitialVariance, strategy: Strategy) -> StrategyScan {
    let mut traces = vec![None; seeds as usize + 1];
    let summary =
        run_strategy_batch_with_observer(seeds, variance, strategy, |seed, game, report| {
            let slot = &mut traces[seed as usize];
            if slot.is_none() {
                *slot = Some(Trace::new(seed, strategy));
            }
            slot.as_mut().unwrap().observe(game, report);
        });
    StrategyScan {
        summary,
        traces: traces.into_iter().flatten().collect(),
    }
}

fn print_strategy_line(summary: &StrategySummary, strategy: Strategy) {
    println!(
        "{:>10}: win {:5.1}% | finish {:5.1}% ({:4.1}-{:4.1}) | defeats board {} recv {} access {} | stress avg {:4.0} max {:4.0}",
        strategy.name(),
        summary.win_rate() * 100.0,
        summary.average_finish_share() * 100.0,
        summary.finish_share_range.min * 100.0,
        summary.finish_share_range.max * 100.0,
        summary.board_defeats,
        summary.receivership_defeats,
        summary.market_access_defeats,
        summary.average_peak_acquisition_stress() * 100.0,
        summary.peak_acquisition_stress_range.max * 100.0,
    );
}

fn print_cross_strategy_findings(scans: &[StrategyScan]) {
    println!("cross-strategy seed findings");
    if scans.len() < Strategy::all().len() {
        println!("  focused strategy scan; cross-strategy seed comparisons skipped");
        return;
    }
    let rows = traces_by_seed(scans);
    let deliberate = [
        Strategy::Organic,
        Strategy::Balanced,
        Strategy::Ma,
        Strategy::Landshark,
    ];

    let hard_but_fair = rows
        .values()
        .filter(|row| {
            let organic = trace_for(row, Strategy::Organic);
            let balanced = trace_for(row, Strategy::Balanced);
            let landshark = trace_for(row, Strategy::Landshark);
            organic.is_some_and(|trace| !trace.victory)
                && balanced.is_some_and(|trace| trace.victory)
                && landshark.is_some_and(|trace| trace.victory)
        })
        .take(12)
        .collect::<Vec<_>>();
    print_seed_rows(
        "hard but fair (organic loses; balanced and landshark win)",
        &hard_but_fair,
    );

    let high_risk_payoff = rows
        .values()
        .filter(|row| {
            trace_for(row, Strategy::Organic).is_some_and(|trace| !trace.victory)
                && trace_for(row, Strategy::Ma)
                    .is_some_and(|trace| trace.victory && trace.final_share >= 0.50)
        })
        .take(12)
        .collect::<Vec<_>>();
    print_seed_rows(
        "high-risk payoff (organic loses; M&A wins)",
        &high_risk_payoff,
    );

    let very_hard = rows
        .values()
        .filter(|row| {
            deliberate
                .iter()
                .all(|strategy| trace_for(row, *strategy).is_some_and(|trace| !trace.victory))
        })
        .take(12)
        .collect::<Vec<_>>();
    print_seed_rows("very hard (all deliberate Y5 strategies lose)", &very_hard);

    let easy_openers = rows
        .values()
        .filter(|row| trace_for(row, Strategy::Naive).is_some_and(|trace| trace.victory))
        .take(12)
        .collect::<Vec<_>>();
    print_seed_rows("favorable openers (naive wins)", &easy_openers);
}

fn print_defeat_mode_coverage(scans: &[StrategyScan]) {
    let traces = scans
        .iter()
        .flat_map(|scan| scan.traces.iter())
        .collect::<Vec<_>>();
    let total = traces.len();
    let board = traces
        .iter()
        .filter(|trace| !trace.victory && trace.outcome.contains("Board"))
        .count();
    let receivership = traces
        .iter()
        .filter(|trace| !trace.victory && trace.outcome.contains("Receivership"))
        .count();
    let acquisition_stress_receivership = traces
        .iter()
        .filter(|trace| {
            !trace.victory
                && trace.outcome.contains("Receivership")
                && trace.outcome_details.contains("strained acquisition")
        })
        .count();
    let hostile_takeover = traces
        .iter()
        .filter(|trace| !trace.victory && trace.outcome.contains("Hostile Takeover"))
        .count();
    let market_access = traces
        .iter()
        .filter(|trace| !trace.victory && trace.outcome.contains("Market Access"))
        .count();
    println!("defeat-mode coverage across {total} games");
    println!("  board review:            {board}");
    println!("  receivership:            {receivership}");
    println!("  market access:           {market_access}");
    println!("  hostile takeover:        {hostile_takeover}");
    println!("  acquisition stress recv: {acquisition_stress_receivership}");
}

fn print_edge_cases(scans: &[StrategyScan]) {
    println!("edge cases and candidate release seeds");
    let mut all = scans
        .iter()
        .flat_map(|scan| scan.traces.iter().cloned())
        .collect::<Vec<_>>();

    print_top(
        "high-share board defeats",
        &mut all,
        |trace| !trace.victory && trace.final_share >= 0.55 && trace.outcome.contains("Board"),
        |trace| trace.final_share,
    );
    print_top(
        "largest acquisition-stress runs",
        &mut all,
        |trace| trace.max_acquisition_stress >= 0.60,
        |trace| trace.max_acquisition_stress,
    );
    print_top(
        "earliest terminal collapses",
        &mut all,
        |trace| !trace.victory && trace.final_quarter < 20,
        |trace| -(trace.final_quarter as f64),
    );
    print_top(
        "lowest-cash survivals",
        &mut all,
        |trace| trace.min_cash < -2_000.0,
        |trace| -trace.min_cash,
    );
    print_top(
        "close Y5 victories",
        &mut all,
        |trace| {
            trace.victory
                && trace.final_quarter == 20
                && (0.45..=0.53).contains(&trace.final_share)
                && trace.final_profit > -500.0
        },
        |trace| -(trace.final_share - 0.455).abs(),
    );
    print_top(
        "close Y10 regional victories",
        &mut all,
        |trace| {
            trace.strategy == Strategy::Regional
                && trace.victory
                && trace.final_quarter == 40
                && (0.58..=0.64).contains(&trace.final_share)
        },
        |trace| -(trace.final_share - 0.585).abs(),
    );
}

fn traces_by_seed(scans: &[StrategyScan]) -> BTreeMap<u32, Vec<&Trace>> {
    let mut rows = BTreeMap::new();
    for scan in scans {
        for trace in &scan.traces {
            rows.entry(trace.seed).or_insert_with(Vec::new).push(trace);
        }
    }
    rows
}

fn trace_for<'a>(row: &'a [&'a Trace], strategy: Strategy) -> Option<&'a Trace> {
    row.iter().copied().find(|trace| trace.strategy == strategy)
}

fn print_seed_rows(label: &str, rows: &[&Vec<&Trace>]) {
    println!();
    println!("{label}");
    if rows.is_empty() {
        println!("  none found in this window");
        return;
    }
    for row in rows {
        let seed = row[0].seed;
        let parts = row
            .iter()
            .filter(|trace| {
                matches!(
                    trace.strategy,
                    Strategy::Naive
                        | Strategy::Organic
                        | Strategy::Balanced
                        | Strategy::Ma
                        | Strategy::Landshark
                        | Strategy::Costanza
                )
            })
            .map(|trace| {
                format!(
                    "{} {} {:.1}%",
                    trace.strategy.name(),
                    if trace.victory { "W" } else { "L" },
                    trace.final_share * 100.0
                )
            })
            .collect::<Vec<_>>()
            .join(" | ");
        println!("  seed {seed} (game {}): {parts}", row[0].game_seed);
    }
}

fn print_top<F, S>(label: &str, traces: &mut [Trace], filter: F, score: S)
where
    F: Fn(&Trace) -> bool,
    S: Fn(&Trace) -> f64,
{
    let mut selected = traces
        .iter()
        .filter(|trace| filter(trace))
        .cloned()
        .collect::<Vec<_>>();
    selected.sort_by(|left, right| score(right).total_cmp(&score(left)));
    selected.truncate(12);

    println!();
    println!("{label}");
    if selected.is_empty() {
        println!("  none found in this window");
        return;
    }
    for trace in selected {
        println!("  {}", trace_line(&trace));
    }
}

fn trace_line(trace: &Trace) -> String {
    format!(
        "seed {} game {} {} {} Q{} share {:.1}% max {:.1}% cash {} min {} debt {} rel {:.0}% min {:.0}% profit {} stress {:.0} pts lev {:.0}% {}",
        trace.seed,
        trace.game_seed,
        trace.strategy.name(),
        if trace.victory { "W" } else { "L" },
        trace.final_quarter,
        trace.final_share * 100.0,
        trace.max_share * 100.0,
        money(trace.final_cash),
        money(trace.min_cash),
        money(trace.final_debt),
        trace.final_reliability * 100.0,
        trace.min_reliability * 100.0,
        money(trace.final_profit),
        trace.max_acquisition_stress * 100.0,
        trace.max_debt_to_assets * 100.0,
        trace.outcome
    )
}
