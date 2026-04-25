use electrification::{
    DISTRIBUTION_PROJECT_CAPACITY, Decision, GENERATION_PROJECT_CAPACITY_MWH, Game,
    InitialVariance, OutcomeKind,
};

fn main() {
    let options = parse_options();
    if options.sweep_starts {
        run_starting_variance_sweep(&options);
    } else {
        let variance = InitialVariance::new(options.variance.unwrap_or(1.0));
        let summary = run_batch(options.seeds, options.verbose, variance);
        print_summary(None, variance, &summary);
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
    println!(
        "starting variance sweep; seeds per profile: {}",
        options.seeds
    );
    for (name, variance) in profiles {
        let summary = run_batch(options.seeds, options.verbose, variance);
        print_summary(Some(name), variance, &summary);
    }
}

fn run_batch(seeds: u32, verbose: bool, variance: InitialVariance) -> Summary {
    let mut summary = Summary::default();
    summary.runs = seeds;
    let mut victories = 0;

    for seed in 1..=seeds {
        let mut game = Game::with_seed_and_initial_variance(seed as u64 * 1_003, variance);
        let start_share = game.market_share();
        let start_cash = game.player.cash;
        let start_debt = game.player.debt;
        let start_customers = game.player.customers;
        let start_reliability = game.player.reliability;
        let start_rate = game.player.rate_cents;
        let start_headroom = game.player.capacity_headroom(&game.market);
        summary.start_share += start_share;
        summary.start_cash += start_cash;
        summary.start_debt += start_debt;
        summary.start_customers += start_customers;
        summary.start_reliability += start_reliability;
        summary.start_rate += start_rate;
        summary.start_headroom += start_headroom;
        summary.start_share_range.observe(start_share);
        summary.start_cash_range.observe(start_cash);
        summary.start_debt_range.observe(start_debt);
        summary.start_customers_range.observe(start_customers);
        summary.start_reliability_range.observe(start_reliability);
        summary.start_rate_range.observe(start_rate);
        summary.start_headroom_range.observe(start_headroom);

        while game.outcome.is_none() {
            scripted_policy(&mut game);
            let report = game.advance_quarter();
            if verbose {
                println!(
                    "{} seed {}: share {:.1}%, customers {:.0}, cash {}, debt {}, reliability {:.0}%, profit {}",
                    report.label,
                    seed,
                    game.market_share() * 100.0,
                    game.player.customers,
                    electrification::sim::money(game.player.cash),
                    electrification::sim::money(game.player.debt),
                    game.player.reliability * 100.0,
                    electrification::sim::money(report.profit)
                );
            }
        }

        if matches!(
            game.outcome.as_ref().map(|outcome| &outcome.kind),
            Some(OutcomeKind::Victory)
        ) {
            victories += 1;
        } else if let Some(outcome) = &game.outcome {
            if outcome.headline.contains("Market Access") {
                summary.market_access_defeats += 1;
            } else if outcome.headline.contains("Receivership") {
                summary.receivership_defeats += 1;
            } else {
                summary.board_defeats += 1;
            }
        }

        summary.finish_share += game.market_share();
        summary.finish_cash += game.player.cash;
        summary.finish_debt += game.player.debt;
        summary.finish_customers += game.player.customers;
        summary.finish_reliability += game.player.reliability;
    }

    summary.victories = victories;
    summary
}

fn print_summary(profile: Option<&str>, variance: InitialVariance, summary: &Summary) {
    if let Some(profile) = profile {
        println!();
        println!(
            "profile: {profile} | variance amplitude {:.2}",
            variance.amplitude
        );
    }
    println!("runs: {}", summary.runs);
    println!(
        "victories: {} ({:.1}%)",
        summary.victories,
        summary.victories as f64 / summary.runs as f64 * 100.0
    );
    if summary.victories < summary.runs {
        println!(
            "defeats: board {} | receivership {} | access {}",
            summary.board_defeats, summary.receivership_defeats, summary.market_access_defeats
        );
    }
    println!(
        "avg start: share {:.1}%, customers {:.0}, cash {}, debt {}, reliability {:.0}%, rate {:.1}c, headroom {:.0}",
        summary.start_share / summary.runs as f64 * 100.0,
        summary.start_customers / summary.runs as f64,
        electrification::sim::money(summary.start_cash / summary.runs as f64),
        electrification::sim::money(summary.start_debt / summary.runs as f64),
        summary.start_reliability / summary.runs as f64 * 100.0,
        summary.start_rate / summary.runs as f64,
        summary.start_headroom / summary.runs as f64
    );
    println!(
        "start ranges: share {:.1}-{:.1}%, customers {:.0}-{:.0}, cash {}-{}, debt {}-{}, reliability {:.0}-{:.0}%, rate {:.1}-{:.1}c, headroom {:.0}-{:.0}",
        summary.start_share_range.min * 100.0,
        summary.start_share_range.max * 100.0,
        summary.start_customers_range.min,
        summary.start_customers_range.max,
        electrification::sim::money(summary.start_cash_range.min),
        electrification::sim::money(summary.start_cash_range.max),
        electrification::sim::money(summary.start_debt_range.min),
        electrification::sim::money(summary.start_debt_range.max),
        summary.start_reliability_range.min * 100.0,
        summary.start_reliability_range.max * 100.0,
        summary.start_rate_range.min,
        summary.start_rate_range.max,
        summary.start_headroom_range.min,
        summary.start_headroom_range.max
    );
    println!(
        "avg finish: share {:.1}%, customers {:.0}, cash {}, debt {}, reliability {:.0}%",
        summary.finish_share / summary.runs as f64 * 100.0,
        summary.finish_customers / summary.runs as f64,
        electrification::sim::money(summary.finish_cash / summary.runs as f64),
        electrification::sim::money(summary.finish_debt / summary.runs as f64),
        summary.finish_reliability / summary.runs as f64 * 100.0
    );
}

#[derive(Default)]
struct Summary {
    runs: u32,
    victories: u32,
    board_defeats: u32,
    receivership_defeats: u32,
    market_access_defeats: u32,
    start_share: f64,
    start_cash: f64,
    start_debt: f64,
    start_customers: f64,
    start_reliability: f64,
    start_rate: f64,
    start_headroom: f64,
    start_share_range: Range,
    start_cash_range: Range,
    start_debt_range: Range,
    start_customers_range: Range,
    start_reliability_range: Range,
    start_rate_range: Range,
    start_headroom_range: Range,
    finish_share: f64,
    finish_cash: f64,
    finish_debt: f64,
    finish_customers: f64,
    finish_reliability: f64,
}

struct Range {
    min: f64,
    max: f64,
}

impl Range {
    fn observe(&mut self, value: f64) {
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }
}

impl Default for Range {
    fn default() -> Self {
        Self {
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
        }
    }
}

struct Options {
    seeds: u32,
    verbose: bool,
    variance: Option<f64>,
    sweep_starts: bool,
}

fn parse_options() -> Options {
    let mut args = std::env::args().skip(1);
    let mut seeds = 25;
    let mut verbose = false;
    let mut variance = None;
    let mut sweep_starts = false;
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
    }
}

