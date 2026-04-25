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
fn stock_issuance_has_no_fixed_proceeds_cap() {
    let mut game = Game::with_seed(14);
    let starting_cash = game.player.cash;

    game.apply_decision(Decision::IssueStock { amount: 120_000.0 })
        .unwrap();

    assert!(game.player.cash > starting_cash + 90_000.0);
    assert!(game.player.shares > 2_000.0);
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
    assert!(game.player.stock_price >= 1.0);
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
fn churn_compares_player_rate_to_rival_rates_not_own_weighted_average() {
    let mut game = Game::with_seed(47);
    game.player.customers = 1_800.0;
    game.player.distribution_capacity = 2_400.0;
    game.player.generation_capacity_mwh = 700.0;
    game.player.rate_cents = 8.4;
    game.player.reliability = 0.94;
    game.player.reputation = 74.0;
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
fn acquisition_removes_competitor_and_adds_scale() {
    let mut game = Game::with_seed(3);
    game.apply_decision(Decision::IssueStock { amount: 60_000.0 })
        .unwrap();
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
fn acquisition_integration_blocks_immediate_rollup() {
    let mut game = Game::with_seed(30);
    game.apply_decision(Decision::IssueStock { amount: 60_000.0 })
        .unwrap();
    game.apply_decision(Decision::Acquire {
        competitor_index: 2,
    })
    .unwrap();

    let error = game
        .apply_decision(Decision::Acquire {
            competitor_index: 1,
        })
        .unwrap_err();

    assert!(error.contains("Integration capacity"));
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
fn acquisition_absorbs_target_cash_and_debt() {
    let mut game = Game::with_seed(60);
    game.player.cash = 250_000.0;
    let target = game.competitors[2].clone();
    let starting_cash_after_payment_only = game.player.cash - game.acquisition_price(2);
    let starting_debt = game.player.debt;

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
    let target = game.competitors[0].clone();

    let acquired_customers = target.customers * 0.92;
    let expected_reputation =
        weighted_average(80.0, 100.0, target.reputation, acquired_customers * 0.65) - 1.5;
    let acquired_generation = target.generation_capacity_mwh * 0.86;
    let expected_reliability =
        weighted_average(0.92, 100.0, target.reliability, acquired_generation * 0.70) - 0.035;

    game.apply_decision(Decision::Acquire {
        competitor_index: 0,
    })
    .unwrap();

    assert!((game.player.reputation - expected_reputation).abs() < 0.001);
    assert!((game.player.reliability - expected_reliability).abs() < 0.001);
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
