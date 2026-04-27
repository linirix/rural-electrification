use super::*;

#[test]
fn stock_issuance_raises_cash_and_dilutes_shares() {
    let mut game = Game::with_seed(1);
    let starting_cash = game.player.cash;
    let starting_shares = game.player.shares;
    let starting_price = game.player.stock_price;

    game.apply_decision(Decision::IssueStock { amount: 20_000.0 })
        .unwrap();

    assert!(game.player.cash > starting_cash);
    assert!(game.player.cash < starting_cash + 20_000.0);
    assert!(game.player.shares > starting_shares);
    assert!(game.player.stock_price < starting_price);
}

#[test]
fn fixed_initial_variance_preserves_baseline_start() {
    let game = Game::with_seed_and_initial_variance(1, InitialVariance::fixed());

    assert!((game.player.cash - 34_000.0).abs() < 0.01);
    assert!((game.player.debt - 20_000.0).abs() < 0.01);
    assert!((game.player.customers - 160.0).abs() < 0.01);
    assert!((game.player.reliability - 0.82).abs() < 0.0001);
    assert!((game.market.addressable_customers - 6_500.0).abs() < 0.01);
    assert!(
        (game.market.variable_cost_per_mwh - game.market.baseline_variable_cost_per_mwh).abs()
            < 0.0001
    );
}

#[test]
fn seeded_initial_variance_changes_starting_conditions() {
    let first = Game::with_seed_and_initial_variance(1, InitialVariance::default());
    let second = Game::with_seed_and_initial_variance(2, InitialVariance::default());

    assert_ne!(first.player.cash, second.player.cash);
    assert_ne!(first.player.customers, second.player.customers);
    assert_ne!(
        first.market.addressable_customers,
        second.market.addressable_customers
    );
}

#[test]
fn serialized_game_round_trip_preserves_next_quarter() {
    let mut original = Game::with_seed(88);
    original
        .apply_decision(Decision::Borrow { amount: 15_000.0 })
        .unwrap();
    original
        .apply_decision(Decision::Marketing { spend: 2_000.0 })
        .unwrap();
    original.advance_quarter();

    let json = serde_json::to_string(&original).unwrap();
    let mut loaded: Game = serde_json::from_str(&json).unwrap();

    let original_report = original.advance_quarter();
    let loaded_report = loaded.advance_quarter();

    assert_eq!(loaded.quarter, original.quarter);
    assert_eq!(loaded_report.label, original_report.label);
    assert_eq!(loaded_report.events, original_report.events);
    assert_eq!(loaded_report.attributions, original_report.attributions);
    assert!((loaded_report.profit - original_report.profit).abs() < 0.01);
    assert!((loaded_report.market_share - original_report.market_share).abs() < 0.000001);
    assert!((loaded.player.cash - original.player.cash).abs() < 0.01);
    assert!((loaded.player.customers - original.player.customers).abs() < 0.000001);
    assert_eq!(loaded.competitors.len(), original.competitors.len());
    for (loaded_competitor, original_competitor) in
        loaded.competitors.iter().zip(original.competitors.iter())
    {
        assert_eq!(loaded_competitor.name, original_competitor.name);
        assert!((loaded_competitor.customers - original_competitor.customers).abs() < 0.000001);
        assert!((loaded_competitor.cash - original_competitor.cash).abs() < 0.01);
    }
}

