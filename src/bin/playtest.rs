use electrification::{
    DISTRIBUTION_PROJECT_CAPACITY, Decision, GENERATION_PROJECT_CAPACITY_MWH, Game, OutcomeKind,
};

fn main() {
    let options = parse_options();
    let seeds = options.seeds;
    let mut victories = 0;
    let mut total_share = 0.0;
    let mut total_cash = 0.0;
    let mut total_debt = 0.0;
    let mut total_customers = 0.0;

    for seed in 1..=seeds {
        let mut game = Game::with_seed(seed as u64 * 1_003);
        while game.outcome.is_none() {
            scripted_policy(&mut game);
            let report = game.advance_quarter();
            if options.verbose {
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
        }

        total_share += game.market_share();
        total_cash += game.player.cash;
        total_debt += game.player.debt;
        total_customers += game.player.customers;
    }

    println!("runs: {seeds}");
    println!(
        "victories: {victories} ({:.0}%)",
        victories as f64 / seeds as f64 * 100.0
    );
    println!(
        "avg market share: {:.1}%",
        total_share / seeds as f64 * 100.0
    );
    println!("avg customers: {:.0}", total_customers / seeds as f64);
    println!(
        "avg cash: {}",
        electrification::sim::money(total_cash / seeds as f64)
    );
    println!(
        "avg debt: {}",
        electrification::sim::money(total_debt / seeds as f64)
    );
}

struct Options {
    seeds: u32,
    verbose: bool,
}

fn parse_options() -> Options {
    let mut args = std::env::args().skip(1);
    let mut seeds = 25;
    let mut verbose = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--seeds" => {
                if let Some(value) = args.next().and_then(|raw| raw.parse::<u32>().ok()) {
                    seeds = value.max(1);
                }
            }
            "--verbose" => verbose = true,
            _ => {}
        }
    }
    Options { seeds, verbose }
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
        let _ = game.apply_decision(Decision::Maintenance { spend: 7_000.0 });
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
