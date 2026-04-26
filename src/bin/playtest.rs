use electrification::{
    DISTRIBUTION_PROJECT_CAPACITY, Decision, GENERATION_PROJECT_CAPACITY_MWH, Game,
    InitialVariance, OutcomeKind, distribution_project_cost, generation_project_cost,
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

fn run_batch(seeds: u32, verbose: bool, variance: InitialVariance, strategy: Strategy) -> Summary {
    let mut summary = Summary {
        runs: seeds,
        ..Summary::default()
    };

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
            run_policy(strategy, &mut game);
            let report = game.advance_quarter();
            if verbose {
                println!(
                    "{} {} seed {}: share {:.1}%, customers {:.0}, cash {}, debt {}, reliability {:.0}%, profit {}",
                    report.label,
                    strategy.name(),
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
            summary.victories += 1;
        } else if let Some(outcome) = &game.outcome {
            summary.record_defeat(&outcome.headline, game.quarter);
        }

        let finish_share = game.market_share();
        summary.finish_share += finish_share;
        summary.finish_cash += game.player.cash;
        summary.finish_debt += game.player.debt;
        summary.finish_customers += game.player.customers;
        summary.finish_reliability += game.player.reliability;
        summary.finish_share_range.observe(finish_share);
        summary.finish_share_histogram.observe(finish_share);
    }

    summary
}

fn print_summary(
    profile: Option<&str>,
    strategy: Strategy,
    variance: InitialVariance,
    summary: &Summary,
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
        summary.victories as f64 / runs * 100.0
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
        electrification::sim::money(summary.start_cash / runs),
        electrification::sim::money(summary.start_debt / runs),
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
        "avg finish: share {:.1}% (range {:.1}-{:.1}%), customers {:.0}, cash {}, debt {}, reliability {:.0}%",
        summary.finish_share / runs * 100.0,
        summary.finish_share_range.min * 100.0,
        summary.finish_share_range.max * 100.0,
        summary.finish_customers / runs,
        electrification::sim::money(summary.finish_cash / runs),
        electrification::sim::money(summary.finish_debt / runs),
        summary.finish_reliability / runs * 100.0
    );
    println!(
        "finish share distribution: {}",
        summary.finish_share_histogram.render()
    );
}

fn avg_quarter_suffix(count: u32, quarter_sum: f64) -> String {
    if count == 0 {
        String::new()
    } else {
        format!(" (avg Q{:.1})", quarter_sum / count as f64)
    }
}

#[derive(Default)]
struct Summary {
    runs: u32,
    victories: u32,
    board_defeats: u32,
    receivership_defeats: u32,
    market_access_defeats: u32,
    board_defeat_quarters: f64,
    receivership_defeat_quarters: f64,
    market_access_defeat_quarters: f64,
    defeat_quarters: f64,
    defeat_quarter_range: Range,
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
    finish_share_range: Range,
    finish_share_histogram: ShareHistogram,
    finish_cash: f64,
    finish_debt: f64,
    finish_customers: f64,
    finish_reliability: f64,
}

impl Summary {
    fn defeats(&self) -> u32 {
        self.board_defeats + self.receivership_defeats + self.market_access_defeats
    }

    fn record_defeat(&mut self, headline: &str, quarter: u32) {
        let quarter = quarter as f64;
        self.defeat_quarters += quarter;
        self.defeat_quarter_range.observe(quarter);
        if headline.contains("Market Access") {
            self.market_access_defeats += 1;
            self.market_access_defeat_quarters += quarter;
        } else if headline.contains("Receivership") {
            self.receivership_defeats += 1;
            self.receivership_defeat_quarters += quarter;
        } else {
            self.board_defeats += 1;
            self.board_defeat_quarters += quarter;
        }
    }
}

#[derive(Default)]
struct ShareHistogram {
    bins: [u32; 7],
}

impl ShareHistogram {
    fn observe(&mut self, share: f64) {
        let pct = share * 100.0;
        let index = if pct < 35.0 {
            0
        } else if pct < 40.0 {
            1
        } else if pct < 45.0 {
            2
        } else if pct < 50.0 {
            3
        } else if pct < 55.0 {
            4
        } else if pct < 60.0 {
            5
        } else {
            6
        };
        self.bins[index] += 1;
    }