#[test]
fn opening_competitor_names_are_drawn_from_pool_without_duplicates() {
    let game = Game::with_seed(7);
    let names = game
        .competitors
        .iter()
        .map(|competitor| competitor.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(names.len(), 3);
    for name in &names {
        assert!(COMPETITOR_NAME_POOL.contains(name));
    }
    for (index, name) in names.iter().enumerate() {
        assert!(!names[..index].contains(name));
    }
}

#[test]
fn competitor_name_draw_avoids_active_names_until_pool_is_exhausted() {
    let mut rng = Rng::new(12);
    let used_names = COMPETITOR_NAME_POOL
        .iter()
        .take(COMPETITOR_NAME_POOL.len() - 1)
        .map(|name| name.to_string())
        .collect::<Vec<_>>();

    let name = draw_competitor_name(&mut rng, &used_names, 1);

    assert_eq!(name, COMPETITOR_NAME_POOL[COMPETITOR_NAME_POOL.len() - 1]);
}

#[test]
fn rival_merger_combines_two_largest_competitors() {
    let mut game = Game::with_seed(32);
    game.competitors[0].customers = 300.0;
    game.competitors[1].customers = 220.0;
    game.competitors[2].customers = 90.0;
    let starting_count = game.competitors.len();
    let mut events = Vec::new();

    assert!(game.merge_largest_rivals(&mut events));

    assert_eq!(game.competitors.len(), starting_count - 1);
    assert!(
        game.competitors
            .iter()
            .any(|competitor| competitor.customers > 500.0)
    );
    assert!(
        events
            .iter()
            .any(|event| event.contains("creating a stronger rival"))
    );
}

#[test]
fn rival_merger_respects_low_rate_environment_without_old_floor() {
    let mut game = Game::with_seed(33);
    game.market.standard_rate_cents = 5.2;
    game.market.variable_cost_per_mwh = 12.0;
    game.market.baseline_variable_cost_per_mwh = 12.0;
    game.macro_state.annual_base_rate = 0.035;
    game.macro_state.credit_spread = 0.008;
    game.competitors[0].customers = 420.0;
    game.competitors[0].rate_cents = 5.4;
    game.competitors[0].asset_base = 65_000.0;
    game.competitors[1].customers = 360.0;
    game.competitors[1].rate_cents = 5.2;
    game.competitors[1].asset_base = 60_000.0;
    game.competitors[2].customers = 90.0;
    let mut events = Vec::new();

    assert!(game.merge_largest_rivals(&mut events));

    let merged_rate = game
        .competitors
        .iter()
        .find(|competitor| competitor.customers > 700.0)
        .map(|competitor| competitor.rate_cents)
        .expect("expected merged competitor");
    assert!(
        merged_rate < 8.4,
        "merged rival should not inherit the old 8.4c floor; rate was {merged_rate}"
    );
}

#[test]
fn default_initial_variance_stays_within_playable_bounds() {
    for seed in 1..=250 {
        let game = Game::with_seed_and_initial_variance(seed, InitialVariance::default());

        assert!(game.player.cash >= 27_000.0 && game.player.cash <= 41_500.0);
        assert!(game.player.debt >= 16_000.0 && game.player.debt <= 24_500.0);
        assert!(game.player.customers >= 140.0 && game.player.customers <= 180.0);
        assert!(game.player.reliability >= 0.77 && game.player.reliability <= 0.87);
        assert!(game.player.capacity_headroom(&game.market) >= 45.0);
    }
}

#[test]
fn simulation_state_stays_sane_across_extended_policy_runs() {
    for seed in 1..=80 {
        let mut game = Game::with_seed(seed);
        game.campaign_quarters = 80;

        for _ in 0..40 {
            run_sanity_policy(&mut game);
            let report = game.advance_quarter();
            assert_report_sane(&report);
            assert_game_sane(&game);

            if game
                .outcome
                .as_ref()
                .is_some_and(|outcome| !outcome.can_continue)
            {
                break;
            }
        }
    }
}

#[test]
fn nonfinite_and_negative_decision_inputs_are_rejected_without_mutation() {
    let decisions = [
        Decision::BuildGeneration {
            capacity_mwh: f64::NAN,
        },
        Decision::BuildDistribution {
            customer_capacity: -50.0,
        },
        Decision::Marketing {
            spend: f64::NEG_INFINITY,
        },
        Decision::Maintenance { spend: -100.0 },
        Decision::AdjustRate {
            delta_cents: f64::NAN,
        },
    ];

    for decision in decisions {
        let mut game = Game::with_seed(12);
        let starting_cash = game.player.cash;
        let starting_asset_base = game.player.asset_base;
        let starting_rate = game.player.rate_cents;
        let starting_projects = game.pending_projects.len();

        assert!(game.apply_decision(decision).is_err());
        assert_eq!(game.player.cash, starting_cash);
        assert_eq!(game.player.asset_base, starting_asset_base);
        assert_eq!(game.player.rate_cents, starting_rate);
        assert_eq!(game.pending_projects.len(), starting_projects);
        assert_game_sane(&game);
    }
}

fn run_sanity_policy(game: &mut Game) {
    if game.outcome.is_some() {
        return;
    }

    if game.player.cash < 7_000.0 && game.borrowing_room() > 1_000.0 {
        let amount = game.borrowing_room().min(12_000.0);
        let _ = game.apply_decision(Decision::Borrow { amount });
    }

    if game.player.reliability < 0.78 && game.player.cash > 6_500.0 {
        let _ = game.apply_decision(Decision::Maintenance { spend: 4_500.0 });
    }

    if game.player.firm_generation_reserve_mwh(&game.market) < 35.0 {
        let cost = generation_project_cost(GENERATION_PROJECT_CAPACITY_MWH);
        if game.player.cash > cost + 5_000.0 {
            let _ = game.apply_decision(Decision::BuildGeneration {
                capacity_mwh: GENERATION_PROJECT_CAPACITY_MWH,
            });
        }
    }

    if game.player.capacity_headroom(&game.market) < 90.0 {
        let cost = distribution_project_cost(DISTRIBUTION_PROJECT_CAPACITY);
        if game.player.cash > cost + 5_000.0 {
            let _ = game.apply_decision(Decision::BuildDistribution {
                customer_capacity: DISTRIBUTION_PROJECT_CAPACITY,
            });
        }
    }

    if game.quarter.is_multiple_of(3) && game.player.cash > 7_000.0 {
        let _ = game.apply_decision(Decision::Marketing { spend: 3_000.0 });
    }

    if game.market_share() < 0.35 && game.player.rate_cents > game.market.standard_rate_cents {
        let _ = game.apply_decision(Decision::AdjustRate { delta_cents: -0.25 });
    }
}

fn assert_report_sane(report: &QuarterReport) {
    assert!(report.revenue.is_finite() && report.revenue >= 0.0);
    assert!(report.operating_cost.is_finite() && report.operating_cost >= 0.0);
    assert!(report.interest.is_finite() && report.interest >= 0.0);
    assert!(report.profit.is_finite());
    assert!(report.new_customers.is_finite() && report.new_customers >= 0.0);
    assert!(report.lost_customers.is_finite() && report.lost_customers >= 0.0);
    assert!(report.lost_customer_rate.is_finite() && report.lost_customer_rate >= 0.0);
    assert!(report.market_share.is_finite() && (0.0..=1.0).contains(&report.market_share));
    assert!(
        report.prior_market_share.is_finite() && (0.0..=1.0).contains(&report.prior_market_share)
    );
}

fn assert_game_sane(game: &Game) {
    assert!(
        game.market.addressable_customers.is_finite() && game.market.addressable_customers > 0.0
    );
    assert!(game.market.electrification.is_finite());
    assert!((0.05..=0.68).contains(&game.market.electrification));
    assert!(game.market.avg_mwh_per_customer.is_finite() && game.market.avg_mwh_per_customer > 0.0);
    assert!(
        game.market.variable_cost_per_mwh.is_finite() && game.market.variable_cost_per_mwh > 0.0
    );
    assert!(game.macro_state.annual_base_rate.is_finite());
    assert!(game.macro_state.credit_spread.is_finite());
    assert!(game.macro_state.demand_index.is_finite());
    assert!(game.macro_state.cost_pressure.is_finite());
    assert!(game.market_share().is_finite() && (0.0..=1.0).contains(&game.market_share()));
    assert_utility_sane(&game.player, true);
    for competitor in &game.competitors {
        assert_utility_sane(competitor, false);
    }
}

fn assert_utility_sane(utility: &Utility, has_public_stock: bool) {
    assert!(utility.cash.is_finite() && utility.cash >= 0.0);
    assert!(utility.debt.is_finite() && utility.debt >= 0.0);
    assert!(utility.customers.is_finite() && utility.customers >= 0.0);
    assert!(utility.generation_capacity_mwh.is_finite() && utility.generation_capacity_mwh > 0.0);
    assert!(utility.distribution_capacity.is_finite() && utility.distribution_capacity > 0.0);
    assert!(utility.rate_cents.is_finite() && utility.rate_cents > 0.0);
    assert!(utility.reputation.is_finite() && (0.0..=100.0).contains(&utility.reputation));
    assert!(utility.reliability.is_finite() && (0.35..=0.98).contains(&utility.reliability));
    assert!(utility.marketing_momentum.is_finite() && utility.marketing_momentum >= 0.0);
    assert!(utility.asset_base.is_finite() && utility.asset_base > 0.0);
    assert!(utility.last_quarter_customers.is_finite() && utility.last_quarter_customers >= 0.0);
    if has_public_stock {
        assert!(utility.shares.is_finite() && utility.shares > 0.0);
        assert!(utility.stock_price.is_finite() && utility.stock_price >= MIN_STOCK_PRICE);
    }
}

fn eligible_adjacent_expansion_game() -> Game {
    let mut game = Game::with_seed(54);
    game.quarter = 8;
    game.player.cash = 260_000.0;
    game.player.debt = 30_000.0;
    game.player.asset_base = 140_000.0;
    game.player.customers = 780.0;
    game.player.generation_capacity_mwh = 260.0;
    game.player.distribution_capacity = 1_050.0;
    game.player.reliability = 0.88;
    game
}

#[test]
fn stock_issuance_has_no_fixed_proceeds_cap() {
    let mut game = Game::with_seed(14);
    let starting_cash = game.player.cash;

    game.apply_decision(Decision::IssueStock { amount: 120_000.0 })
        .unwrap();

    assert!(game.player.cash > starting_cash + 90_000.0);
    assert!(game.player.shares > 2_000.0);
}

#[test]
fn large_stock_issue_does_not_add_full_cash_to_market_cap() {
    let mut game = Game::with_seed(14);
    let starting_cash = game.player.cash;
    let starting_market_cap = game.player.market_cap();

    game.apply_decision(Decision::IssueStock { amount: 120_000.0 })
        .unwrap();

    let cash_gain = game.player.cash - starting_cash;
    let market_cap_gain = game.player.market_cap() - starting_market_cap;

    assert!(cash_gain > 90_000.0);
    assert!(
        market_cap_gain < cash_gain,
        "market cap gain {market_cap_gain} should stay below net cash gain {cash_gain} because the issue reprices pre-money equity"
    );
}

#[test]
fn stock_issuance_uses_pre_and_post_money_valuation() {
    let mut game = Game::with_seed(14);
    let starting_cash = game.player.cash;
    let starting_shares = game.player.shares;
    let amount = 45_000.0;

    game.apply_decision(Decision::IssueStock { amount })
        .unwrap();

    let issued_shares = game.player.shares - starting_shares;
    let issue_price = amount / issued_shares;
    let implied_pre_money = issue_price * starting_shares;
    let net_proceeds = game.player.cash - starting_cash;
    let expected_post_money = implied_pre_money + net_proceeds;

    assert!(issued_shares > 0.0);
    assert!(
        (game.player.market_cap() - expected_post_money).abs() < 0.01,
        "post-money market cap should equal implied pre-money plus net proceeds: actual {}, expected {}",
        game.player.market_cap(),
        expected_post_money
    );
}

#[test]
fn repeated_large_stock_issues_hit_market_fatigue() {
    let mut game = Game::with_seed(14);

    game.apply_decision(Decision::IssueStock { amount: 120_000.0 })
        .unwrap();
    game.apply_decision(Decision::IssueStock { amount: 120_000.0 })
        .unwrap();

    let error = game
        .apply_decision(Decision::IssueStock { amount: 120_000.0 })
        .unwrap_err();

    assert!(game.equity_market_fatigue > 0.0);
    assert!(error.contains("cannot absorb"));
}

#[test]
fn second_issue_in_same_quarter_gets_worse_pricing_than_the_first() {
    let mut game = Game::with_seed(220);

    let cash_before_first = game.player.cash;
    game.apply_decision(Decision::IssueStock { amount: 25_000.0 })
        .unwrap();
    let cash_after_first = game.player.cash;
    let first_yield = (cash_after_first - cash_before_first) / 25_000.0;
    let fatigue_after_first = game.equity_market_fatigue;

    game.apply_decision(Decision::IssueStock { amount: 25_000.0 })
        .unwrap();
    let cash_after_second = game.player.cash;
    let second_yield = (cash_after_second - cash_after_first) / 25_000.0;

    assert!(
        fatigue_after_first > 0.0,
        "first issue should record equity-market fatigue: {fatigue_after_first}"
    );
    assert!(
        game.equity_market_fatigue > fatigue_after_first,
        "second same-quarter issue should add more fatigue: {} -> {}",
        fatigue_after_first,
        game.equity_market_fatigue
    );
    assert!(
        second_yield + 0.005 < first_yield,
        "second issue should net less cash per dollar than the first: first {first_yield:.4}, second {second_yield:.4}"
    );
}

#[test]
fn issues_spaced_across_quarters_recover_better_yield_than_back_to_back() {
    let mut spaced = Game::with_seed(221);
    let mut sequential = Game::with_seed(221);

    let spaced_starting_cash = spaced.player.cash;
    let sequential_starting_cash = sequential.player.cash;

    sequential
        .apply_decision(Decision::IssueStock { amount: 25_000.0 })
        .unwrap();
    sequential
        .apply_decision(Decision::IssueStock { amount: 25_000.0 })
        .unwrap();

    spaced
        .apply_decision(Decision::IssueStock { amount: 25_000.0 })
        .unwrap();
    spaced.advance_quarter();
    spaced
        .apply_decision(Decision::IssueStock { amount: 25_000.0 })
        .unwrap();

    let sequential_proceeds = sequential.player.cash - sequential_starting_cash;
    let spaced_proceeds = spaced.player.cash - spaced_starting_cash;

    // The spaced game advanced an extra quarter (operating profit + macro shift), so we
    // compare residual fatigue to verify the decay path rather than raw cash deltas.
    assert!(
        spaced.equity_market_fatigue < sequential.equity_market_fatigue,
        "spacing issues across a quarter should leave less residual fatigue: spaced {:.3}, sequential {:.3}",
        spaced.equity_market_fatigue,
        sequential.equity_market_fatigue
    );
    // Sanity: both issuances actually netted real cash.
    assert!(sequential_proceeds > 0.0);
    assert!(spaced_proceeds > 0.0);
}

#[test]
fn equity_fatigue_decays_at_documented_per_quarter_rate() {
    let mut game = Game::with_seed(222);
    game.equity_market_fatigue = 1.0;

    let before = game.equity_market_fatigue;
    game.advance_quarter();
    let after_one = game.equity_market_fatigue;

    // The decay multiplier on each advance_quarter is 0.55. Allow a small epsilon for any
    // additional fatigue added by board checkpoint or other quarter logic. From quarter 0
    // we are well before the Year 2 board checkpoint, so no extra fatigue should be added.
    assert!(
        (after_one - before * 0.55).abs() < 1e-6,
        "fatigue should decay by 0.55x per quarter pre-checkpoint: before {before}, after {after_one}"
    );

    // Across four quarters the decay should compound to roughly 0.55^4.
    let mut decay_game = Game::with_seed(223);
    decay_game.equity_market_fatigue = 1.0;
    for _ in 0..4 {
        decay_game.advance_quarter();
    }
    let expected = 0.55_f64.powi(4);
    assert!(
        (decay_game.equity_market_fatigue - expected).abs() < 1e-3,
        "four-quarter compound decay should match 0.55^4 = {:.4}, got {:.4}",
        expected,
        decay_game.equity_market_fatigue
    );
}

#[test]
fn excessive_dilution_yields_less_cash_per_dollar() {
    let mut small = Game::with_seed(101);
    let mut large = small.clone();

    let small_starting_cash = small.player.cash;
    let large_starting_cash = large.player.cash;

    small
        .apply_decision(Decision::IssueStock { amount: 20_000.0 })
        .unwrap();
    large
        .apply_decision(Decision::IssueStock { amount: 120_000.0 })
        .unwrap();

    let small_yield = (small.player.cash - small_starting_cash) / 20_000.0;
    let large_yield = (large.player.cash - large_starting_cash) / 120_000.0;

    assert!(
        small_yield > large_yield + 0.10,
        "small yield {small_yield} should beat large yield {large_yield}"
    );
}

#[test]
fn oversized_stock_issue_is_rejected_before_negative_proceeds() {
    let mut game = Game::with_seed(102);
    let starting_cash = game.player.cash;
    let starting_shares = game.player.shares;

    let error = game
        .apply_decision(Decision::IssueStock {
            amount: 1_000_000.0,
        })
        .unwrap_err();

    assert!(error.contains("cannot absorb"));
    assert!((game.player.cash - starting_cash).abs() < 0.01);
    assert!((game.player.shares - starting_shares).abs() < 0.01);
}

#[test]
fn stock_buyback_reduces_cash_and_share_count() {
    let mut game = Game::with_seed(10);
    let starting_cash = game.player.cash;
    let starting_shares = game.player.shares;
    let starting_debt = game.player.debt;

    game.apply_decision(Decision::BuyBackStock { amount: 10_000.0 })
        .unwrap();

    assert!(game.player.cash < starting_cash);
    assert!(game.player.shares < starting_shares);
    assert_eq!(game.player.debt, starting_debt);
    assert!(game.player.stock_price >= MIN_STOCK_PRICE);
}

#[test]
fn buybacks_lift_per_share_price_modestly() {
    let mut game = Game::with_seed(10);
    game.player.cash = 100_000.0;
    let starting_price = game.player.stock_price;
    let starting_market_cap = game.player.market_cap();
    let amount = starting_market_cap * 0.10;

    game.apply_decision(Decision::BuyBackStock { amount })
        .unwrap();

    assert!(
        game.player.stock_price > starting_price * 1.01,
        "10% market-cap buyback should modestly lift per-share price: {} -> {}",
        starting_price,
        game.player.stock_price
    );
    assert!(
        game.player.stock_price < starting_price * 1.08,
        "10% market-cap buyback should not mechanically pump the full tender premium: {} -> {}",
        starting_price,
        game.player.stock_price
    );
    assert!(
        game.player.market_cap() < starting_market_cap,
        "buybacks should not create total market-cap value from cash spend"
    );
}

#[test]
fn stock_buyback_reduces_equity_at_current_market_cap() {
    let mut game = Game::with_seed(10);
    game.player.cash = 100_000.0;
    let starting_cash = game.player.cash;
    let starting_market_cap = game.player.market_cap();
    let amount = game.player.market_cap() * 0.12;

    game.apply_decision(Decision::BuyBackStock { amount })
        .unwrap();

    let actual_spend = starting_cash - game.player.cash;
    let baseline_post_buyback_value = starting_market_cap - actual_spend;

    assert!(
        game.player.market_cap() >= baseline_post_buyback_value * 0.97,
        "post-buyback market cap should stay anchored to pre-buyback value less cash spent: actual {}, baseline {}",
        game.player.market_cap(),
        baseline_post_buyback_value
    );
    assert!(
        game.player.market_cap() <= baseline_post_buyback_value * 1.07,
        "buyback signal should not re-anchor the whole company at the tender premium: actual {}, baseline {}",
        game.player.market_cap(),
        baseline_post_buyback_value
    );
}

#[test]
fn issue_then_buyback_roundtrip_loses_value_to_discount_and_fees() {
    let mut game = Game::with_seed(10);
    game.player.cash = 100_000.0;
    let starting_cash = game.player.cash;
    let starting_market_cap = game.player.market_cap();
    let starting_shares = game.player.shares;

    game.apply_decision(Decision::IssueStock { amount: 20_000.0 })
        .unwrap();
    game.apply_decision(Decision::BuyBackStock { amount: 20_000.0 })
        .unwrap();

    assert!(game.player.cash < starting_cash);
    assert!(
        game.player.market_cap() < starting_market_cap * 1.02,
        "roundtrip should not manufacture market cap: {} -> {}",
        starting_market_cap,
        game.player.market_cap()
    );
    assert!(
        game.player.shares > starting_shares,
        "roundtrip should leave dilution when stock is issued at a discount and repurchased through the market"
    );
}

#[test]
fn three_back_to_back_buybacks_cannot_pump_price_above_threshold() {
    let mut game = Game::with_seed(1);
    game.player.cash = 200_000.0;
    let starting_price = game.player.stock_price;

    for _ in 0..3 {
        game.apply_decision(Decision::BuyBackStock { amount: 10_000.0 })
            .unwrap();
    }

    assert!(
        game.player.stock_price < starting_price * 1.20,
        "three buybacks pumped price too aggressively: {} -> {}",
        starting_price,
        game.player.stock_price
    );
}

#[test]
fn stock_buyback_cannot_exceed_cash_available() {
    let mut game = Game::with_seed(13);
    game.player.cash = 2_500.0;

    let error = game
        .apply_decision(Decision::BuyBackStock { amount: 10_000.0 })
        .unwrap_err();

    assert!(error.contains("cash on hand"));
}

#[test]
fn stock_buyback_has_no_fixed_command_cap() {
    let mut game = Game::with_seed(15);
    game.player.cash = 140_000.0;
    let starting_shares = game.player.shares;

    game.apply_decision(Decision::BuyBackStock { amount: 100_000.0 })
        .unwrap();

    assert!(game.player.shares < starting_shares);
    assert!(game.player.cash < 140_000.0);
}

#[test]
fn debt_repayment_reduces_cash_and_debt() {
    let mut game = Game::with_seed(11);
    let starting_cash = game.player.cash;
    let starting_debt = game.player.debt;

    game.apply_decision(Decision::RepayDebt { amount: 8_000.0 })
        .unwrap();

    assert!((game.player.cash - (starting_cash - 8_000.0)).abs() < 0.01);
    assert!((game.player.debt - (starting_debt - 8_000.0)).abs() < 0.01);
}

#[test]
fn debt_repayment_cannot_exceed_cash_available() {
    let mut game = Game::with_seed(12);
    game.player.cash = 2_500.0;

    let error = game
        .apply_decision(Decision::RepayDebt { amount: 10_000.0 })
        .unwrap_err();

    assert!(error.contains("cash on hand"));
}

#[test]
fn debt_repayment_has_no_fixed_command_cap() {
    let mut game = Game::with_seed(16);
    game.player.cash = 120_000.0;
    game.player.debt = 100_000.0;

    game.apply_decision(Decision::RepayDebt { amount: 90_000.0 })
        .unwrap();

    assert!((game.player.debt - 10_000.0).abs() < 0.01);
    assert!((game.player.cash - 30_000.0).abs() < 0.01);
}

#[test]
fn borrowing_is_limited_by_asset_debt_capacity_not_command_cap() {
    let mut game = Game::with_seed(17);
    game.player.asset_base = 240_000.0;
    game.player.debt = 20_000.0;

    game.apply_decision(Decision::Borrow { amount: 120_000.0 })
        .unwrap();

    assert!((game.player.debt - 140_000.0).abs() < 0.01);
}

#[test]
fn interest_uses_current_macro_and_leverage_rate() {
    let mut game = Game::with_seed(41);
    game.player.asset_base = 100_000.0;
    game.player.debt = 80_000.0;
    game.macro_state.annual_base_rate = 0.070;
    game.macro_state.credit_spread = 0.035;
    let expected_interest = game.player.debt * game.player_annual_interest_rate() / 4.0;
    let mut events = Vec::new();

    let finances = settle_utility(
        &mut game.player,
        &game.market,
        &game.macro_state,
        1.0,
        true,
        &mut events,
    );

    assert!((finances.interest - expected_interest).abs() < 0.01);
    assert!(finances.interest > game.player.debt * 0.0175);
}

#[test]
fn high_leverage_pays_higher_debt_rate() {
    let mut low = Game::with_seed(42);
    let mut high = Game::with_seed(42);
    low.player.asset_base = 100_000.0;
    high.player.asset_base = 100_000.0;
    low.player.debt = 25_000.0;
    high.player.debt = 90_000.0;

    assert!(
        high.player_annual_interest_rate() > low.player_annual_interest_rate() + 0.06,
        "high leverage should carry a meaningful floating-rate premium"
    );
}

#[test]
fn tight_credit_reduces_borrowing_room() {
    let mut normal = Game::with_seed(43);
    let mut tight = Game::with_seed(43);
    normal.player.asset_base = 160_000.0;
    tight.player.asset_base = 160_000.0;
    normal.player.debt = 60_000.0;
    tight.player.debt = 60_000.0;
    tight.macro_state.annual_base_rate = 0.095;
    tight.macro_state.credit_spread = 0.060;

    assert!(tight.borrowing_room() < normal.borrowing_room());
    assert!(tight.macro_state.borrowing_limit_ratio() < normal.macro_state.borrowing_limit_ratio());
}

#[test]
fn macro_environment_changes_each_quarter() {
    let mut game = Game::with_seed(44);
    let starting_rate = game.macro_state.benchmark_credit_rate();
    let starting_demand = game.macro_state.demand_index;

    game.advance_quarter();

    assert!((game.macro_state.benchmark_credit_rate() - starting_rate).abs() > 0.0001);
    assert!((game.macro_state.demand_index - starting_demand).abs() > 0.0001);
}

#[test]
fn market_variable_costs_are_bounded_over_extended_play() {
    for seed in 1..=50 {
        let mut game = Game::with_seed(seed);
        let baseline_cost = game.market.baseline_variable_cost_per_mwh;
        let mut max_cost = game.market.variable_cost_per_mwh;
        let mut min_cost = game.market.variable_cost_per_mwh;
        let mut events = Vec::new();

        for _ in 0..240 {
            game.advance_macro_environment(&mut events);
            game.grow_market(&mut events);
            max_cost = max_cost.max(game.market.variable_cost_per_mwh);
            min_cost = min_cost.min(game.market.variable_cost_per_mwh);
        }

        assert!(
            max_cost <= baseline_cost * VARIABLE_COST_CEILING_MULTIPLE + 0.01,
            "seed {seed} max cost {max_cost} exceeded bounded cost ceiling from baseline {baseline_cost}"
        );
        assert!(
            min_cost >= baseline_cost * VARIABLE_COST_FLOOR_MULTIPLE - 0.01,
            "seed {seed} min cost {min_cost} fell below bounded cost floor from baseline {baseline_cost}"
        );
    }
}

#[test]
fn variable_costs_mean_revert_after_cost_pressure_eases() {
    let mut game = Game::with_seed(45);
    let baseline_cost = game.market.baseline_variable_cost_per_mwh;
    game.market.variable_cost_per_mwh = baseline_cost * VARIABLE_COST_CEILING_MULTIPLE;
    game.macro_state.cost_pressure = -0.030;
    let mut events = Vec::new();

    for _ in 0..20 {
        game.grow_market(&mut events);
    }

    assert!(
        game.market.variable_cost_per_mwh < baseline_cost * 1.25,
        "costs should retreat after pressure eases: current {}, baseline {}",
        game.market.variable_cost_per_mwh,
        baseline_cost
    );
}

#[test]
fn mature_reliable_utility_can_profit_at_bounded_cost_ceiling() {
    let game = Game::with_seed(46);
    let mut utility = game.player.clone();
    let mut market = game.market.clone();
    let mut events = Vec::new();

    utility.customers = 4_000.0;
    utility.generation_capacity_mwh = 2_400.0;
    utility.distribution_capacity = 5_500.0;
    utility.rate_cents = 9.8;
    utility.reliability = 0.92;
    utility.reputation = 84.0;
    utility.asset_base = 620_000.0;
    utility.debt = 180_000.0;
    market.avg_mwh_per_customer = 0.45;
    market.variable_cost_per_mwh =
        market.baseline_variable_cost_per_mwh * VARIABLE_COST_CEILING_MULTIPLE;

    let finances = settle_utility(
        &mut utility,
        &market,
        &game.macro_state,
        1.0,
        true,
        &mut events,
    );

    assert!(
        finances.profit > 0.0,
        "mature reliable utility should still be profitable at bounded cost ceiling: revenue {}, cost {}, interest {}",
        finances.revenue,
        finances.operating_cost,
        finances.interest
    );
}

#[test]
fn lower_rates_and_strong_service_reduce_churn_rate() {
    let mut high_rate = Game::with_seed(45);
    let mut low_rate = high_rate.clone();
    high_rate.player.rate_cents = 11.5;
    low_rate.player.rate_cents = 8.8;
    high_rate.player.reliability = 0.92;
    low_rate.player.reliability = 0.92;
    high_rate.player.reputation = 72.0;
    low_rate.player.reputation = 72.0;
    let market_average = 10.5;

    let high_rate_churn = churn_rate_for(&high_rate.player, market_average);
    let low_rate_churn = churn_rate_for(&low_rate.player, market_average);

    assert!(low_rate_churn < high_rate_churn);
    assert!(low_rate_churn < 0.010);
}

#[test]
fn quarter_report_includes_churn_rate() {
    let mut game = Game::with_seed(46);
    game.player.customers = 900.0;
    game.player.distribution_capacity = 1_400.0;
    game.player.generation_capacity_mwh = 420.0;
    game.player.rate_cents = 8.8;
    game.player.reliability = 0.92;
    game.player.reputation = 72.0;
    let starting_customers = game.player.customers;

    let report = game.advance_quarter();

    assert!(
        (report.lost_customer_rate - report.lost_customers / starting_customers).abs() < 0.0001
    );
    assert!(report.lost_customer_rate < 0.010);
}

#[test]
fn natural_churn_floor_varies_between_quarters() {
    let mut game = Game::with_seed(47);
    game.competitors.clear();
    game.player.customers = 1_000.0;
    game.player.distribution_capacity = 2_000.0;
    game.player.generation_capacity_mwh = 700.0;
    game.player.rate_cents = 6.0;
    game.player.reliability = 0.96;
    game.player.reputation = 88.0;
    game.market.standard_rate_cents = 10.5;
    game.macro_state.demand_index = 0.0;

    let mut observed_basis_points = std::collections::BTreeSet::new();
    for _ in 0..10 {
        game.player.customers = 1_000.0;
        let lost_customers = game.customer_churn(&mut Vec::new());
        let churn_rate = lost_customers / 1_000.0;

        assert!((0.0024..=0.0065).contains(&churn_rate));
        observed_basis_points.insert((churn_rate * 10_000.0).round() as i32);
    }

    assert!(
        observed_basis_points.len() > 1,
        "natural churn floor should vary across quarters, saw {observed_basis_points:?}"
    );
}

#[test]
fn quarter_report_includes_prior_market_share() {
    let mut game = Game::with_seed(48);
    let starting_share = game.market_share();

    let report = game.advance_quarter();

    assert!((report.prior_market_share - starting_share).abs() < 0.0001);
}

#[test]
fn quarter_report_explains_major_result_drivers() {
    let mut game = Game::with_seed(49);

    let report = game.advance_quarter();

    assert!(!report.attributions.is_empty());
    assert!(
        report
            .attributions
            .iter()
            .any(|line| line.contains("Share"))
    );
    assert!(
        report
            .attributions
            .iter()
            .any(|line| line.contains("Churn"))
    );
}

#[test]
fn formal_review_can_be_continued_past_campaign_end() {
    let mut game = Game::with_seed(52);
    game.campaign_quarters = 1;
    game.player.cash = 100_000.0;
    game.player.debt = 0.0;
    game.player.customers = 1_200.0;
    game.player.generation_capacity_mwh = 600.0;
    game.player.distribution_capacity = 1_800.0;
    game.player.reliability = 0.92;
    for competitor in &mut game.competitors {
        competitor.customers = 25.0;
    }

    game.advance_quarter();

    let outcome = game.outcome.as_ref().expect("expected formal review");
    assert_eq!(outcome.kind, OutcomeKind::Victory);
    assert!(outcome.can_continue);
    assert!(outcome.details.contains("no outstanding debt"));
    assert!(!outcome.details.contains("9.9x"));
    assert!(game.review_completed);

    let message = game.continue_after_review().unwrap();
    assert!(message.contains("Continuing after"));
    assert!(game.outcome.is_none());

    let continued_quarter = game.quarter;
    game.advance_quarter();

    assert_eq!(game.quarter, continued_quarter + 1);
    assert!(game.outcome.is_none());
}

#[test]
fn formal_review_rejects_below_cost_market_lead() {
    let mut game = Game::with_seed(520);
    game.quarter = game.campaign_quarters;
    game.player.customers = 1_800.0;
    game.player.generation_capacity_mwh = 900.0;
    game.player.distribution_capacity = 2_400.0;
    game.player.reliability = 0.92;
    game.player.rate_cents = 0.4;
    game.player.debt = 0.0;
    for competitor in &mut game.competitors {
        competitor.customers = 80.0;
    }
    let finances = FirmFinances {
        revenue: 900.0,
        operating_cost: 9_000.0,
        interest: 0.0,
        profit: -8_100.0,
        served_mwh: 900.0,
        unmet_demand_ratio: 0.0,
    };

    game.check_outcome(&finances);

    let outcome = game.outcome.as_ref().expect("expected formal review");
    assert_eq!(outcome.kind, OutcomeKind::Defeat);
    assert!(outcome.can_continue);
    assert!(outcome.details.contains("rate support"));
    assert!(outcome.details.contains("no outstanding debt"));
    assert!(!outcome.details.contains("9.9x"));
}

#[test]
fn below_cost_pricing_creates_financing_pressure() {
    let mut game = Game::with_seed(521);
    game.player.rate_cents = 0.5;
    game.player.reputation = 70.0;
    game.equity_market_fatigue = 0.0;
    let finances = FirmFinances {
        revenue: 500.0,
        operating_cost: 8_000.0,
        interest: 600.0,
        profit: -8_100.0,
        served_mwh: 500.0,
        unmet_demand_ratio: 0.0,
    };
    let starting_debt = game.player.debt;
    let mut events = Vec::new();

    game.apply_below_cost_pricing_pressure(&finances, &mut events);

    assert!(game.equity_market_fatigue > 0.0);
    assert!(game.player.reputation < 70.0);
    assert!(game.player.debt > starting_debt);
    assert!(
        events
            .iter()
            .any(|event| event.contains("Below-cost pricing"))
    );
}

#[test]
fn year_two_board_checkpoint_penalizes_low_share() {
    let mut game = Game::with_seed(54);
    game.quarter = BOARD_CHECKPOINT_QUARTER;
    game.player.customers = 200.0;
    game.player.reputation = 70.0;
    game.equity_market_fatigue = 0.0;
    for competitor in &mut game.competitors {
        competitor.customers = 400.0;
    }
    let mut events = Vec::new();

    game.apply_board_checkpoint(&mut events);

    assert!((game.player.reputation - 67.0).abs() < 0.001);
    assert!((game.equity_market_fatigue - BOARD_CHECKPOINT_EQUITY_FATIGUE).abs() < 0.001);
    assert!(
        events
            .iter()
            .any(|event| event.contains("Year 2 checkpoint"))
    );
}

#[test]
fn year_two_board_checkpoint_ignores_adequate_share() {
    let mut game = Game::with_seed(55);
    game.quarter = BOARD_CHECKPOINT_QUARTER;
    game.player.customers = 800.0;
    game.player.reputation = 70.0;
    game.equity_market_fatigue = 0.0;
    for competitor in &mut game.competitors {
        competitor.customers = 200.0;
    }
    let mut events = Vec::new();

    game.apply_board_checkpoint(&mut events);

    assert!((game.player.reputation - 70.0).abs() < 0.001);
    assert_eq!(game.equity_market_fatigue, 0.0);
    assert!(events.is_empty());
}

#[test]
fn terminal_operating_failures_cannot_be_continued() {
    let mut game = Game::with_seed(53);
    game.player.reliability = 0.40;

    game.advance_quarter();

    let outcome = game.outcome.as_ref().expect("expected terminal failure");
    assert_eq!(outcome.headline, "Market Access Lost");
    assert!(!outcome.can_continue);
    assert!(game.continue_after_review().is_err());
}

#[test]
fn high_leverage_and_losses_trigger_receivership() {
    let mut game = Game::with_seed(56);
    game.quarter = 6;
    game.player.cash = 1_000.0;
    game.player.asset_base = 100_000.0;
    game.player.debt = 110_000.0;
    game.player.customers = 100.0;
    game.player.rate_cents = 1.0;
    game.player.reliability = 0.88;
    game.player.generation_capacity_mwh = 300.0;
    game.player.distribution_capacity = 400.0;

    let report = game.advance_quarter();

    let outcome = game.outcome.as_ref().expect("expected receivership");
    assert_eq!(outcome.headline, "Bankers Forced Receivership");
    assert!(!outcome.can_continue);
    assert!(report.profit < 0.0);
}

#[test]
fn churn_compares_player_rate_to_rival_rates_not_own_weighted_average() {
    let mut game = Game::with_seed(47);
    game.player.customers = 1_800.0;
    game.player.distribution_capacity = 2_400.0;
    game.player.generation_capacity_mwh = 700.0;
    game.player.rate_cents = 8.4;
    game.player.reliability = 0.84;
    game.player.reputation = 62.0;
    for competitor in &mut game.competitors {
        competitor.rate_cents = 10.9;
    }

    let self_weighted_market_rate = game.average_rate();
    let rival_rate = game.rival_average_rate_for_player();
    let self_weighted_churn = churn_rate_for(&game.player, self_weighted_market_rate);
    let rival_based_churn = churn_rate_for(&game.player, rival_rate);

    assert!(rival_rate > self_weighted_market_rate + 1.0);
    assert!(rival_based_churn < self_weighted_churn);
    assert!(rival_based_churn <= 0.005);
}

#[test]
fn distribution_project_completes_after_one_advance() {
    let mut game = Game::with_seed(2);
    let starting_capacity = game.player.distribution_capacity;

    game.apply_decision(Decision::BuildDistribution {
        customer_capacity: DISTRIBUTION_PROJECT_CAPACITY,
    })
    .unwrap();
    assert_eq!(game.pending_projects.len(), 1);

    game.advance_quarter();

    assert!(game.player.distribution_capacity > starting_capacity);
    assert!(game.pending_projects.is_empty());
}

#[test]
fn larger_projects_cost_more_but_are_more_efficient_per_unit() {
    let small_generation = generation_project_cost(100.0);
    let large_generation = generation_project_cost(400.0);
    assert!(large_generation > small_generation);
    assert!(large_generation / 400.0 < small_generation / 100.0);

    let small_distribution = distribution_project_cost(150.0);
    let large_distribution = distribution_project_cost(600.0);
    assert!(large_distribution > small_distribution);
    assert!(large_distribution / 600.0 < small_distribution / 150.0);
}

#[test]
fn custom_generation_project_uses_requested_size_and_cost() {
    let mut game = Game::with_seed(22);
    let starting_cash = game.player.cash;

    game.apply_decision(Decision::BuildGeneration {
        capacity_mwh: 300.0,
    })
    .unwrap();

    assert_eq!(game.pending_projects.len(), 1);
    assert!((game.player.cash - (starting_cash - generation_project_cost(300.0))).abs() < 0.01);
    match game.pending_projects[0].kind {
        ProjectKind::Generation { capacity_mwh } => {
            assert!((capacity_mwh - 300.0).abs() < 0.01)
        }
        _ => panic!("expected generation project"),
    }
}

#[test]
fn generation_completion_lifts_reliability_from_new_equipment() {
    let mut game = Game::with_seed(23);
    game.player.reliability = 0.74;
    game.player.generation_capacity_mwh = 120.0;
    let expected_reliability =
        generation_reliability_after_new_capacity(game.player.reliability, 120.0, 220.0);
    let mut events = Vec::new();

    game.pending_projects.push(Project {
        name: "test plant".to_string(),
        kind: ProjectKind::Generation {
            capacity_mwh: 220.0,
        },
        quarters_remaining: 1,
    });

    game.complete_projects(&mut events);

    assert!((game.player.generation_capacity_mwh - 340.0).abs() < 0.01);
    assert!((game.player.reliability - expected_reliability).abs() < 0.0001);
    assert!(game.player.reliability > 0.74);
    assert!(
        events
            .iter()
            .any(|event| event.contains("lifting reliability"))
    );
}

#[test]
fn adjacent_expansion_requires_a_mature_core_platform() {
    let mut game = eligible_adjacent_expansion_game();
    game.quarter = 7;
    let starting_cash = game.player.cash;

    let early_error = game
        .apply_decision(Decision::EnterAdjacentMarket)
        .unwrap_err();

    assert!(early_error.contains("Year 3"));
    assert!((game.player.cash - starting_cash).abs() < 0.01);

    game.quarter = 8;
    game.player.reliability = 0.81;
    let reliability_error = game
        .apply_decision(Decision::EnterAdjacentMarket)
        .unwrap_err();

    assert!(reliability_error.contains("82%"));
}

#[test]
fn adjacent_expansion_starts_project_and_costs_cash() {
    let mut game = eligible_adjacent_expansion_game();
    let starting_cash = game.player.cash;
    let starting_asset_base = game.player.asset_base;
    let cost = game.adjacent_expansion_cost();

    let message = game.apply_decision(Decision::EnterAdjacentMarket).unwrap();

    assert!(message.contains("adjacent territory"));
    assert!((game.player.cash - (starting_cash - cost)).abs() < 0.01);
    assert!((game.player.asset_base - (starting_asset_base + cost)).abs() < 0.01);
    assert_eq!(game.pending_projects.len(), 1);
    match game.pending_projects[0].kind {
        ProjectKind::AdjacentTerritory {
            addressable_customers,
            initial_customers,
            incumbent_customers,
        } => {
            assert!(
                (addressable_customers - ADJACENT_EXPANSION_ADDRESSABLE_CUSTOMERS).abs() < 0.01
            );
            assert!((initial_customers - ADJACENT_EXPANSION_INITIAL_CUSTOMERS).abs() < 0.01);
            assert!((incumbent_customers - ADJACENT_EXPANSION_INCUMBENT_CUSTOMERS).abs() < 0.01);
        }
        _ => panic!("expected adjacent territory project"),
    }
}

#[test]
fn adjacent_expansion_completion_opens_market_and_adds_incumbent() {
    let mut game = eligible_adjacent_expansion_game();
    let starting_addressable = game.market.addressable_customers;
    let starting_player_customers = game.player.customers;
    let starting_competitors = game.competitors.len();
    let mut events = Vec::new();

    game.pending_projects.push(Project {
        name: "adjacent territory entry".to_string(),
        kind: ProjectKind::AdjacentTerritory {
            addressable_customers: ADJACENT_EXPANSION_ADDRESSABLE_CUSTOMERS,
            initial_customers: ADJACENT_EXPANSION_INITIAL_CUSTOMERS,
            incumbent_customers: ADJACENT_EXPANSION_INCUMBENT_CUSTOMERS,
        },
        quarters_remaining: 1,
    });

    game.complete_projects(&mut events);

    assert_eq!(game.adjacent_expansions, 1);
    assert!(game.market.addressable_customers > starting_addressable);
    assert!(game.player.customers > starting_player_customers);
    assert_eq!(game.competitors.len(), starting_competitors + 1);
    assert!(events.iter().any(|event| event.contains("local incumbent")));
    assert!(game.regional_integration > 0.0);
}

#[test]
fn adjacent_incumbent_rate_uses_current_market_not_old_floor() {
    let mut game = eligible_adjacent_expansion_game();
    game.market.standard_rate_cents = 5.0;
    game.market.variable_cost_per_mwh = 12.0;
    game.market.baseline_variable_cost_per_mwh = 12.0;
    game.macro_state.annual_base_rate = 0.035;
    game.macro_state.credit_spread = 0.008;
    let starting_competitors = game.competitors.len();
    let mut events = Vec::new();

    game.pending_projects.push(Project {
        name: "adjacent territory entry".to_string(),
        kind: ProjectKind::AdjacentTerritory {
            addressable_customers: ADJACENT_EXPANSION_ADDRESSABLE_CUSTOMERS,
            initial_customers: ADJACENT_EXPANSION_INITIAL_CUSTOMERS,
            incumbent_customers: ADJACENT_EXPANSION_INCUMBENT_CUSTOMERS,
        },
        quarters_remaining: 1,
    });

    game.complete_projects(&mut events);

    let incumbent_rate = game.competitors[starting_competitors].rate_cents;
    assert!(
        incumbent_rate < 8.4,
        "adjacent incumbent should not inherit the old 8.4c floor; rate was {incumbent_rate}"
    );
}

#[test]
fn adjacent_expansion_blocks_parallel_projects_but_allows_limited_follow_ons() {
    let mut game = eligible_adjacent_expansion_game();
    game.player.cash = 1_000_000.0;

    game.apply_decision(Decision::EnterAdjacentMarket).unwrap();
    let pending_error = game
        .apply_decision(Decision::EnterAdjacentMarket)
        .unwrap_err();

    assert!(pending_error.contains("already in the pipeline"));

    game.pending_projects.clear();
    game.adjacent_expansions = 1;
    let second_cost = game.adjacent_expansion_cost();
    let second_message = game.apply_decision(Decision::EnterAdjacentMarket).unwrap();

    assert!(second_message.contains("adjacent territory"));
    assert!((second_cost - ADJACENT_EXPANSION_BASE_COST * 1.5).abs() < 0.01);

    game.pending_projects.clear();
    game.adjacent_expansions = MAX_ADJACENT_EXPANSIONS;
    let completed_error = game
        .apply_decision(Decision::EnterAdjacentMarket)
        .unwrap_err();

    assert!(completed_error.contains("already entered 3 adjacent territories"));
}

#[test]
fn marketing_and_maintenance_reduce_regional_integration_burden() {
    let mut game = Game::with_seed(65);
    game.player.cash = 80_000.0;
    game.player.reliability = 0.86;
    game.regional_integration = 0.35;

    game.apply_decision(Decision::Marketing { spend: 8_000.0 })
        .unwrap();
    let after_marketing = game.regional_integration;
    assert!(after_marketing < 0.35);

    game.apply_decision(Decision::Maintenance { spend: 8_000.0 })
        .unwrap();
    assert!(game.regional_integration < after_marketing);
}

#[test]
fn regional_integration_burden_drags_service_until_resolved() {
    let mut game = Game::with_seed(66);
    game.player.reliability = 0.90;
    game.player.reputation = 75.0;
    game.regional_integration = 0.40;
    let starting_reliability = game.player.reliability;
    let starting_reputation = game.player.reputation;
    let mut events = Vec::new();

    game.apply_regional_integration_strain(&mut events);

    assert!(game.player.reliability < starting_reliability);
    assert!(game.player.reputation < starting_reputation);
    assert!(game.regional_integration < 0.40);
    assert!(
        events
            .iter()
            .any(|event| event.contains("Regional expansion"))
    );
}

#[test]
fn regional_mandate_can_be_won_after_formal_review() {
    let mut game = Game::with_seed(67);
    game.review_completed = true;
    game.quarter = REGIONAL_MANDATE_QUARTER;
    game.adjacent_expansions = REGIONAL_MANDATE_EXPANSION_TARGET;
    game.regional_integration = 0.05;
    game.player.customers = 2_000.0;
    game.player.reliability = REGIONAL_MANDATE_RELIABILITY_TARGET + 0.03;
    game.player.asset_base = 400_000.0;
    game.player.debt = 120_000.0;
    for competitor in &mut game.competitors {
        competitor.customers = 120.0;
    }
    let finances = FirmFinances {
        revenue: 10_000.0,
        operating_cost: 7_000.0,
        interest: 500.0,
        profit: 2_500.0,
        served_mwh: 1_000.0,
        unmet_demand_ratio: 0.0,
    };

    game.check_outcome(&finances);

    let outcome = game.outcome.as_ref().unwrap();
    assert_eq!(outcome.kind, OutcomeKind::Victory);
    assert!(outcome.headline.contains("Regional Platform"));
    assert!(outcome.can_continue);
}

#[test]
fn regional_mandate_can_be_missed_after_formal_review() {
    let mut game = Game::with_seed(68);
    game.review_completed = true;
    game.quarter = REGIONAL_MANDATE_QUARTER;
    game.adjacent_expansions = 1;
    game.regional_integration = 0.30;
    game.player.customers = 1_000.0;
    game.player.reliability = 0.78;
    game.player.asset_base = 200_000.0;
    game.player.debt = 150_000.0;
    let finances = FirmFinances {
        revenue: 10_000.0,
        operating_cost: 7_000.0,
        interest: 500.0,
        profit: 2_500.0,
        served_mwh: 1_000.0,
        unmet_demand_ratio: 0.0,
    };

    game.check_outcome(&finances);

    let outcome = game.outcome.as_ref().unwrap();
    assert_eq!(outcome.kind, OutcomeKind::Defeat);
    assert!(outcome.headline.contains("Regional Mandate"));
    assert!(outcome.can_continue);
}

#[test]
fn larger_generation_projects_provide_larger_initial_reliability_lifts() {
    let current_reliability = 0.70;
    let existing_generation = 150.0;

    let small =
        generation_reliability_after_new_capacity(current_reliability, existing_generation, 100.0);
    let large =
        generation_reliability_after_new_capacity(current_reliability, existing_generation, 400.0);
    let already_excellent =
        generation_reliability_after_new_capacity(0.98, existing_generation, 400.0);

    assert!(small > current_reliability);
    assert!(large > small);
    assert!((already_excellent - 0.98).abs() < 0.0001);
}

fn complete_diligence(game: &mut Game, competitor_index: usize) {
    let cost = game.diligence_cost(competitor_index).unwrap();
    game.player.cash += cost;
    game.apply_decision(Decision::Diligence { competitor_index })
        .unwrap();
}

#[test]
fn acquisition_removes_competitor_and_adds_scale() {
    let mut game = Game::with_seed(3);
    game.apply_decision(Decision::IssueStock { amount: 60_000.0 })
        .unwrap();
    complete_diligence(&mut game, 2);
    let starting_customers = game.player.customers;
    let competitor_count = game.competitors.len();

    game.apply_decision(Decision::Acquire {
        competitor_index: 2,
    })
    .unwrap();

    assert_eq!(game.competitors.len(), competitor_count - 1);
    assert!(game.player.customers > starting_customers);
    assert!(game.player.generation_capacity_mwh > 72.0);
}

#[test]
fn acquisition_can_close_without_diligence_but_notes_estimate_risk() {
    let mut game = Game::with_seed(30);
    game.player.cash = 250_000.0;
    let competitor_count = game.competitors.len();

    let message = game
        .apply_decision(Decision::Acquire {
            competitor_index: 1,
        })
        .unwrap();

    assert!(message.contains("without diligence"));
    assert_eq!(game.competitors.len(), competitor_count - 1);
    assert!(!game.has_diligence(1));
}

#[test]
fn risky_acquisition_creates_lender_stress_signal() {
    let mut game = Game::with_seed(30);
    game.player.cash = 600_000.0;
    game.player.debt = 120_000.0;
    game.player.asset_base = 155_000.0;
    game.competitors[0].customers = 780.0;
    game.competitors[0].reliability = 0.58;
    game.competitors[0].reputation = 34.0;
    game.competitors[0].debt = 95_000.0;
    game.competitors[0].asset_base = 110_000.0;

    let score = game.acquisition_stress_score(0).unwrap();
    assert!(
        score >= ACQUISITION_STRESS_STRAINED,
        "expected visibly strained underwriting score, got {score:.2}"
    );

    let message = game
        .apply_decision(Decision::Acquire {
            competitor_index: 0,
        })
        .unwrap();

    assert!(message.contains("Underwriters"));
    assert!(game.acquisition_stress >= ACQUISITION_STRESS_STRAINED);
}

#[test]
fn acquisition_stress_can_force_receivership_when_losses_follow() {
    let mut game = Game::with_seed(31);
    game.quarter = 8;
    game.acquisition_stress = ACQUISITION_STRESS_RECEIVERSHIP + 0.04;
    game.player.cash = -3_000.0;
    game.player.debt = 98_000.0;
    game.player.asset_base = 100_000.0;

    let finances = FirmFinances {
        revenue: 7_000.0,
        operating_cost: 11_500.0,
        interest: 2_000.0,
        profit: -6_500.0,
        served_mwh: 0.0,
        unmet_demand_ratio: 0.0,
    };

    game.check_outcome(&finances);

    let outcome = game.outcome.as_ref().expect("expected receivership");
    assert_eq!(outcome.headline, "Bankers Forced Receivership");
    assert!(outcome.details.contains("strained acquisition"));
    assert!(!outcome.can_continue);
}

#[test]
fn acquisition_stress_pressure_hits_cash_and_financing_flexibility() {
    let mut game = Game::with_seed(32);
    game.acquisition_stress = ACQUISITION_STRESS_STRAINED + 0.10;
    game.player.cash = 5_000.0;
    game.player.debt = 150_000.0;
    game.player.asset_base = 170_000.0;
    let starting_cash = game.player.cash;
    let starting_fatigue = game.equity_market_fatigue;
    let mut events = Vec::new();
    let finances = FirmFinances {
        revenue: 9_000.0,
        operating_cost: 10_500.0,
        interest: 2_200.0,
        profit: -3_700.0,
        served_mwh: 0.0,
        unmet_demand_ratio: 0.0,
    };

    game.apply_acquisition_stress_pressure(&finances, &mut events);

    assert!(game.player.cash < starting_cash);
    assert!(game.equity_market_fatigue > starting_fatigue);
    assert!(events.iter().any(|event| event.contains("covenants")));
}

#[test]
fn public_acquisition_estimate_can_diverge_from_exact_balance_sheet() {
    let mut game = Game::with_seed(30);
    game.competitors[0].public_cash_multiplier = 1.20;
    game.competitors[0].public_debt_multiplier = 0.80;

    let exact = game.current_acquisition_terms(0).unwrap();
    let public = game.public_acquisition_estimate(0).unwrap();

    game.apply_decision(Decision::Diligence {
        competitor_index: 0,
    })
    .unwrap();
    let diligence = game.acquisition_terms(0).unwrap();
    let exact_after_alert = game.current_acquisition_terms(0).unwrap();

    assert!(public.absorbed_cash > exact.absorbed_cash);
    assert!(public.assumed_debt < exact.assumed_debt);
    assert!((diligence.absorbed_cash - exact_after_alert.absorbed_cash).abs() < 0.01);
    assert!((diligence.assumed_debt - exact_after_alert.assumed_debt).abs() < 0.01);
}

#[test]
fn diligence_alerts_target_before_quote_is_frozen() {
    let mut game = Game::with_seed(31);
    game.player.cash = 500_000.0;
    game.competitors[0].debt = 0.0;
    game.competitors[0].rate_cents = game.player.rate_cents + 1.0;
    let before = game.competitors[0].clone();
    let live_before = game.current_acquisition_terms(0).unwrap();

    let message = game
        .apply_decision(Decision::Diligence {
            competitor_index: 0,
        })
        .unwrap();

    let after = &game.competitors[0];
    let quoted = game.acquisition_terms(0).unwrap();
    assert!(message.contains("mounted a defense"));
    assert!(after.debt > before.debt);
    assert!(after.marketing_momentum > before.marketing_momentum);
    assert!(after.asset_base > before.asset_base);
    assert!(after.rate_cents < before.rate_cents);
    assert!(
        (quoted.price - live_before.price).abs() > 100.0,
        "quote should reflect the target response before freezing: before {}, quoted {}",
        live_before.price,
        quoted.price
    );
}

#[test]
fn renewing_active_diligence_does_not_stack_target_alerts() {
    let mut game = Game::with_seed(31);
    game.player.cash = 500_000.0;
    game.apply_decision(Decision::Diligence {
        competitor_index: 0,
    })
    .unwrap();
    let after_first_alert = game.competitors[0].clone();

    let message = game
        .apply_decision(Decision::Diligence {
            competitor_index: 0,
        })
        .unwrap();

    let after_extension = &game.competitors[0];
    assert!(message.contains("extended without a new market signal"));
    assert!((after_extension.debt - after_first_alert.debt).abs() < 0.01);
    assert!((after_extension.asset_base - after_first_alert.asset_base).abs() < 0.01);
    assert!(
        (after_extension.marketing_momentum - after_first_alert.marketing_momentum).abs() < 0.0001
    );
    assert!((after_extension.rate_cents - after_first_alert.rate_cents).abs() < 0.0001);
}

#[test]
fn active_diligence_target_accelerates_defense_over_quarter() {
    let mut game = Game::with_seed(31);
    game.player.cash = 500_000.0;
    game.apply_decision(Decision::Diligence {
        competitor_index: 0,
    })
    .unwrap();
    let starting_momentum = game.competitors[0].marketing_momentum;
    let mut events = Vec::new();

    game.competitor_plans(&mut events);

    assert!(game.competitors[0].marketing_momentum > starting_momentum);
    assert!(
        events
            .iter()
            .any(|event| event.contains("while under diligence"))
    );
}

#[test]
fn diligence_expires_after_three_quarters() {
    let mut game = Game::with_seed(30);
    game.player.cash = 250_000.0;

    game.apply_decision(Decision::Diligence {
        competitor_index: 1,
    })
    .unwrap();
    assert!(game.has_diligence(1));

    game.advance_quarter();
    game.advance_quarter();
    game.advance_quarter();

    assert!(!game.has_diligence(1));
}

#[test]
fn diligence_freezes_acquisition_price_until_window_expires() {
    let mut game = Game::with_seed(31);
    game.player.cash = 500_000.0;
    game.apply_decision(Decision::Diligence {
        competitor_index: 0,
    })
    .unwrap();
    let quoted = game.acquisition_terms(0).unwrap();

    game.player.customers += 1_200.0;
    game.competitors[0].customers += 300.0;
    game.competitors[0].cash += 60_000.0;
    game.competitors[0].debt += 20_000.0;
    let live_terms = game.current_acquisition_terms(0).unwrap();

    assert!(
        (live_terms.price - quoted.price).abs() > 10_000.0,
        "test setup should materially move live price: quoted {}, live {}",
        quoted.price,
        live_terms.price
    );
    assert!((game.acquisition_price(0) - quoted.price).abs() < 0.01);
    assert!((game.acquisition_terms(0).unwrap().absorbed_cash - quoted.absorbed_cash).abs() < 0.01);
    assert!((game.acquisition_terms(0).unwrap().assumed_debt - quoted.assumed_debt).abs() < 0.01);

    game.diligence_reports[0].quarters_remaining = 0;
    game.prune_diligence_reports();

    assert!(!game.has_diligence(0));
    assert!((game.acquisition_price(0) - live_terms.price).abs() < 0.01);
}

#[test]
fn frozen_diligence_quote_recalculates_current_financing_context() {
    let mut game = Game::with_seed(32);
    game.player.cash = 250_000.0;
    game.apply_decision(Decision::Diligence {
        competitor_index: 1,
    })
    .unwrap();
    let quoted = game.acquisition_terms(1).unwrap();

    game.player.cash += 40_000.0;
    game.player.debt += 15_000.0;
    let updated = game.acquisition_terms(1).unwrap();

    assert!((updated.price - quoted.price).abs() < 0.01);
    assert!((updated.absorbed_cash - quoted.absorbed_cash).abs() < 0.01);
    assert!((updated.assumed_debt - quoted.assumed_debt).abs() < 0.01);
    assert!(
        (updated.post_cash - (game.player.cash - quoted.price + quoted.absorbed_cash)).abs() < 0.01
    );
    assert!((updated.post_debt - (game.player.debt + quoted.assumed_debt)).abs() < 0.01);
}

#[test]
fn acquisition_integration_blocks_immediate_rollup() {
    let mut game = Game::with_seed(30);
    game.apply_decision(Decision::IssueStock { amount: 60_000.0 })
        .unwrap();
    complete_diligence(&mut game, 2);
    game.apply_decision(Decision::Acquire {
        competitor_index: 2,
    })
    .unwrap();

    assert!(game.acquisition_cooldown >= 4);
    assert!(game.integration_strain > 0.0);

    let error = game
        .apply_decision(Decision::Acquire {
            competitor_index: 1,
        })
        .unwrap_err();

    assert!(error.contains("Integration capacity"));
}

#[test]
fn acquisition_cooldown_scales_with_target_size() {
    let small = acquisition_integration_cooldown(1_000.0, 50.0);
    let midsize = acquisition_integration_cooldown(1_000.0, 400.0);
    let peer_sized = acquisition_integration_cooldown(1_000.0, 1_000.0);
    let larger_than_player = acquisition_integration_cooldown(1_000.0, 1_200.0);

    assert_eq!(small, 2);
    assert!(midsize > small);
    assert!(peer_sized > midsize);
    assert!(larger_than_player >= 4);
}

#[test]
fn high_share_consolidation_premium_exceeds_two_times() {
    assert!(acquisition_consolidation_premium(0.70) > 2.0);
}

#[test]
fn acquisition_strain_temporarily_drags_operations() {
    let mut game = Game::with_seed(30);
    game.integration_strain = acquisition_integration_strain(1_000.0, 500.0);
    let starting_strain = game.integration_strain;
    let starting_reliability = game.player.reliability;
    let starting_reputation = game.player.reputation;
    let mut events = Vec::new();

    game.apply_integration_strain(&mut events);

    assert!(game.integration_strain < starting_strain);
    assert!(game.player.reliability < starting_reliability);
    assert!(game.player.reputation < starting_reputation);
    assert!(
        events
            .iter()
            .any(|event| event.contains("Acquisition integration strained"))
    );
}

#[test]
fn merger_execution_risk_rises_with_weak_large_targets_and_leverage() {
    let mut safe = Game::with_seed(33);
    safe.player.customers = 1_200.0;
    safe.player.debt = 10_000.0;
    safe.player.asset_base = 220_000.0;
    safe.competitors[0].customers = 90.0;
    safe.competitors[0].reliability = 0.92;
    safe.competitors[0].reputation = 78.0;
    let safe_target = safe.competitors[0].clone();
    let safe_terms = safe.current_acquisition_terms(0).unwrap();
    let safe_risk =
        merger_execution_risk(&safe, &safe_target, &safe_terms, safe.player.customers, 0.0);

    let mut risky = Game::with_seed(33);
    risky.player.customers = 180.0;
    risky.player.debt = 130_000.0;
    risky.player.asset_base = 170_000.0;
    risky.competitors[0].customers = 720.0;
    risky.competitors[0].reliability = 0.58;
    risky.competitors[0].reputation = 36.0;
    let risky_target = risky.competitors[0].clone();
    let risky_terms = risky.current_acquisition_terms(0).unwrap();
    let risky_risk = merger_execution_risk(
        &risky,
        &risky_target,
        &risky_terms,
        risky.player.customers,
        0.45,
    );

    assert!(
        risky_risk > safe_risk + 0.25,
        "large weak leveraged deals should be materially riskier: safe {safe_risk}, risky {risky_risk}"
    );
}

#[test]
fn regulator_blocks_purchase_of_final_independent_rival() {
    let mut game = Game::with_seed(31);
    game.competitors.truncate(1);
    game.acquisition_cooldown = 0;
    game.apply_decision(Decision::IssueStock { amount: 80_000.0 })
        .unwrap();

    let error = game
        .apply_decision(Decision::Acquire {
            competitor_index: 0,
        })
        .unwrap_err();

    assert!(error.contains("last independent rival"));
}

#[test]
fn high_share_makes_acquisitions_non_linearly_pricier() {
    let low = Game::with_seed(50);
    let mut high = Game::with_seed(50);
    high.player.customers = 1_500.0;

    let low_price = low.acquisition_price(0);
    let high_price = high.acquisition_price(0);

    assert!(
        high_price > low_price * 1.4,
        "consolidation premium should rise non-linearly: low {low_price}, high {high_price}"
    );
}

#[test]
fn acquisitions_crossing_control_threshold_include_public_interest_concession() {
    let game = Game::with_seed(52);
    let terms = game.acquisition_terms(0).unwrap();

    assert!(terms.post_market_share > 0.50);
    assert!(terms.public_interest_concession > 0.0);
}

#[test]
fn acquisition_price_accounts_for_target_cash_and_debt() {
    let base = Game::with_seed(51);
    let mut cash_rich = base.clone();
    let mut debt_heavy = base.clone();
    cash_rich.competitors[0].cash += 40_000.0;
    debt_heavy.competitors[0].debt += 40_000.0;

    assert!(
        cash_rich.acquisition_price(0) > base.acquisition_price(0) + 30_000.0,
        "cash-rich targets should cost more because that cash is acquired"
    );
    assert!(
        debt_heavy.acquisition_price(0) < base.acquisition_price(0) - 25_000.0,
        "debt-heavy targets should have lower equity purchase prices because debt is assumed"
    );
}

#[test]
fn rival_counteroffensive_strengthens_largest_rival() {
    let mut game = Game::with_seed(89);
    game.player.customers = 900.0;
    game.competitors[0].customers = 420.0;
    game.competitors[1].customers = 180.0;
    let starting_capacity = game.competitors[0].customer_capacity(&game.market);
    let starting_reputation = game.competitors[0].reputation;
    let starting_rate = game.competitors[0].rate_cents;
    let mut events = Vec::new();

    assert!(game.rival_counteroffensive(&mut events));

    assert!(game.competitors[0].customer_capacity(&game.market) > starting_capacity);
    assert!(game.competitors[0].reputation > starting_reputation);
    assert!(game.competitors[0].rate_cents <= starting_rate);
    assert!(
        events
            .iter()
            .any(|event| event.contains("counteroffensive"))
    );
}

#[test]
fn rival_counteroffensive_handles_rates_below_old_floor() {
    let mut game = Game::with_seed(90);
    game.player.customers = 900.0;
    game.player.rate_cents = 5.0;
    game.competitors[0].customers = 420.0;
    game.competitors[0].rate_cents = 7.2;
    game.competitors[1].customers = 180.0;
    let mut events = Vec::new();

    assert!(game.rival_counteroffensive(&mut events));

    assert!(game.competitors[0].rate_cents.is_finite());
    assert!(game.competitors[0].rate_cents > 0.0);
    assert!(game.competitors[0].rate_cents <= 7.2);
}

#[test]
fn low_rate_extended_market_advances_without_rate_bound_panic() {
    let mut game = Game::with_seed(91);
    game.review_completed = true;
    game.quarter = 28;
    game.market.standard_rate_cents = 4.0;
    game.market.rate_tolerance_adjustment_cents = 0.4;
    game.market.variable_cost_per_mwh = 12.0;
    game.market.baseline_variable_cost_per_mwh = 12.0;
    game.macro_state.annual_base_rate = 0.035;
    game.macro_state.credit_spread = 0.008;
    game.player.cash = 220_000.0;
    game.player.debt = 55_000.0;
    game.player.asset_base = 360_000.0;
    game.player.customers = 1_100.0;
    game.player.generation_capacity_mwh = 3_300.0;
    game.player.distribution_capacity = 1_650.0;
    game.player.rate_cents = 5.0;
    game.player.reliability = 0.91;
    game.player.reputation = 76.0;
    for (index, competitor) in game.competitors.iter_mut().enumerate() {
        competitor.cash = 85_000.0;
        competitor.debt = 25_000.0;
        competitor.asset_base = 95_000.0;
        competitor.customers = 240.0 + index as f64 * 70.0;
        competitor.generation_capacity_mwh = competitor.customers * 2.4;
        competitor.distribution_capacity = competitor.customers * 1.35;
        competitor.rate_cents = 6.2 + index as f64 * 0.2;
        competitor.reliability = 0.84;
        competitor.reputation = 58.0;
    }

    for _ in 0..12 {
        game.advance_quarter();
        assert!(game.player.rate_cents.is_finite());
        for competitor in &game.competitors {
            assert!(competitor.rate_cents.is_finite());
            assert!(competitor.rate_cents > 0.0);
        }
    }
}

#[test]
fn acquisition_absorbs_target_cash_and_debt() {
    let mut game = Game::with_seed(60);
    game.player.cash = 250_000.0;
    let target = game.competitors[2].clone();
    let starting_cash_after_payment_only = game.player.cash - game.acquisition_price(2);
    let starting_debt = game.player.debt;
    complete_diligence(&mut game, 2);

    game.apply_decision(Decision::Acquire {
        competitor_index: 2,
    })
    .unwrap();

    assert!(game.player.cash > starting_cash_after_payment_only + target.cash * 0.5);
    assert!(game.player.debt > starting_debt + target.debt * 0.5);
}

#[test]
fn acquisition_terms_match_actual_close_effects() {
    let mut game = Game::with_seed(62);
    game.player.cash = 350_000.0;
    let starting_customers = game.player.customers;
    let starting_generation = game.player.generation_capacity_mwh;
    let starting_distribution = game.player.distribution_capacity;
    complete_diligence(&mut game, 1);
    let terms = game.acquisition_terms(1).unwrap();

    assert!((game.acquisition_price(1) - terms.price).abs() < 0.01);

    game.apply_decision(Decision::Acquire {
        competitor_index: 1,
    })
    .unwrap();

    assert!((game.player.cash - terms.post_cash).abs() < 0.01);
    assert!((game.player.debt - terms.post_debt).abs() < 0.01);
    assert!((game.player.asset_base - terms.post_asset_base).abs() < 0.01);
    assert!((game.player.customers - (starting_customers + terms.acquired_customers)).abs() < 0.01);
    assert!(
        (game.player.generation_capacity_mwh
            - (starting_generation + terms.acquired_generation_capacity_mwh))
            .abs()
            < 0.01
    );
    assert!(
        (game.player.distribution_capacity
            - (starting_distribution + terms.acquired_distribution_capacity))
            .abs()
            < 0.01
    );
}

#[test]
fn acquisition_integration_uses_pre_acquisition_weights() {
    let mut game = Game::with_seed(61);
    game.player.cash = 400_000.0;
    game.player.customers = 100.0;
    game.player.generation_capacity_mwh = 100.0;
    game.player.reputation = 80.0;
    game.player.reliability = 0.92;
    game.competitors[0].customers = 1_000.0;
    game.competitors[0].generation_capacity_mwh = 1_000.0;
    game.competitors[0].reputation = 40.0;
    game.competitors[0].reliability = 0.60;
    complete_diligence(&mut game, 0);
    let target = game.competitors[0].clone();
    let terms = game.acquisition_terms(0).unwrap();

    let expected_reputation = weighted_average(
        80.0,
        100.0,
        target.reputation,
        terms.acquired_customers * 0.65,
    ) - 1.5;
    let expected_reliability = weighted_average(
        0.92,
        100.0,
        target.reliability,
        terms.acquired_generation_capacity_mwh * 0.70,
    ) - 0.035;

    let message = game
        .apply_decision(Decision::Acquire {
            competitor_index: 0,
        })
        .unwrap();

    if message.contains("Integration broke badly") {
        assert!(game.player.reputation < expected_reputation);
        assert!(game.player.reliability < expected_reliability);
    } else {
        assert!((game.player.reputation - expected_reputation).abs() < 0.001);
        assert!((game.player.reliability - expected_reliability).abs() < 0.001);
    }
}

#[test]
fn reliability_decays_without_maintenance() {
    let mut game = Game::with_seed(70);
    let starting_reliability = game.player.reliability;

    for _ in 0..6 {
        game.advance_quarter();
    }

    assert!(
        game.player.reliability < starting_reliability - 0.04,
        "reliability should decay: {} -> {}",
        starting_reliability,
        game.player.reliability
    );
}

#[test]
fn maintenance_has_diminishing_returns_at_high_reliability() {
    let mut low = Game::with_seed(71);
    let mut high = Game::with_seed(71);
    low.player.reliability = 0.55;
    high.player.reliability = 0.95;

    let low_starting = low.player.reliability;
    let high_starting = high.player.reliability;

    low.apply_decision(Decision::Maintenance { spend: 5_000.0 })
        .unwrap();
    high.apply_decision(Decision::Maintenance { spend: 5_000.0 })
        .unwrap();

    let low_gain = low.player.reliability - low_starting;
    let high_gain = high.player.reliability - high_starting;

    assert!(
        low_gain > high_gain * 2.0,
        "low reliability should gain much more: low {low_gain} vs high {high_gain}"
    );
}

#[test]
fn maintenance_at_reliability_cap_is_rejected_without_spending() {
    let mut game = Game::with_seed(73);
    game.player.reliability = MAX_RELIABILITY;
    game.player.reputation = 50.0;
    let starting_cash = game.player.cash;
    let starting_reputation = game.player.reputation;

    let error = game
        .apply_decision(Decision::Maintenance { spend: 6_000.0 })
        .unwrap_err();

    assert!(error.contains("98% operating cap"));
    assert_eq!(game.player.cash, starting_cash);
    assert_eq!(game.player.reliability, MAX_RELIABILITY);
    assert_eq!(game.player.reputation, starting_reputation);
}

#[test]
fn maintenance_reports_actual_capped_reliability_gain() {
    let mut game = Game::with_seed(74);
    game.player.reliability = 0.976;
    game.player.cash = 50_000.0;
    let starting_reliability = game.player.reliability;

    let message = game
        .apply_decision(Decision::Maintenance { spend: 20_000.0 })
        .unwrap();

    let actual_gain_points = (game.player.reliability - starting_reliability) * 100.0;
    assert_eq!(game.player.reliability, MAX_RELIABILITY);
    assert!(message.contains(&format!("{actual_gain_points:.1} reliability points")));
}

#[test]
fn maintenance_has_diminishing_returns_as_asset_base_grows() {
    let mut small = Game::with_seed(72);
    let mut large = Game::with_seed(72);
    small.player.asset_base = 60_000.0;
    large.player.asset_base = 300_000.0;
    small.player.reliability = 0.76;
    large.player.reliability = 0.76;
    small.player.reputation = 55.0;
    large.player.reputation = 55.0;

    let small_reliability = small.player.reliability;
    let large_reliability = large.player.reliability;
    let small_reputation = small.player.reputation;
    let large_reputation = large.player.reputation;

    small
        .apply_decision(Decision::Maintenance { spend: 6_000.0 })
        .unwrap();
    large
        .apply_decision(Decision::Maintenance { spend: 6_000.0 })
        .unwrap();

    let small_reliability_gain = small.player.reliability - small_reliability;
    let large_reliability_gain = large.player.reliability - large_reliability;
    let small_reputation_gain = small.player.reputation - small_reputation;
    let large_reputation_gain = large.player.reputation - large_reputation;

    assert!(
        small_reliability_gain > large_reliability_gain * 1.8,
        "small asset base should gain more reliability: small {small_reliability_gain}, large {large_reliability_gain}"
    );
    assert!(
        small_reputation_gain > large_reputation_gain * 1.8,
        "small asset base should gain more reputation: small {small_reputation_gain}, large {large_reputation_gain}"
    );
}

#[test]
fn reliable_service_at_reasonable_utilization_lifts_reputation() {
    let mut game = Game::with_seed(73);
    game.player.cash = 80_000.0;
    game.player.customers = 500.0;
    game.player.generation_capacity_mwh = 180.0;
    game.player.distribution_capacity = 800.0;
    game.player.reliability = 0.92;
    game.player.reputation = 55.0;
    let starting_reputation = game.player.reputation;
    let mut events = Vec::new();

    settle_utility(
        &mut game.player,
        &game.market,
        &game.macro_state,
        1.0,
        true,
        &mut events,
    );

    assert!(game.player.reputation > starting_reputation);
    assert!(
        events
            .iter()
            .any(|event| event.contains("manageable utilization"))
    );
}

#[test]
fn reliable_service_reputation_event_suppressed_when_displayed_gain_is_zero() {
    let mut game = Game::with_seed(73);
    game.player.cash = 80_000.0;
    game.player.customers = 500.0;
    game.player.generation_capacity_mwh = 180.0;
    game.player.distribution_capacity = 800.0;
    game.player.reliability = 0.92;
    game.player.reputation = 99.99;
    let mut events = Vec::new();

    settle_utility(
        &mut game.player,
        &game.market,
        &game.macro_state,
        1.0,
        true,
        &mut events,
    );

    assert_eq!(game.player.reputation, 100.0);
    assert!(
        !events
            .iter()
            .any(|event| event.contains("manageable utilization"))
    );
}

#[test]
fn high_utilization_blocks_passive_reputation_gain() {
    let mut game = Game::with_seed(74);
    game.player.cash = 80_000.0;
    game.player.customers = 710.0;
    game.player.generation_capacity_mwh = 180.0;
    game.player.distribution_capacity = 900.0;
    game.player.reliability = 0.92;
    game.player.reputation = 55.0;
    let starting_reputation = game.player.reputation;
    let mut events = Vec::new();

    settle_utility(
        &mut game.player,
        &game.market,
        &game.macro_state,
        1.0,
        true,
        &mut events,
    );

    assert!((game.player.reputation - starting_reputation).abs() < 0.001);
    assert!(
        !events
            .iter()
            .any(|event| event.contains("manageable utilization"))
    );
}

#[test]
fn stock_price_can_exceed_old_eighty_five_cap_with_strong_fundamentals() {
    let mut game = Game::with_seed(81);
    game.player.stock_price = 200.0;
    game.player.asset_base = 800_000.0;
    game.player.cash = 100_000.0;
    game.player.debt = 30_000.0;
    game.player.shares = 5_000.0;

    game.advance_quarter();

    assert!(
        game.player.stock_price > 85.0,
        "stock should not be hard-capped at 85, got {}",
        game.player.stock_price
    );
}

#[test]
fn stock_price_reflects_balance_sheet_equity() {
    let mut healthy = Game::with_seed(82);
    let mut leveraged = Game::with_seed(82);

    healthy.player.cash = 80_000.0;
    healthy.player.debt = 10_000.0;
    leveraged.player.cash = 5_000.0;
    leveraged.player.debt = 90_000.0;

    for _ in 0..3 {
        healthy.advance_quarter();
        leveraged.advance_quarter();
    }

    assert!(
        healthy.player.stock_price > leveraged.player.stock_price + 1.0,
        "healthier balance sheet should produce a higher stock price: healthy {}, leveraged {}",
        healthy.player.stock_price,
        leveraged.player.stock_price
    );
}

#[test]
fn reputation_lifts_earnings_multiple() {
    let mut weak = Game::with_seed(83);
    let mut strong = weak.clone();
    weak.player.reputation = 30.0;
    strong.player.reputation = 85.0;

    assert!(strong.earnings_multiple() > weak.earnings_multiple() + 1.0);
}

#[test]
fn active_shocks_affect_stock_valuation_immediately() {
    let mut normal = Game::with_seed(84);
    let mut frozen = normal.clone();
    frozen.active_shocks.push(ActiveShock {
        kind: ShockKind::RateFreeze,
        quarters_remaining: 3,
    });

    normal.advance_quarter();
    frozen.advance_quarter();

    assert!(
        frozen.player.stock_price < normal.player.stock_price,
        "rate freeze should create an immediate valuation drag: frozen {}, normal {}",
        frozen.player.stock_price,
        normal.player.stock_price
    );
}

#[test]
fn rate_freeze_blocks_rate_increases() {
    let mut game = Game::with_seed(90);
    game.active_shocks.push(ActiveShock {
        kind: ShockKind::RateFreeze,
        quarters_remaining: 2,
    });

    let error = game
        .apply_decision(Decision::AdjustRate { delta_cents: 0.5 })
        .unwrap_err();

    assert!(error.contains("rate freeze"));
}

#[test]
fn rate_freeze_allows_rate_cuts() {
    let mut game = Game::with_seed(91);
    game.active_shocks.push(ActiveShock {
        kind: ShockKind::RateFreeze,
        quarters_remaining: 2,
    });

    game.apply_decision(Decision::AdjustRate { delta_cents: -0.5 })
        .unwrap();
}

#[test]
fn rates_can_be_cut_below_former_seven_cent_floor() {
    let mut game = Game::with_seed(92);
    game.player.rate_cents = 7.2;

    game.apply_decision(Decision::AdjustRate { delta_cents: -1.0 })
        .unwrap();

    assert!((game.player.rate_cents - 6.2).abs() < 0.001);
}

#[test]
fn nonpositive_rates_are_rejected_without_clamping() {
    let mut game = Game::with_seed(92);
    game.player.rate_cents = 6.2;

    let error = game
        .apply_decision(Decision::AdjustRate { delta_cents: -6.2 })
        .unwrap_err();

    assert!(error.contains("above 0.0c"));
    assert!((game.player.rate_cents - 6.2).abs() < 0.001);
}

#[test]
fn below_cost_rates_reduce_quarter_profit() {
    let game = Game::with_seed(92);
    let mut normal = game.player.clone();
    let mut underpriced = game.player.clone();
    normal.rate_cents = 10.0;
    underpriced.rate_cents = 4.0;

    let normal_profit = settle_utility(
        &mut normal,
        &game.market,
        &game.macro_state,
        1.0,
        true,
        &mut Vec::new(),
    )
    .profit;
    let underpriced_profit = settle_utility(
        &mut underpriced,
        &game.market,
        &game.macro_state,
        1.0,
        true,
        &mut Vec::new(),
    )
    .profit;

    assert!(
        underpriced_profit < normal_profit - 1_000.0,
        "underpriced profit {underpriced_profit} should trail normal profit {normal_profit}"
    );
}

#[test]
fn sustained_below_cost_rates_create_financing_stress() {
    let mut normal = Game::with_seed(93);
    let mut underpriced = normal.clone();
    normal.player.rate_cents = 9.5;
    underpriced.player.rate_cents = 4.0;

    for _ in 0..8 {
        normal.advance_quarter();
        underpriced.advance_quarter();
    }

    let normal_net_liquidity = normal.player.cash - normal.player.debt;
    let underpriced_net_liquidity = underpriced.player.cash - underpriced.player.debt;
    assert!(
        underpriced_net_liquidity < normal_net_liquidity - 12_000.0,
        "underpriced net liquidity {underpriced_net_liquidity} should trail normal {normal_net_liquidity}"
    );
}

#[test]
fn rates_can_exceed_old_fifteen_cent_cap() {
    let mut game = Game::with_seed(85);
    game.player.rate_cents = 14.6;

    game.apply_decision(Decision::AdjustRate { delta_cents: 1.2 })
        .unwrap();

    assert!((game.player.rate_cents - 15.8).abs() < 0.001);
}

#[test]
fn extreme_rates_are_rejected_above_public_ceiling() {
    let mut game = Game::with_seed(85);
    let starting_rate = game.player.rate_cents;

    let error = game
        .apply_decision(Decision::AdjustRate { delta_cents: 40.0 })
        .unwrap_err();

    assert!(error.contains("beyond public tolerance"));
    assert!((game.player.rate_cents - starting_rate).abs() < 0.001);
}

#[test]
fn public_rate_ceiling_allows_some_premium_but_rejects_runaway_rates() {
    let mut game = Game::with_seed(85);
    let tolerance = public_rate_tolerance(&game.market);
    game.player.rate_cents = tolerance + MAX_PUBLIC_RATE_PREMIUM_CENTS - 0.2;

    game.apply_decision(Decision::AdjustRate { delta_cents: 0.1 })
        .unwrap();
    let error = game
        .apply_decision(Decision::AdjustRate { delta_cents: 0.3 })
        .unwrap_err();

    assert!(error.contains("beyond public tolerance"));
    assert!(
        game.player.rate_cents <= tolerance + MAX_PUBLIC_RATE_PREMIUM_CENTS + 0.001,
        "rate should remain inside public ceiling"
    );
}

#[test]
fn public_rate_ceiling_still_allows_cuts_from_above_ceiling() {
    let mut game = Game::with_seed(85);
    let tolerance = public_rate_tolerance(&game.market);
    game.player.rate_cents = tolerance + MAX_PUBLIC_RATE_PREMIUM_CENTS + 1.0;

    game.apply_decision(Decision::AdjustRate { delta_cents: -0.4 })
        .unwrap();

    assert!(
        (game.player.rate_cents - (tolerance + MAX_PUBLIC_RATE_PREMIUM_CENTS + 0.6)).abs() < 0.001
    );
}

#[test]
fn public_rate_tolerance_rises_with_boom_and_strong_service() {
    let mut game = Game::with_seed(86);
    let starting_tolerance = public_rate_tolerance(&game.market);
    game.macro_state.demand_index = 0.62;
    game.macro_state.annual_base_rate = 0.038;
    game.macro_state.credit_spread = 0.012;
    game.macro_state.cost_pressure = -0.012;
    game.player.reliability = 0.94;
    for competitor in &mut game.competitors {
        competitor.reliability = 0.90;
    }
    let mut events = Vec::new();

    game.update_public_rate_tolerance(0.0, &mut events);

    assert!(public_rate_tolerance(&game.market) > starting_tolerance + 0.20);
    assert!(
        events
            .iter()
            .any(|event| event.contains("tolerance improved"))
    );
}

#[test]
fn public_rate_tolerance_falls_with_stress_and_poor_service() {
    let mut game = Game::with_seed(86);
    let starting_tolerance = public_rate_tolerance(&game.market);
    game.macro_state.demand_index = -0.55;
    game.macro_state.annual_base_rate = 0.10;
    game.macro_state.credit_spread = 0.06;
    game.macro_state.cost_pressure = 0.06;
    game.player.reliability = 0.68;
    for competitor in &mut game.competitors {
        competitor.reliability = 0.70;
    }
    game.active_shocks.push(ActiveShock {
        kind: ShockKind::DemandRecession,
        quarters_remaining: 2,
    });
    let mut events = Vec::new();

    game.update_public_rate_tolerance(0.10, &mut events);

    assert!(public_rate_tolerance(&game.market) < starting_tolerance - 0.35);
    assert!(
        events
            .iter()
            .any(|event| event.contains("tolerance tightened"))
    );
}

#[test]
fn rates_above_public_tolerance_churn_harder() {
    let mut game = Game::with_seed(86);
    game.player.rate_cents = public_rate_tolerance(&game.market) + 3.5;
    game.player.reliability = 0.90;
    game.player.reputation = 70.0;
    let market_rate = game.market.standard_rate_cents;

    let old_style_churn = churn_rate_for(&game.player, market_rate);
    let tolerance_churn = churn_rate_for_market(&game.player, market_rate, &game.market);

    assert!(
        tolerance_churn > old_style_churn + 0.08,
        "high public-rate pressure should materially lift churn: old {old_style_churn}, new {tolerance_churn}"
    );
}

#[test]
fn rates_above_public_tolerance_damage_reputation() {
    let mut game = Game::with_seed(87);
    game.player.cash = 80_000.0;
    game.player.rate_cents = public_rate_tolerance(&game.market) + 2.0;
    game.player.reliability = 0.88;
    game.player.reputation = 70.0;
    let starting_reputation = game.player.reputation;
    let mut events = Vec::new();

    settle_utility(
        &mut game.player,
        &game.market,
        &game.macro_state,
        1.0,
        true,
        &mut events,
    );

    assert!(game.player.reputation < starting_reputation - 0.5);
    assert!(
        events
            .iter()
            .any(|event| event.contains("above public tolerance"))
    );
}

#[test]
fn high_rates_raise_rate_freeze_probability() {
    let normal = Game::with_seed(88);
    let mut high = normal.clone();
    high.player.rate_cents = public_rate_tolerance(&high.market) + 4.0;

    assert!(high.rate_freeze_probability() > normal.rate_freeze_probability() + 0.10);
}

#[test]
fn high_player_rates_create_startup_pressure() {
    let mut game = Game::with_seed(89);
    game.player.customers = 1_100.0;
    game.player.distribution_capacity = 2_000.0;
    game.player.generation_capacity_mwh = 700.0;
    game.player.rate_cents = public_rate_tolerance(&game.market) + 2.0;
    game.market.addressable_customers = 8_000.0;
    game.market.electrification = 0.45;
    for competitor in &mut game.competitors {
        competitor.customers = 150.0;
        competitor.rate_cents = game.market.standard_rate_cents;
    }

    let Some((reason, probability)) = game.startup_entry_signal() else {
        panic!("expected high player rates to create startup pressure");
    };

    assert_eq!(reason, StartupEntryReason::HighRates);
    assert!(probability >= 0.10);
}

#[test]
fn rate_freeze_blocks_rival_rate_increases() {
    let mut game = Game::with_seed(92);
    game.player.rate_cents = 14.0;
    game.player.customers = 500.0;
    game.active_shocks.push(ActiveShock {
        kind: ShockKind::RateFreeze,
        quarters_remaining: 2,
    });
    for competitor in &mut game.competitors {
        competitor.customers = 600.0;
        competitor.last_quarter_customers = 600.0;
        competitor.distribution_capacity = 600.0;
        competitor.generation_capacity_mwh = 150.0;
        competitor.rate_cents = 9.2;
    }

    let starting_rates: Vec<f64> = game.competitors.iter().map(|c| c.rate_cents).collect();
    game.advance_quarter();

    for (competitor, starting_rate) in game.competitors.iter().zip(starting_rates) {
        assert!(
            competitor.rate_cents <= starting_rate,
            "{} raised rates during a freeze",
            competitor.name
        );
    }
}

#[test]
fn cost_shock_increases_operating_cost() {
    let mut normal = Game::with_seed(93);
    let mut shocked = Game::with_seed(93);
    shocked.active_shocks.push(ActiveShock {
        kind: ShockKind::InputCostShock,
        quarters_remaining: 2,
    });

    let normal_report = normal.advance_quarter();
    let shocked_report = shocked.advance_quarter();

    assert!(
        shocked_report.operating_cost > normal_report.operating_cost * 1.05,
        "input cost shock should raise operating cost: normal {}, shocked {}",
        normal_report.operating_cost,
        shocked_report.operating_cost
    );
}

#[test]
fn equipment_fire_damage_can_apply_to_any_utility() {
    let mut game = Game::with_seed(94);
    let player_generation = game.player.generation_capacity_mwh;
    let player_reliability = game.player.reliability;
    let rival_generation = game.competitors[0].generation_capacity_mwh;
    let rival_reliability = game.competitors[0].reliability;

    let player_lost = apply_equipment_fire_damage(&mut game.player, 0.10);
    let rival_lost = apply_equipment_fire_damage(&mut game.competitors[0], 0.10);

    assert!((player_lost - player_generation * 0.10).abs() < 0.01);
    assert!((rival_lost - rival_generation * 0.10).abs() < 0.01);
    assert!(game.player.generation_capacity_mwh < player_generation);
    assert!(game.competitors[0].generation_capacity_mwh < rival_generation);
    assert!(game.player.reliability < player_reliability);
    assert!(game.competitors[0].reliability < rival_reliability);
}

#[test]
fn startup_can_enter_consolidated_high_rate_market() {
    let mut game = Game::with_seed(94);
    game.player.customers = 3_500.0;
    game.player.distribution_capacity = 5_000.0;
    game.player.generation_capacity_mwh = 1_500.0;
    game.player.rate_cents = 12.0;
    for competitor in &mut game.competitors {
        competitor.rate_cents = 12.5;
    }
    game.market.electrification = 0.9;
    game.market.addressable_customers = 12_000.0;

    let starting_competitors = game.competitors.len();
    for _ in 0..30 {
        game.advance_quarter();
        if game.competitors.len() > starting_competitors {
            return;
        }
    }
    panic!("expected at least one startup to enter the market under these conditions");
}

#[test]
fn startup_signal_detects_stagnant_market() {
    let mut game = Game::with_seed(96);
    game.quarter = 8;
    game.market.addressable_customers = 10_000.0;
    game.market.electrification = 0.11;
    game.macro_state.demand_index = -0.35;
    game.player.customers = 250.0;
    game.player.rate_cents = 10.0;
    game.player.reliability = 0.70;
    game.player.distribution_capacity = 500.0;
    game.player.generation_capacity_mwh = 200.0;
    for competitor in &mut game.competitors {
        competitor.customers = 120.0;
        competitor.rate_cents = 10.1;
        competitor.reliability = 0.70;
    }

    let Some((reason, probability)) = game.startup_entry_signal() else {
        panic!("expected stagnant market to create startup pressure");
    };

    assert_eq!(reason, StartupEntryReason::StagnantMarket);
    assert!(probability >= 0.12);
}

#[test]
fn startup_can_enter_stagnant_market_without_high_rates() {
    let mut game = Game::with_seed(97);
    game.quarter = 8;
    game.market.addressable_customers = 10_000.0;
    game.market.electrification = 0.11;
    game.market.standard_rate_cents = 10.2;
    game.macro_state.demand_index = -0.35;
    game.player.customers = 250.0;
    game.player.rate_cents = 10.0;
    game.player.reliability = 0.70;
    game.player.distribution_capacity = 500.0;
    game.player.generation_capacity_mwh = 200.0;
    for competitor in &mut game.competitors {
        competitor.customers = 120.0;
        competitor.last_quarter_customers = 120.0;
        competitor.rate_cents = 10.1;
        competitor.reliability = 0.70;
    }

    let starting_competitors = game.competitors.len();
    for _ in 0..60 {
        let mut events = Vec::new();
        game.maybe_spawn_startup(&mut events);
        if game.competitors.len() > starting_competitors {
            assert!(events.iter().any(|event| event.contains("stagnant market")));
            return;
        }
    }

    panic!("expected a stagnant-market startup entrant");
}

#[test]
fn competitors_react_to_player_undercutting() {
    let mut game = Game::with_seed(95);
    game.player.rate_cents = 8.5;
    game.player.reliability = 0.94;
    game.player.reputation = 78.0;
    game.player.distribution_capacity = 1_500.0;
    game.player.generation_capacity_mwh = 600.0;
    game.player.customers = 800.0;
    for competitor in &mut game.competitors {
        competitor.last_quarter_customers = competitor.customers;
    }

    let starting_rates: Vec<f64> = game.competitors.iter().map(|c| c.rate_cents).collect();

    let mut any_cut = false;
    for _ in 0..6 {
        game.advance_quarter();
        for (competitor, &start) in game.competitors.iter().zip(&starting_rates) {
            if competitor.rate_cents < start - 0.1 {
                any_cut = true;
            }
        }
        if any_cut {
            break;
        }
    }

    assert!(any_cut, "expected at least one competitor to cut rates");
}

#[test]
fn competitors_can_defend_below_former_rate_floor_without_chasing_zero() {
    let mut game = Game::with_seed(96);
    game.player.rate_cents = 5.0;
    game.player.customers = 1_100.0;
    game.player.last_quarter_customers = 700.0;
    game.player.reliability = 0.94;
    game.player.reputation = 82.0;

    for competitor in &mut game.competitors {
        competitor.rate_cents = 8.5;
        competitor.customers = 260.0;
        competitor.last_quarter_customers = 390.0;
        competitor.debt = 0.0;
        competitor.reliability = 0.88;
        competitor.generation_capacity_mwh = 180.0;
        competitor.distribution_capacity = 500.0;
    }

    let starting_rate = game.competitors[0].rate_cents;
    let break_even = economics::break_even_rate_cents(
        &game.competitors[0],
        &game.market,
        &game.macro_state,
        game.cost_shock_multiplier(),
    );

    let mut events = Vec::new();
    game.competitor_plans(&mut events);

    let ending_rate = game.competitors[0].rate_cents;
    assert!(
        ending_rate < 8.4,
        "rival should be able to defend below the old 8.4c floor; ended at {ending_rate}"
    );
    assert!(ending_rate < starting_rate);
    assert!(
        ending_rate > break_even * 0.80,
        "rival should not chase a zero-rate price war; ended at {ending_rate}, break-even {break_even}"
    );
}

#[test]
fn competitors_can_raise_above_old_fixed_ceiling_when_public_tolerance_supports_it() {
    let mut game = Game::with_seed(97);
    game.market.standard_rate_cents = 14.5;
    game.market.civic_patience = 0.90;
    game.market.rate_tolerance_adjustment_cents = 1.0;
    game.player.rate_cents = 15.4;
    game.player.customers = 800.0;
    game.competitors[0].rate_cents = 13.45;
    game.competitors[0].customers = 500.0;
    game.competitors[0].last_quarter_customers = 500.0;
    game.competitors[0].distribution_capacity = 510.0;
    game.competitors[0].generation_capacity_mwh =
        game.competitors[0].customers * game.market.avg_mwh_per_customer * 1.01;
    let mut events = Vec::new();

    game.competitor_plans(&mut events);

    assert!(
        game.competitors[0].rate_cents > 13.5,
        "rival rate should not be capped by the old fixed 13.5c ceiling; ended at {}",
        game.competitors[0].rate_cents
    );
}

#[test]
fn market_allocation_rewards_marketing_and_capacity() {
    let mut quiet = Game::with_seed(4);
    let mut aggressive = Game::with_seed(4);

    aggressive
        .apply_decision(Decision::Marketing { spend: 10_000.0 })
        .unwrap();
    aggressive
        .apply_decision(Decision::BuildDistribution {
            customer_capacity: DISTRIBUTION_PROJECT_CAPACITY,
        })
        .unwrap();

    quiet.advance_quarter();
    aggressive.advance_quarter();

    assert!(aggressive.player.customers > quiet.player.customers);
}

#[test]
fn allocation_scores_reward_low_rates_and_reliable_service() {
    let mut neutral = Game::with_seed(98);
    configure_equal_customer_selection_market(&mut neutral);
    let neutral_new_share = neutral.allocation_share(CustomerAllocationKind::NewConnections);

    let mut advantaged = neutral.clone();
    advantaged.player.rate_cents = 8.6;
    advantaged.player.reliability = 0.94;
    for competitor in &mut advantaged.competitors {
        competitor.rate_cents = 10.8;
        competitor.reliability = 0.76;
    }

    let new_share = advantaged.allocation_share(CustomerAllocationKind::NewConnections);
    let switched_share = advantaged.allocation_share(CustomerAllocationKind::SwitchedAccounts);

    assert!(
        new_share > neutral_new_share + 0.07,
        "low rates and high reliability should materially improve new-customer share: neutral {neutral_new_share}, advantaged {new_share}"
    );
    assert!(
        switched_share > new_share + 0.02,
        "switching customers should be more responsive than new customers: new {new_share}, switched {switched_share}"
    );
}

#[test]
fn high_utilization_reduces_customer_selection_advantage() {
    let mut healthy = Game::with_seed(99);
    configure_equal_customer_selection_market(&mut healthy);
    healthy.player.rate_cents = 8.6;
    healthy.player.reliability = 0.94;

    let mut hot = healthy.clone();
    hot.player.generation_capacity_mwh = 78.0;

    let healthy_share = healthy.allocation_share(CustomerAllocationKind::NewConnections);
    let hot_share = hot.allocation_share(CustomerAllocationKind::NewConnections);

    assert!(
        healthy_share > hot_share + 0.03,
        "reasonable utilization should beat a hot-running network: healthy {healthy_share}, hot {hot_share}"
    );
}

#[test]
fn switching_accounts_do_not_return_to_source_utility() {
    let mut game = Game::with_seed(100);
    configure_equal_customer_selection_market(&mut game);
    game.player.rate_cents = 8.6;
    game.player.reliability = 0.94;
    let starting_customers = game.player.customers;

    let player_gain =
        game.allocate_customers_from(40.0, CustomerAllocationKind::SwitchedAccounts, Some(0));

    assert_eq!(player_gain, 0.0);
    assert_eq!(game.player.customers, starting_customers);
}

fn configure_equal_customer_selection_market(game: &mut Game) {
    game.market.addressable_customers = 8_000.0;
    game.market.avg_mwh_per_customer = 0.22;
    game.market.standard_rate_cents = 10.0;
    game.player.customers = 300.0;
    game.player.generation_capacity_mwh = 220.0;
    game.player.distribution_capacity = 900.0;
    game.player.rate_cents = 10.0;
    game.player.reliability = 0.84;
    game.player.reputation = 55.0;
    game.player.marketing_momentum = 0.0;

    for competitor in &mut game.competitors {
        competitor.customers = 300.0;
        competitor.generation_capacity_mwh = 220.0;
        competitor.distribution_capacity = 900.0;
        competitor.rate_cents = 10.0;
        competitor.reliability = 0.84;
        competitor.reputation = 55.0;
        competitor.marketing_momentum = 0.0;
    }
}