fn scripted_policy(game: &mut Game) {
    if let Some((index, price)) = cheapest_competitor(game) {
        let reserve = if game.quarter < 4 { 5_000.0 } else { 10_000.0 };
        if game.player.cash < price + reserve && game.player.debt_to_assets() < 0.66 {
            let _ = game.apply_decision(Decision::Borrow {
                amount: (price + reserve - game.player.cash).clamp(8_000.0, 42_000.0),
            });
        }

        if game.player.cash < price + reserve && game.player.debt_to_assets() >= 0.60 {
            let _ = game.apply_decision(Decision::IssueStock {
                amount: (price + reserve - game.player.cash).clamp(12_000.0, 52_000.0),
            });
        }

        if game.player.cash > price + reserve && game.player.debt_to_assets() < 0.98 {
            let _ = game.apply_decision(Decision::Acquire {
                competitor_index: index,
            });
        }
    }

    if game.player.cash < 12_000.0 && game.player.debt_to_assets() < 0.72 {
        let _ = game.apply_decision(Decision::Borrow { amount: 20_000.0 });
    }

    if game.player.cash < 10_000.0 {
        let _ = game.apply_decision(Decision::IssueStock { amount: 24_000.0 });
    }

    let generation_headroom = game.player.generation_capacity_mwh
        - game.player.customers * game.market.avg_mwh_per_customer;
    if generation_headroom < 70.0 && game.player.cash > 24_000.0 {
        let _ = game.apply_decision(Decision::BuildGeneration {
            capacity_mwh: GENERATION_PROJECT_CAPACITY_MWH,
        });
    }

    if game.player.capacity_headroom(&game.market) < 95.0 && game.player.cash > 14_000.0 {
        let _ = game.apply_decision(Decision::BuildDistribution {
            customer_capacity: DISTRIBUTION_PROJECT_CAPACITY,
        });
    }

    if game.quarter % 3 == 0 && game.player.cash > 8_000.0 {
        let _ = game.apply_decision(Decision::Marketing { spend: 5_500.0 });
    }

    if game.market_share() < 0.34 && game.player.rate_cents > 9.4 {
        let _ = game.apply_decision(Decision::AdjustRate { delta_cents: -0.5 });
    }

    if game.player.reliability < 0.80 && game.player.cash > 7_500.0 {
        let spend = (5_500.0 + game.player.asset_base * 0.018).clamp(7_000.0, 20_000.0);
        let _ = game.apply_decision(Decision::Maintenance { spend });
    }

    if let Some((index, price)) = cheapest_competitor(game) {
        let can_absorb_debt = game.player.debt_to_assets() < 0.82;
        if game.player.cash > price + 10_000.0 && can_absorb_debt {
            let _ = game.apply_decision(Decision::Acquire {
                competitor_index: index,
            });
        }
    }
}

fn cheapest_competitor(game: &Game) -> Option<(usize, f64)> {
    (0..game.competitors.len())
        .map(|index| (index, game.acquisition_price(index)))
        .min_by(|left, right| left.1.total_cmp(&right.1))
}