    fn render(&self) -> String {
        let labels = [
            "<35%", "35-40%", "40-45%", "45-50%", "50-55%", "55-60%", "60%+",
        ];
        let mut parts = Vec::with_capacity(labels.len());
        for (label, count) in labels.iter().zip(self.bins.iter()) {
            parts.push(format!("{label}: {count}"));
        }
        parts.join(" | ")
    }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Strategy {
    Naive,
    Organic,
    Balanced,
    Mna,
}

impl Strategy {
    fn all() -> [Strategy; 4] {
        [
            Strategy::Naive,
            Strategy::Organic,
            Strategy::Balanced,
            Strategy::Mna,
        ]
    }

    fn name(self) -> &'static str {
        match self {
            Strategy::Naive => "naive",
            Strategy::Organic => "organic",
            Strategy::Balanced => "balanced",
            Strategy::Mna => "mna",
        }
    }
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
        "mna" | "ma" | "m&a" | "acquire" | "acquisition" | "expert" => {
            Some(StrategySelection::One(Strategy::Mna))
        }
        _ => None,
    }
}

fn run_policy(strategy: Strategy, game: &mut Game) {
    match strategy {
        Strategy::Naive => naive_policy(game),
        Strategy::Organic => organic_policy(game),
        Strategy::Balanced => balanced_policy(game),
        Strategy::Mna => mna_policy(game),
    }
}

fn naive_policy(game: &mut Game) {
    if game.player.cash < 6_000.0 && game.player.debt_to_assets() < 0.55 {
        borrow_up_to(game, 12_000.0, 0.55);
    }

    if game.player.cash < 5_000.0 {
        let _ = game.apply_decision(Decision::IssueStock { amount: 10_000.0 });
    }

    if game.player.capacity_headroom(&game.market) < 35.0 {
        let cost = distribution_project_cost(DISTRIBUTION_PROJECT_CAPACITY);
        if game.player.cash > cost + 5_000.0 {
            let _ = game.apply_decision(Decision::BuildDistribution {
                customer_capacity: DISTRIBUTION_PROJECT_CAPACITY,
            });
        }
    }

    if game.player.firm_generation_reserve_mwh(&game.market) < 25.0 {
        let cost = generation_project_cost(GENERATION_PROJECT_CAPACITY_MWH);
        if game.player.cash > cost + 6_000.0 {
            let _ = game.apply_decision(Decision::BuildGeneration {
                capacity_mwh: GENERATION_PROJECT_CAPACITY_MWH,
            });
        }
    }

    if game.quarter % 4 == 0 && game.player.cash > 7_000.0 {
        let _ = game.apply_decision(Decision::Marketing { spend: 2_500.0 });
    }

    if game.player.reliability < 0.74 && game.player.cash > 6_500.0 {
        let _ = game.apply_decision(Decision::Maintenance { spend: 3_500.0 });
    }

    if game.market_share() < 0.27 && game.player.rate_cents > 10.0 {
        let _ = game.apply_decision(Decision::AdjustRate { delta_cents: -0.25 });
    }
}

fn organic_policy(game: &mut Game) {
    run_organic_program(
        game,
        OrganicPlan {
            target_rate_cents: 8.2,
            marketing_spend: 10_000.0,
            marketing_interval: 1,
            reliability_floor: 0.80,
            headroom_target: 320.0,
            firm_reserve_target_mwh: 110.0,
            distribution_project_customers: 700.0,
            generation_project_mwh: 500.0,
            cash_reserve: 8_000.0,
            max_debt_to_assets: 0.80,
        },
    );
}

