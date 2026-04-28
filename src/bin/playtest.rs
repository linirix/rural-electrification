use electrification::sim::money;
use electrification::{
    InitialVariance, Strategy, StrategySummary, run_strategy_batch_with_observer,
};

fn main() {
    let options = parse_options();
    if options.sweep_starts {
        run_starting_variance_sweep(&options);
    } else {
        let variance = InitialVariance::new(options.variance.unwrap_or(1.0));
        let strategies = options.strategies();
        for (index, strategy) in strategies.iter().copied().enumerate() {
            if index > 0 {
                println!();
            }
            let summary = run_batch(options.seeds, options.verbose, variance, strategy);
            print_summary(None, strategy, variance, &summary);
        }
    }
}

fn run_starting_variance_sweep(options: &Options) {
    let profiles = [
        ("fixed", InitialVariance::fixed()),
        ("light", InitialVariance::new(0.55)),
        ("moderate", InitialVariance::new(1.0)),
        ("wide", InitialVariance::new(1.35)),
        ("volatile", InitialVariance::new(1.70)),
        ("max", InitialVariance::new(2.0)),
    ];
    let strategies = options.strategies();
    let strategy_label = if matches!(options.strategy, StrategySelection::All) {
        "all"
    } else {
        strategies[0].name()
    };
    println!(
        "starting variance sweep; seeds per profile: {}; strategy: {}",
        options.seeds, strategy_label
    );
    for (strategy_index, strategy) in strategies.iter().copied().enumerate() {
        if strategies.len() > 1 {
            if strategy_index > 0 {
                println!();
            }
            println!("strategy: {}", strategy.name());
        }
        for &(name, variance) in &profiles {
            let summary = run_batch(options.seeds, options.verbose, variance, strategy);
            print_summary(Some(name), strategy, variance, &summary);
        }
    }
}

fn run_batch(
    seeds: u32,
    verbose: bool,
    variance: InitialVariance,
    strategy: Strategy,
) -> StrategySummary {
    run_strategy_batch_with_observer(seeds, variance, strategy, |seed, game, report| {
        if verbose {
            println!(
                "{} {} seed {}: share {:.1}%, customers {:.0}, cash {}, debt {}, reliability {:.0}%, rate {:.1}c ({:.0}% of {:.1}c break-even), profit {}, M&A stress {:.0} pts",
                report.label,
                strategy.name(),
                seed,
                game.market_share() * 100.0,
                game.player.customers,
                money(game.player.cash),
                money(game.player.debt),
                game.player.reliability * 100.0,
                game.player.rate_cents,
                game.player_rate_support_ratio().min(9.99) * 100.0,
                game.player_break_even_rate_cents(),
                money(report.profit),
                game.acquisition_stress * 100.0
            );
            for event in &report.events {
                println!("  event: {event}");
            }
        }
    })
}

fn print_summary(
    profile: Option<&str>,
    strategy: Strategy,
    variance: InitialVariance,
    summary: &StrategySummary,
) {
    if let Some(profile) = profile {
        println!();
        println!(
            "profile: {profile} | strategy: {} | variance amplitude {:.2}",
            strategy.name(),
            variance.amplitude
        );
    } else {
        println!(
            "strategy: {} | variance amplitude {:.2}",
            strategy.name(),
            variance.amplitude
        );
    }

    let runs = summary.runs as f64;
    println!("runs: {}", summary.runs);
    println!(
        "victories: {} ({:.1}%)",
        summary.victories,
        summary.win_rate() * 100.0
    );

    let defeats = summary.defeats();
    if defeats > 0 {
        println!(
            "defeats: board {}{} | receivership {}{} | access {}{}",
            summary.board_defeats,
            avg_quarter_suffix(summary.board_defeats, summary.board_defeat_quarters),
            summary.receivership_defeats,
            avg_quarter_suffix(
                summary.receivership_defeats,
                summary.receivership_defeat_quarters
            ),
            summary.market_access_defeats,
            avg_quarter_suffix(
                summary.market_access_defeats,
                summary.market_access_defeat_quarters
            )
        );
        println!(
            "avg defeat quarter: {:.1} (range Q{:.0}-Q{:.0})",
            summary.defeat_quarters / defeats as f64,
            summary.defeat_quarter_range.min,
            summary.defeat_quarter_range.max
        );
    }

    println!(
        "avg start: share {:.1}%, customers {:.0}, cash {}, debt {}, reliability {:.0}%, rate {:.1}c, headroom {:.0}",
        summary.start_share / runs * 100.0,
        summary.start_customers / runs,
        money(summary.start_cash / runs),
        money(summary.start_debt / runs),
        summary.start_reliability / runs * 100.0,
        summary.start_rate / runs,
        summary.start_headroom / runs
    );
    println!(
        "start ranges: share {:.1}-{:.1}%, customers {:.0}-{:.0}, cash {}-{}, debt {}-{}, reliability {:.0}-{:.0}%, rate {:.1}-{:.1}c, headroom {:.0}-{:.0}",
        summary.start_share_range.min * 100.0,
        summary.start_share_range.max * 100.0,
        summary.start_customers_range.min,
        summary.start_customers_range.max,
        money(summary.start_cash_range.min),
        money(summary.start_cash_range.max),
        money(summary.start_debt_range.min),
        money(summary.start_debt_range.max),
        summary.start_reliability_range.min * 100.0,
        summary.start_reliability_range.max * 100.0,
        summary.start_rate_range.min,
        summary.start_rate_range.max,
        summary.start_headroom_range.min,
        summary.start_headroom_range.max
    );
    println!(
        "avg finish: share {:.1}% (range {:.1}-{:.1}%), customers {:.0}, cash {}, debt {}, reliability {:.0}%",
        summary.average_finish_share() * 100.0,
        summary.finish_share_range.min * 100.0,
        summary.finish_share_range.max * 100.0,
        summary.finish_customers / runs,
        money(summary.finish_cash / runs),
        money(summary.finish_debt / runs),
        summary.finish_reliability / runs * 100.0
    );
    println!(
        "finish share distribution: {}",
        summary.finish_share_histogram.render()
    );
    println!(
        "acquisition stress: avg peak {:.0} pts (range {:.0}-{:.0}) | stressed defeats {} | stressed receiverships {}",
        summary.average_peak_acquisition_stress() * 100.0,
        summary.peak_acquisition_stress_range.min * 100.0,
        summary.peak_acquisition_stress_range.max * 100.0,
        summary.acquisition_stress_defeats,
        summary.acquisition_stress_receiverships
    );
}