fn balanced_policy(game: &mut Game) {
    let acquisition_share_cap = if game.quarter < 8 { 0.32 } else { 0.38 };
    if game.market_share() < acquisition_share_cap {
        let _ = try_acquire_cheapest(game, 12_000.0, 0.72, 0.68);
    }

    run_organic_program(
        game,
        OrganicPlan {
            target_rate_cents: 9.0,
            marketing_spend: 6_000.0,
            marketing_interval: 2,
            reliability_floor: 0.81,
            headroom_target: 180.0,
            firm_reserve_target_mwh: 75.0,
            distribution_project_customers: 500.0,
            generation_project_mwh: 350.0,
            cash_reserve: 10_000.0,
            max_debt_to_assets: 0.72,
        },
    );

    if game.market_share() < 0.38 {
        let _ = try_acquire_cheapest(game, 14_000.0, 0.74, 0.72);
    }
}

fn mna_policy(game: &mut Game) {
    if let Some((index, price)) = cheapest_competitor(game) {
        let reserve = if game.quarter < 4 { 5_000.0 } else { 10_000.0 };
        let diligence_cost = if game.has_diligence(index) {
            0.0
        } else {
            game.diligence_cost(index).unwrap_or(0.0)
        };
        if game.player.cash < price + reserve + diligence_cost
            && game.player.debt_to_assets() < 0.66
        {
            let _ = game.apply_decision(Decision::Borrow {
                amount: (price + reserve + diligence_cost - game.player.cash)
                    .clamp(8_000.0, 42_000.0),
            });
        }

        if game.player.cash < price + reserve + diligence_cost
            && game.player.debt_to_assets() >= 0.60
        {
            let _ = game.apply_decision(Decision::IssueStock {
                amount: (price + reserve + diligence_cost - game.player.cash)
                    .clamp(12_000.0, 52_000.0),
            });
        }

        if !game.has_diligence(index) && game.player.cash > diligence_cost + reserve * 0.25 {
            let _ = game.apply_decision(Decision::Diligence {
                competitor_index: index,
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
        if !game.has_diligence(index) {
            if let Some(cost) = game.diligence_cost(index) {
                if game.player.cash > cost + 5_000.0 {
                    let _ = game.apply_decision(Decision::Diligence {
                        competitor_index: index,
                    });
                }
            }
        }
        if game.player.cash > price + 10_000.0 && can_absorb_debt {
            let _ = game.apply_decision(Decision::Acquire {
                competitor_index: index,
            });
        }
    }
}

#[derive(Clone, Copy)]
struct OrganicPlan {
    target_rate_cents: f64,
    marketing_spend: f64,
    marketing_interval: u32,
    reliability_floor: f64,
    headroom_target: f64,
    firm_reserve_target_mwh: f64,
    distribution_project_customers: f64,
    generation_project_mwh: f64,
    cash_reserve: f64,
    max_debt_to_assets: f64,
}

fn run_organic_program(game: &mut Game, plan: OrganicPlan) {
    cut_rate_toward(game, plan.target_rate_cents);
    build_distribution_to_headroom(game, plan);
    build_generation_to_reserve(game, plan);

    if plan.marketing_interval > 0 && game.quarter % plan.marketing_interval == 0 {
        let marketing_buffer = plan.cash_reserve * 0.75;
        finance_to_cash(
            game,
            plan.marketing_spend + marketing_buffer,
            plan.max_debt_to_assets,
        );
        if game.player.cash > plan.marketing_spend + project_cash_buffer(plan) {
            let _ = game.apply_decision(Decision::Marketing {
                spend: plan.marketing_spend,
            });
        }
    }

    maintain_reliability(game, plan);
    if game.player.cash < plan.cash_reserve {
        finance_to_cash(game, plan.cash_reserve + 10_000.0, plan.max_debt_to_assets);
    }
}

fn build_generation_to_reserve(game: &mut Game, plan: OrganicPlan) {
    let reserve = game.player.firm_generation_reserve_mwh(&game.market);
    if reserve >= plan.firm_reserve_target_mwh {
        return;
    }

    let project_size = (plan.generation_project_mwh
        + (plan.firm_reserve_target_mwh - reserve).max(0.0) * 0.35)
        .clamp(250.0, 900.0);
    let cost = generation_project_cost(project_size);
    finance_to_cash(game, cost + plan.cash_reserve, plan.max_debt_to_assets);
    if game.player.cash > cost + project_cash_buffer(plan) {
        let _ = game.apply_decision(Decision::BuildGeneration {
            capacity_mwh: project_size,
        });
    }
}

fn build_distribution_to_headroom(game: &mut Game, plan: OrganicPlan) {
    let headroom = game.player.capacity_headroom(&game.market);
    if headroom >= plan.headroom_target {
        return;
    }

    let project_size = (plan.distribution_project_customers
        + (plan.headroom_target - headroom).max(0.0) * 0.45)
        .clamp(300.0, 1_200.0);
    let cost = distribution_project_cost(project_size);
    finance_to_cash(game, cost + plan.cash_reserve, plan.max_debt_to_assets);
    if game.player.cash > cost + project_cash_buffer(plan) {
        let _ = game.apply_decision(Decision::BuildDistribution {
            customer_capacity: project_size,
        });
    }
}

fn maintain_reliability(game: &mut Game, plan: OrganicPlan) {
    if game.player.reliability >= plan.reliability_floor {
        return;
    }

    let spend = (8_000.0 + game.player.asset_base * 0.012).clamp(8_000.0, 20_000.0);
    finance_to_cash(
        game,
        spend + plan.cash_reserve * 0.5,
        plan.max_debt_to_assets,
    );
    if game.player.cash > spend + plan.cash_reserve * 0.375 {
        let _ = game.apply_decision(Decision::Maintenance { spend });
    }
}

fn cut_rate_toward(game: &mut Game, target_rate_cents: f64) {
    while game.player.rate_cents > target_rate_cents + 0.05 {
        let cut = (game.player.rate_cents - target_rate_cents).min(0.5);
        let _ = game.apply_decision(Decision::AdjustRate { delta_cents: -cut });
    }
}

fn project_cash_buffer(plan: OrganicPlan) -> f64 {
    (plan.cash_reserve * 0.625).max(3_000.0)
}

fn try_acquire_cheapest(
    game: &mut Game,
    reserve: f64,
    max_debt_to_assets: f64,
    max_price_to_assets: f64,
) -> bool {
    if game.acquisition_cooldown > 0 || game.competitors.len() <= 1 {
        return false;
    }

    let Some((index, price)) = cheapest_competitor(game) else {
        return false;
    };

    if price > game.player.asset_base * max_price_to_assets {
        return false;
    }

    let diligence_cost = if game.has_diligence(index) {
        0.0
    } else {
        game.diligence_cost(index).unwrap_or(0.0)
    };
    finance_to_cash(game, price + reserve + diligence_cost, max_debt_to_assets);
    if !game.has_diligence(index) {
        if game.player.cash <= diligence_cost + reserve * 0.25 {
            return false;
        }
        if game
            .apply_decision(Decision::Diligence {
                competitor_index: index,
            })
            .is_err()
        {
            return false;
        }
    }
    if game.player.cash <= price + reserve
        || game.player.debt_to_assets() > max_debt_to_assets + 0.10
    {
        return false;
    }

    game.apply_decision(Decision::Acquire {
        competitor_index: index,
    })
    .is_ok()
}

fn finance_to_cash(game: &mut Game, target_cash: f64, max_debt_to_assets: f64) {
    if game.player.cash >= target_cash {
        return;
    }

    let need = target_cash - game.player.cash;
    borrow_up_to(game, need, max_debt_to_assets);

    if game.player.cash < target_cash {
        let amount = (target_cash - game.player.cash) * 1.15;
        if amount >= 1_000.0 {
            let _ = game.apply_decision(Decision::IssueStock { amount });
        }
    }
}

fn borrow_up_to(game: &mut Game, target_amount: f64, max_debt_to_assets: f64) {
    let leverage_room = (game.player.asset_base * max_debt_to_assets - game.player.debt).max(0.0);
    let amount = target_amount.min(game.borrowing_room()).min(leverage_room);
    if amount >= 1_000.0 {
        let _ = game.apply_decision(Decision::Borrow { amount });
    }
}

fn cheapest_competitor(game: &Game) -> Option<(usize, f64)> {
    (0..game.competitors.len())
        .map(|index| (index, game.acquisition_price(index)))
        .min_by(|left, right| left.1.total_cmp(&right.1))
}