fn avg_quarter_suffix(count: u32, quarter_sum: f64) -> String {
    if count == 0 {
        String::new()
    } else {
        format!(" (avg Q{:.1})", quarter_sum / count as f64)
    }
}

struct Options {
    seeds: u32,
    verbose: bool,
    variance: Option<f64>,
    sweep_starts: bool,
    strategy: StrategySelection,
}

impl Options {
    fn strategies(&self) -> Vec<Strategy> {
        match self.strategy {
            StrategySelection::One(strategy) => vec![strategy],
            StrategySelection::All => Strategy::all().to_vec(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StrategySelection {
    One(Strategy),
    All,
}

fn parse_options() -> Options {
    let mut args = std::env::args().skip(1);
    let mut seeds = 25;
    let mut verbose = false;
    let mut variance = None;
    let mut sweep_starts = false;
    let mut strategy = StrategySelection::One(Strategy::Mna);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--seeds" => {
                if let Some(value) = args.next().and_then(|raw| raw.parse::<u32>().ok()) {
                    seeds = value.max(1);
                }
            }
            "--variance" => {
                variance = args.next().and_then(|raw| raw.parse::<f64>().ok());
            }
            "--strategy" => {
                if let Some(raw) = args.next() {
                    if let Some(parsed) = parse_strategy(&raw) {
                        strategy = parsed;
                    } else {
                        eprintln!("unknown strategy '{raw}'; using {}", Strategy::Mna.name());
                    }
                }
            }
            "--all-strategies" => strategy = StrategySelection::All,
            "--sweep-starts" => sweep_starts = true,
            "--verbose" => verbose = true,
            _ => {}
        }
    }
    Options {
        seeds,
        verbose,
        variance,
        sweep_starts,
        strategy,
    }
}

fn parse_strategy(raw: &str) -> Option<StrategySelection> {
    match raw.to_ascii_lowercase().as_str() {
        "all" => Some(StrategySelection::All),
        "naive" | "baseline" => Some(StrategySelection::One(Strategy::Naive)),
        "organic" | "growth" => Some(StrategySelection::One(Strategy::Organic)),
        "balanced" | "hybrid" => Some(StrategySelection::One(Strategy::Balanced)),
        "regional" | "expansion" | "platform" => Some(StrategySelection::One(Strategy::Regional)),
        "mna" | "ma" | "m&a" | "acquire" | "acquisition" | "expert" => {
            Some(StrategySelection::One(Strategy::Mna))
        }
        "raider" | "rollup" | "reckless" => Some(StrategySelection::One(Strategy::Raider)),
        "landshark" | "optimizer" | "ev" => Some(StrategySelection::One(Strategy::Landshark)),
        "costanza" | "opposite" | "contrarian" => Some(StrategySelection::One(Strategy::Costanza)),
        "glonzo" | "random" | "chaos" => Some(StrategySelection::One(Strategy::Glonzo)),
        _ => None,
    }
}
