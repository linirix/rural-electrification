use std::path::PathBuf;

use super::command::*;
use super::render::*;
use super::rivals::*;
use super::screens::*;
use super::*;

fn unique_test_save_dir(label: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("electrification-{label}-{nanos}"))
}

fn make_adjacent_expansion_ready(game: &mut Game) {
    game.quarter = 8;
    game.player.cash = 260_000.0;
    game.player.debt = 30_000.0;
    game.player.asset_base = 140_000.0;
    game.player.customers = 780.0;
    game.player.generation_capacity_mwh = 260.0;
    game.player.distribution_capacity = 1_050.0;
    game.player.reliability = 0.88;
}

#[test]
fn save_and_load_round_trip_from_directory() {
    let root = unique_test_save_dir("round-trip");
    let mut game = Game::with_seed(130);
    game.player.cash = 123_456.0;
    game.player.rate_cents = 5.5;
    game.advance_quarter();

    let save_message = save_game_to_dir(&game, "slot1", &root).unwrap();
    let (loaded, load_message) = load_game_from_dir("slot1", &root).unwrap();

    assert!(save_message.contains("Saved game"));
    assert!(load_message.contains("Loaded game"));
    assert_eq!(loaded.quarter, game.quarter);
    assert!((loaded.player.cash - game.player.cash).abs() < 0.01);
    assert!((loaded.player.rate_cents - 5.5).abs() < 0.001);
    assert_eq!(loaded.competitors.len(), game.competitors.len());
}

#[test]
fn save_name_rejects_paths_and_allows_simple_slots() {
    assert!(normalized_save_name("../bad").is_err());
    assert!(normalized_save_name("bad/name").is_err());
    assert!(normalized_save_name("bad\\name").is_err());
    assert!(normalized_save_name("..").is_err());
    assert_eq!(normalized_save_name("slot_1").unwrap(), "slot_1");
    assert_eq!(normalized_save_name("slot-1.json").unwrap(), "slot-1");
}

#[test]
fn rate_up_with_amount_uses_requested_delta() {
    let parts = vec!["rate", "up", "2"];
    assert_eq!(parse_rate_delta(&parts, 10.5).unwrap(), Some(2.0));
}

#[test]
fn rate_down_with_amount_uses_requested_delta() {
    let parts = vec!["rate", "down", "1.5"];
    assert_eq!(parse_rate_delta(&parts, 10.5).unwrap(), Some(-1.5));
}

#[test]
fn bare_rate_number_sets_target_rate() {
    let parts = vec!["rate", "9.5"];
    assert_eq!(parse_rate_delta(&parts, 10.5).unwrap(), Some(-1.0));
}

#[test]
fn rate_command_accepts_below_former_floor() {
    let mut game = Game::with_seed(100);
    game.player.rate_cents = 7.2;

    match handle_command(&mut game, "rate 6.0") {
        CommandResult::Continue(message) => assert!(message.contains("Cut the rate")),
        _ => panic!("rate command should apply a below-floor target"),
    }

    assert!((game.player.rate_cents - 6.0).abs() < 0.001);
}

#[test]
fn rate_preview_does_not_floor_below_seven_cents() {
    let mut game = Game::with_seed(100);
    game.player.rate_cents = 7.2;

    match handle_command(&mut game, "preview rate 6.0") {
        CommandResult::Preview(lines) => {
            assert!(lines.iter().any(|line| line.contains("6.0c/kWh")));
        }
        _ => panic!("preview rate should show the below-floor target"),
    }

    assert!((game.player.rate_cents - 7.2).abs() < 0.001);
}

#[test]
fn rate_preview_warns_when_above_public_ceiling() {
    let mut game = Game::with_seed(100);
    game.player.rate_cents =
        public_rate_tolerance(&game.market) + MAX_PUBLIC_RATE_PREMIUM_CENTS - 0.2;

    match handle_command(&mut game, "preview rate up 0.4") {
        CommandResult::Preview(lines) => {
            assert!(lines.iter().any(|line| line.contains("would be rejected")));
            assert!(lines.iter().any(|line| line.contains("public ceiling")));
        }
        _ => panic!("preview rate should warn before exceeding the public ceiling"),
    }
}

#[test]
fn rate_preview_warns_when_below_sustainable_cost() {
    let mut game = Game::with_seed(100);

    match handle_command(&mut game, "preview rate 0.5") {
        CommandResult::Preview(lines) => {
            assert!(lines.iter().any(|line| line.contains("Sustainability")));
            assert!(lines.iter().any(|line| line.contains("break-even")));
        }
        _ => panic!("preview rate should warn about unsustainable pricing"),
    }
}

#[test]
fn rate_command_rejects_above_public_ceiling_without_mutating() {
    let mut game = Game::with_seed(100);
    game.player.rate_cents =
        public_rate_tolerance(&game.market) + MAX_PUBLIC_RATE_PREMIUM_CENTS - 0.2;
    let starting_rate = game.player.rate_cents;

    match handle_command(&mut game, "rate up 0.4") {
        CommandResult::Continue(message) => {
            assert!(message.contains("Cannot do that"));
            assert!(message.contains("public tolerance"));
        }
        _ => panic!("rate command should reject an above-ceiling target"),
    }

    assert!((game.player.rate_cents - starting_rate).abs() < 0.001);
}

#[test]
fn rate_command_allows_cuts_from_above_public_ceiling() {
    let mut game = Game::with_seed(100);
    game.player.rate_cents =
        public_rate_tolerance(&game.market) + MAX_PUBLIC_RATE_PREMIUM_CENTS + 1.0;
    let starting_rate = game.player.rate_cents;

    match handle_command(&mut game, "rate down 0.4") {
        CommandResult::Continue(message) => assert!(message.contains("Cut the rate")),
        _ => panic!("rate command should allow cuts from above the public ceiling"),
    }

    assert!((game.player.rate_cents - (starting_rate - 0.4)).abs() < 0.001);
}

#[test]
fn next_advances_the_quarter() {
    let mut game = Game::with_seed(100);

    match handle_command(&mut game, "next") {
        CommandResult::Advanced(report) => assert_eq!(report.label, "Year 1 Q1"),
        _ => panic!("next should advance the quarter"),
    }
    assert_eq!(game.quarter, 1);
}

#[test]
fn n_alias_advances_the_quarter() {
    let mut game = Game::with_seed(101);

    match handle_command(&mut game, "n") {
        CommandResult::Advanced(report) => assert_eq!(report.label, "Year 1 Q1"),
        _ => panic!("n should advance the quarter"),
    }
    assert_eq!(game.quarter, 1);
}

#[test]
fn old_advance_command_is_not_the_turn_command() {
    let mut game = Game::with_seed(102);

    match handle_command(&mut game, "advance") {
        CommandResult::Continue(message) => assert!(message.contains("Unknown command")),
        _ => panic!("advance should no longer finish the quarter"),
    }
    assert_eq!(game.quarter, 0);
}

#[test]
fn board_command_shows_objectives() {
    let mut game = Game::with_seed(104);

    match handle_command(&mut game, "board") {
        CommandResult::ShowBoard => {}
        _ => panic!("board should show objectives"),
    }
}

#[test]
fn continue_command_resumes_after_formal_review() {
    let mut game = Game::with_seed(116);
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

    match handle_command(&mut game, "continue") {
        CommandResult::Continue(message) => assert!(message.contains("Continuing after")),
        _ => panic!("continue should clear a continuable formal review"),
    }

    assert!(game.outcome.is_none());
    assert!(game.review_completed);
}

#[test]
fn rivals_command_shows_rival_screen() {
    let mut game = Game::with_seed(113);

    match handle_command(&mut game, "rivals") {
        CommandResult::ShowRivals => {}
        _ => panic!("rivals should show rival detail"),
    }
}

#[test]
fn report_command_shows_last_quarter_screen() {
    let mut game = Game::with_seed(119);
    game.advance_quarter();

    match handle_command(&mut game, "report") {
        CommandResult::ShowReport => {}
        _ => panic!("report should show the last-quarter detail screen"),
    }
}

#[test]
fn stable_dashboard_panels_keep_fixed_heights() {
    let mut game = Game::with_seed(120);
    assert_eq!(
        stable_panel_lines(last_quarter_lines(&game), 5, "report for full detail").len(),
        5
    );
    assert_eq!(
        stable_panel_lines(signal_lines(&game), 6, "board for remaining risks").len(),
        6
    );
    assert_eq!(
        stable_panel_lines(project_lines(&game), 5, "board for planning context").len(),
        5
    );
    assert_eq!(
        stable_panel_lines(compact_command_lines(&game), 5, "help for all commands").len(),
        5
    );

    game.advance_quarter();
    game.pending_projects.push(crate::Project {
        name: "Test generation".to_string(),
        kind: crate::ProjectKind::Generation {
            capacity_mwh: 500.0,
        },
        quarters_remaining: 2,
    });
    game.pending_projects.push(crate::Project {
        name: "Test distribution".to_string(),
        kind: crate::ProjectKind::Distribution {
            customer_capacity: 700.0,
        },
        quarters_remaining: 1,
    });
    game.acquisition_cooldown = 3;
    assert_eq!(
        stable_panel_lines(last_quarter_lines(&game), 5, "report for full detail").len(),
        5
    );
    assert_eq!(
        stable_panel_lines(project_lines(&game), 5, "board for planning context").len(),
        5
    );
}

#[test]
fn compact_commands_show_build_sizes_with_spaces() {
    let game = Game::with_seed(120);
    let commands = compact_command_lines(&game).join(" ");

    assert!(commands.contains("gen 400"));
    assert!(commands.contains("lines 600"));
    assert!(!commands.contains("gen400"));
    assert!(!commands.contains("lines600"));
}

#[test]
fn start_screen_mentions_objectives_and_core_commands() {
    let game = Game::with_seed(120);
    let text = [
        start_screen_overview_lines(&game),
        start_screen_metric_lines(),
        start_screen_command_lines(&game),
        start_screen_begin_lines(),
    ]
    .concat()
    .join(" ");

    assert!(text.contains("Metro Consolidated"));
    assert!(text.contains("Year 5 review"));
    assert!(text.contains("Year 10 regional mandate"));
    assert!(text.contains("reliable service"));
    assert!(text.contains("preview"));
    assert!(text.contains("next / n"));
    assert!(text.contains("rivals / board"));
    assert!(text.contains("save / load"));
    assert!(text.contains("Press Return"));
}

#[test]
fn empty_input_returns_from_subscreens_to_dashboard() {
    assert!(should_return_to_dashboard("\n", TerminalScreen::Subscreen));
    assert!(should_return_to_dashboard(
        "\r\n",
        TerminalScreen::Subscreen
    ));
    assert!(should_return_to_dashboard(" \n", TerminalScreen::Subscreen));
    assert!(should_return_to_dashboard(
        "   \r\n",
        TerminalScreen::Subscreen
    ));
    assert!(!should_return_to_dashboard(
        " status\n",
        TerminalScreen::Subscreen
    ));
    assert!(!should_return_to_dashboard(
        " \n",
        TerminalScreen::Dashboard
    ));
}

#[test]
fn rival_overview_shows_acquisition_financing_context() {
    let game = Game::with_seed(113);
    let lines = rival_overview_lines(&game);

    assert!(lines.iter().any(|line| line.contains("Cash")));
    assert!(lines.iter().any(|line| line.contains("Borrowing room")));
    assert!(lines.iter().any(|line| line.contains("Market cap")));
}

#[test]
fn dashboard_rival_lines_show_public_health_not_exact_operations() {
    let game = Game::with_seed(113);
    let lines = dashboard_competitor_lines(&game);

    assert!(lines.iter().any(|line| line.contains("health")));
    assert!(lines.iter().any(|line| line.contains("share")));
    assert!(!lines.iter().any(|line| line.contains("rel")));
}

#[test]
fn dashboard_rival_table_keeps_health_and_buy_columns_aligned() {
    let game = Game::with_seed(113);
    let lines = dashboard_competitor_lines(&game);
    let header = lines.first().expect("dashboard competitor header");
    let health_column = visible_width(
        &header[..header
            .find("health")
            .expect("header should include health column")],
    );
    let buy_column = visible_width(
        &header[..header
            .find("buy")
            .expect("header should include buy column")],
    );

    for line in lines.iter().skip(1).take(3) {
        let health_index = ["healthy", "strained", "vulnerable"]
            .iter()
            .filter_map(|label| line.find(label))
            .next()
            .expect("rival row should include health label");
        let buy_index = line
            .find("estimate")
            .expect("default row should show estimate");

        assert_eq!(visible_width(&line[..health_index]), health_column);
        assert_eq!(visible_width(&line[..buy_index]), buy_column);
    }
}

#[test]
fn rival_detail_includes_acquisition_economics() {
    let mut game = Game::with_seed(114);
    game.player.cash = 100_000.0;
    game.apply_decision(Decision::Diligence {
        competitor_index: 0,
    })
    .unwrap();
    let lines = rival_detail_lines(&game);

    assert!(lines.iter().any(|line| line.contains("M&A price")));
    assert!(lines.iter().any(|line| line.contains("net cost")));
    assert!(lines.iter().any(|line| line.contains("assume debt")));
    assert!(lines.iter().any(|line| line.contains("post debt/assets")));
    assert!(lines.iter().any(|line| line.contains("reliability")));
    assert!(lines.iter().any(|line| line.contains("headroom")));
    assert!(lines.iter().any(|line| line.contains("firm reserve")));
    assert!(lines.iter().any(|line| line.contains("cash")));
    assert!(lines.iter().any(|line| line.contains("assets")));
    assert!(lines.iter().any(|line| line.contains("reputation")));
    assert!(
        lines
            .iter()
            .any(|line| line.contains("integration haircuts included"))
    );
    assert!(
        lines
            .iter()
            .any(|line| line.contains("post-close reliability"))
    );
    assert!(lines.iter().any(|line| line.contains("integration burden")));
    assert!(lines.iter().any(|line| line.contains("underwriting")));
}

#[test]
fn rival_detail_hides_exact_deal_terms_before_diligence() {
    let game = Game::with_seed(117);
    let lines = rival_detail_lines(&game);

    assert!(lines.iter().any(|line| line.contains("M&A estimate")));
    assert!(
        lines
            .iter()
            .any(|line| line.contains("public estimates only"))
    );
    assert!(lines.iter().any(|line| line.contains("risk clues")));
    assert!(lines.iter().any(|line| line.contains("service quality")));
    assert!(!lines.iter().any(|line| line.contains("post debt/assets")));
    assert!(!lines.iter().any(|line| line.contains("reliability")));
    assert!(!lines.iter().any(|line| line.contains("headroom")));
    assert!(!lines.iter().any(|line| line.contains("firm reserve")));
    assert!(!lines.iter().any(|line| line.contains("debt/assets")));
    assert!(!lines.iter().any(|line| line.contains("reputation")));
    assert!(!lines.iter().any(|line| line.contains("assets")));
    assert!(!lines.iter().any(|line| line.contains("cash ")));
}

#[test]
fn rival_detail_uses_cash_to_close_for_funding_status() {
    let mut game = Game::with_seed(115);
    game.player.cash = 100_000.0;
    game.apply_decision(Decision::Diligence {
        competitor_index: 0,
    })
    .unwrap();
    let terms = game.acquisition_terms(0).unwrap();
    game.player.cash = (terms.price - 1_000.0).max(0.0);

    let lines = rival_acquisition_lines(&game, 0);

    assert!(lines.iter().any(|line| line.contains("need")));
    assert!(lines.iter().any(|line| line.contains("cash to close")));
}

#[test]
fn acquisition_preview_without_diligence_shows_public_range() {
    let mut game = Game::with_seed(116);

    match handle_command(&mut game, "preview buy 1") {
        CommandResult::Preview(lines) => {
            assert!(lines.iter().any(|line| line.contains("public estimate")));
            assert!(lines.iter().any(|line| line.contains("close risk")));
            assert!(lines.iter().any(|line| line.contains("diligence reveals")));
            assert!(lines.iter().any(|line| line.contains("Underwriting")));
            assert!(lines.iter().any(|line| line.contains("service quality")));
            assert!(lines.iter().any(|line| line.contains("liquidity buffer")));
        }
        _ => panic!("preview buy should show preview output"),
    }
}

#[test]
fn acquisition_preview_with_diligence_shows_integration_risk() {
    let mut game = Game::with_seed(116);
    game.apply_decision(Decision::Diligence {
        competitor_index: 0,
    })
    .unwrap();

    match handle_command(&mut game, "preview buy 1") {
        CommandResult::Preview(lines) => {
            assert!(lines.iter().any(|line| line.contains("integration risk")));
            assert!(lines.iter().any(|line| line.contains("underwriting")));
            assert!(lines.iter().any(|line| line.contains("post debt/assets")));
            assert!(
                lines
                    .iter()
                    .any(|line| line.contains("post-close reliability"))
            );
            assert!(lines.iter().any(|line| line.contains("review standard")));
        }
        _ => panic!("preview buy should show preview output"),
    }
}

#[test]
fn diligence_preview_shows_acquisition_terms_that_will_be_locked_in() {
    let mut game = Game::with_seed(151);

    match handle_command(&mut game, "preview diligence 1") {
        CommandResult::Preview(lines) => {
            assert!(lines.iter().any(|line| line.contains("inspect")));
            assert!(lines.iter().any(|line| line.contains("Will lock")));
            assert!(lines.iter().any(|line| line.contains("net cost")));
            assert!(lines.iter().any(|line| line.contains("assumes debt")));
            assert!(lines.iter().any(|line| line.contains("post debt/assets")));
            assert!(
                lines
                    .iter()
                    .any(|line| line.contains("post-close reliability"))
            );
        }
        _ => panic!("preview diligence should show preview output"),
    }
}

#[test]
fn maint_alias_runs_maintenance() {
    let mut game = Game::with_seed(103);
    let starting_cash = game.player.cash;
    let starting_reliability = game.player.reliability;

    match handle_command(&mut game, "maint 4500") {
        CommandResult::Continue(message) => assert!(message.contains("reliability")),
        _ => panic!("maint should apply maintenance"),
    }
    assert!(game.player.cash < starting_cash);
    assert!(game.player.reliability > starting_reliability);
}

#[test]
fn issue_single_word_runs_stock_issuance() {
    let mut game = Game::with_seed(109);
    let starting_cash = game.player.cash;
    let starting_shares = game.player.shares;

    match handle_command(&mut game, "issue 10000") {
        CommandResult::Continue(message) => assert!(message.contains("Issued")),
        _ => panic!("issue should apply stock issuance"),
    }

    assert!(game.player.cash > starting_cash);
    assert!(game.player.shares > starting_shares);
}

#[test]
fn invalid_money_amounts_are_rejected_without_defaulting() {
    let mut game = Game::with_seed(118);
    let starting_cash = game.player.cash;
    let starting_debt = game.player.debt;

    match handle_command(&mut game, "borrow bananas") {
        CommandResult::Continue(message) => assert!(message.contains("borrow [amount|max]")),
        _ => panic!("invalid borrow amount should return an error message"),
    }

    assert_eq!(game.player.cash, starting_cash);
    assert_eq!(game.player.debt, starting_debt);
}

#[test]
fn nonpositive_money_amounts_are_rejected_without_minimum_spend() {
    let mut game = Game::with_seed(120);
    let starting_cash = game.player.cash;
    let starting_reputation = game.player.reputation;

    match handle_command(&mut game, "marketing -100") {
        CommandResult::Continue(message) => assert!(message.contains("positive amount")),
        _ => panic!("negative marketing spend should return an error message"),
    }

    assert_eq!(game.player.cash, starting_cash);
    assert_eq!(game.player.reputation, starting_reputation);
}

#[test]
fn preview_invalid_money_amount_stays_with_failing_command() {
    let mut game = Game::with_seed(119);

    match handle_command(&mut game, "preview issue bananas borrow 1000") {
        CommandResult::Preview(lines) => {
            assert!(lines.iter().any(|line| line.contains("Cannot preview")));
            assert!(lines.iter().any(|line| line.contains("issue [amount]")));
            assert!(!lines.iter().any(|line| line.contains("Step 2")));
        }
        _ => panic!("invalid preview amount should show preview error"),
    }
}

#[test]
fn borrow_max_uses_remaining_borrowing_room() {
    let mut game = Game::with_seed(110);
    let starting_cash = game.player.cash;
    let starting_debt = game.player.debt;
    let room = game.borrowing_room();

    match handle_command(&mut game, "borrow max") {
        CommandResult::Continue(message) => assert!(message.contains("Borrowed")),
        _ => panic!("borrow max should apply borrowing"),
    }

    assert!((game.player.cash - (starting_cash + room)).abs() < 0.01);
    assert!((game.player.debt - (starting_debt + room)).abs() < 0.01);
}

#[test]
fn repay_max_uses_lesser_of_cash_and_debt() {
    let mut game = Game::with_seed(111);
    game.player.cash = 7_500.0;
    game.player.debt = 20_000.0;

    match handle_command(&mut game, "repay max") {
        CommandResult::Continue(message) => assert!(message.contains("Repaid")),
        _ => panic!("repay max should apply repayment"),
    }

    assert!((game.player.cash - 0.0).abs() < 0.01);
    assert!((game.player.debt - 12_500.0).abs() < 0.01);
}

#[test]
fn preview_debt_does_not_mutate_game() {
    let mut game = Game::with_seed(105);
    let starting_cash = game.player.cash;
    let starting_debt = game.player.debt;

    match handle_command(&mut game, "preview debt 10000") {
        CommandResult::Preview(lines) => {
            assert!(lines.iter().any(|line| line.contains("Would succeed")));
            assert!(lines.iter().any(|line| line.contains("Cash")));
            assert!(lines.iter().any(|line| line.contains("Debt")));
            assert!(lines.iter().any(|line| line.contains("Interest/q")));
        }
        _ => panic!("preview debt should show preview output"),
    }

    assert_eq!(game.player.cash, starting_cash);
    assert_eq!(game.player.debt, starting_debt);
}

#[test]
fn preview_can_chain_borrowing_and_acquisition_without_mutating_game() {
    let mut game = Game::with_seed(107);
    game.player.cash = 100_000.0;
    let starting_cash = game.player.cash;
    let starting_debt = game.player.debt;
    let starting_competitors = game.competitors.len();

    match handle_command(&mut game, "preview borrow 10000 diligence 3 buy 3") {
        CommandResult::Preview(lines) => {
            assert!(lines.iter().any(|line| line.contains("Step 1")));
            assert!(lines.iter().any(|line| line.contains("Step 2")));
            assert!(lines.iter().any(|line| line.contains("Step 3")));
            assert!(lines.iter().any(|line| line.contains("Interest/q")));
            assert!(lines.iter().any(|line| line.contains("Rivals")));
        }
        _ => panic!("chained preview should show preview output"),
    }

    assert_eq!(game.player.cash, starting_cash);
    assert_eq!(game.player.debt, starting_debt);
    assert_eq!(game.competitors.len(), starting_competitors);
}

#[test]
fn preview_chain_keeps_stock_buyback_as_one_transaction() {
    let mut game = Game::with_seed(108);
    game.player.cash = 60_000.0;

    match handle_command(&mut game, "preview issue 5000 buyback 5000 borrow 10000") {
        CommandResult::Preview(lines) => {
            assert!(
                lines
                    .iter()
                    .any(|line| line.contains("Step 1") && line.contains("issue 5000"))
            );
            assert!(
                lines
                    .iter()
                    .any(|line| line.contains("Step 2") && line.contains("buyback 5000"))
            );
            assert!(
                lines
                    .iter()
                    .any(|line| line.contains("Step 3") && line.contains("borrow 10000"))
            );
            assert!(!lines.iter().any(|line| line.contains("Step 4")));
        }
        _ => panic!("financial command chain should show preview output"),
    }
}

#[test]
fn preview_chain_keeps_max_with_borrow_and_repay() {
    let mut game = Game::with_seed(112);

    match handle_command(&mut game, "preview borrow max repay max") {
        CommandResult::Preview(lines) => {
            assert!(
                lines
                    .iter()
                    .any(|line| line.contains("Step 1") && line.contains("borrow max"))
            );
            assert!(
                lines
                    .iter()
                    .any(|line| line.contains("Step 2") && line.contains("repay max"))
            );
            assert!(!lines.iter().any(|line| line.contains("Step 3")));
        }
        _ => panic!("max financial command chain should show preview output"),
    }
}

#[test]
fn expand_command_starts_adjacent_expansion_project() {
    let mut game = Game::with_seed(121);
    make_adjacent_expansion_ready(&mut game);
    let starting_cash = game.player.cash;

    match handle_command(&mut game, "expand") {
        CommandResult::Continue(message) => assert!(message.contains("adjacent territory")),
        _ => panic!("expand should apply adjacent territory entry"),
    }

    assert!(game.player.cash < starting_cash);
    assert!(game.adjacent_expansion_pending());
}

#[test]
fn preview_expand_shows_cost_and_does_not_mutate_game() {
    let mut game = Game::with_seed(122);
    make_adjacent_expansion_ready(&mut game);
    let starting_cash = game.player.cash;
    let starting_projects = game.pending_projects.len();

    match handle_command(&mut game, "preview expand") {
        CommandResult::Preview(lines) => {
            assert!(lines.iter().any(|line| line.contains("adjacent territory")));
            assert!(lines.iter().any(|line| line.contains("Would succeed")));
            assert!(lines.iter().any(|line| line.contains("Projects")));
        }
        _ => panic!("preview expand should show preview output"),
    }

    assert_eq!(game.player.cash, starting_cash);
    assert_eq!(game.pending_projects.len(), starting_projects);
}

#[test]
fn regional_status_lines_show_post_review_targets_and_integration() {
    let mut game = Game::with_seed(146);
    game.review_completed = true;
    game.adjacent_expansions = 1;
    game.regional_integration = 0.26;

    let lines = regional_status_lines(&game);
    let joined = lines.join(" ");

    assert!(joined.contains("Territories"));
    assert!(joined.contains("Integration"));
    assert!(joined.contains("Year 10"));
}

#[test]
fn regional_status_visible_pre_review_with_planning_hint() {
    let game = Game::with_seed(149);
    assert_eq!(game.quarter, 0);
    assert!(!game.review_completed);

    assert!(should_show_regional_status(&game));

    let lines = regional_status_lines(&game);
    let joined = lines.join(" ");

    assert!(joined.contains("Year 10"));
    assert!(joined.contains("Plan ahead"));
    assert!(joined.contains("Y5 review"));
}

#[test]
fn regional_status_hidden_after_mandate_completed() {
    let mut game = Game::with_seed(150);
    game.regional_mandate_completed = true;

    assert!(!should_show_regional_status(&game));
}

#[test]
fn post_review_dashboard_guides_regional_expansion_work() {
    let mut game = Game::with_seed(148);
    make_adjacent_expansion_ready(&mut game);
    game.review_completed = true;
    game.quarter = 20;
    game.regional_integration = 0.22;

    let status = regional_status_lines(&game).join(" ");
    let commands = command_footer_lines(&game).join(" ");

    assert!(status.contains("Regional share"));
    assert!(status.contains("Territories"));
    assert!(status.contains("Integration"));
    assert!(commands.contains("expand"));
    assert!(commands.contains("regional integration"));
}

#[test]
fn regional_mandate_lines_include_expansion_and_financial_targets() {
    let mut game = Game::with_seed(147);
    game.review_completed = true;
    game.adjacent_expansions = 2;
    game.regional_integration = 0.05;

    let lines = regional_mandate_lines(&game);
    let joined = lines.join(" ");

    assert!(joined.contains("Territories"));
    assert!(joined.contains("Debt/assets"));
    assert!(joined.contains("Integration"));
}

#[test]
fn preview_maint_alias_does_not_mutate_game() {
    let mut game = Game::with_seed(106);
    let starting_cash = game.player.cash;
    let starting_reliability = game.player.reliability;

    match handle_command(&mut game, "quote maint 4500") {
        CommandResult::Preview(lines) => {
            assert!(lines.iter().any(|line| line.contains("Would succeed")));
            assert!(lines.iter().any(|line| line.contains("Reliability")));
        }
        _ => panic!("quote maint should show preview output"),
    }

    assert_eq!(game.player.cash, starting_cash);
    assert_eq!(game.player.reliability, starting_reliability);
}

#[test]
fn preview_maintenance_at_cap_fails_without_mutating_game() {
    let mut game = Game::with_seed(106);
    game.player.reliability = 0.98;
    let starting_cash = game.player.cash;

    match handle_command(&mut game, "preview maint 4500") {
        CommandResult::Preview(lines) => {
            assert!(lines.iter().any(|line| line.contains("already at 98%")));
            assert!(lines.iter().any(|line| line.contains("Would fail")));
        }
        _ => panic!("preview maint should show preview output"),
    }

    assert_eq!(game.player.cash, starting_cash);
    assert_eq!(game.player.reliability, 0.98);
}

#[test]
fn visible_width_ignores_ansi_escape_sequences() {
    let line = format!("{DIM}Cash{RESET} {BOLD_GREEN}$34.0k{RESET}");
    assert_eq!(visible_width(&line), "Cash $34.0k".len());
}

#[test]
fn fit_line_preserves_ansi_sequences_without_counting_them() {
    let line = format!("{BOLD_CYAN}issue [amt]{RESET} no fixed cap");
    let fitted = fit_line(&line, 24);
    assert_eq!(visible_width(&fitted), 24);
    assert!(fitted.contains(BOLD_CYAN));
}

#[test]
fn ansi_can_be_disabled_by_environment_convention() {
    assert!(should_use_ansi(false, Some("xterm-256color")));
    assert!(!should_use_ansi(true, Some("xterm-256color")));
    assert!(!should_use_ansi(false, Some("dumb")));
}

#[test]
fn render_box_preserves_requested_visible_width() {
    let (column_width, _) = paired_column_widths(DEFAULT_SCREEN_WIDTH);
    let lines = vec![format!(
        "{} {}",
        styled(BOLD_GREEN, "Cash"),
        styled(BOLD_CYAN, "$34.0k")
    )];

    for line in render_box("Capital", &lines, column_width) {
        assert_eq!(visible_width(&line), column_width);
    }
}

#[test]
fn dashboard_width_uses_wider_default_and_clamps_terminal_columns() {
    assert_eq!(dashboard_width_for_columns(None), DEFAULT_SCREEN_WIDTH);
    assert_eq!(dashboard_width_for_columns(Some(72)), MIN_SCREEN_WIDTH);
    assert_eq!(dashboard_width_for_columns(Some(160)), MAX_SCREEN_WIDTH);
}

#[test]
fn command_footer_shows_undiligenced_buy_as_an_option() {
    let game = Game::with_seed(118);
    let joined = command_footer_lines(&game).join("\n");

    assert!(joined.contains("buy "));
    assert!(joined.contains("public est"));
    assert!(joined.contains("close risk"));
}

#[test]
fn financial_lines_show_break_even_and_interest_coverage() {
    let mut game = Game::with_seed(117);
    game.last_report = Some(QuarterReport {
        label: "Year 2 Q1".to_string(),
        revenue: 12_000.0,
        operating_cost: 8_000.0,
        interest: 1_000.0,
        profit: 3_000.0,
        new_customers: 40.0,
        lost_customers: 4.0,
        lost_customer_rate: 0.01,
        market_share: game.market_share(),
        prior_market_share: game.market_share(),
        attributions: Vec::new(),
        events: Vec::new(),
    });

    let joined = financial_lines(&game).join("\n");

    assert!(joined.contains("Break-even"));
    assert!(joined.contains("Coverage"));
}

#[test]
fn signal_panel_keeps_critical_items_when_crowded() {
    let mut game = Game::with_seed(117);
    game.active_shocks.push(ActiveShock {
        kind: ShockKind::InputCostShock,
        quarters_remaining: 2,
    });
    game.last_report = Some(QuarterReport {
        label: "Year 3 Q1".to_string(),
        revenue: 0.0,
        operating_cost: 0.0,
        interest: 0.0,
        profit: 0.0,
        new_customers: 0.0,
        lost_customers: 10.0,
        lost_customer_rate: 0.05,
        market_share: 0.40,
        prior_market_share: 0.42,
        attributions: Vec::new(),
        events: Vec::new(),
    });
    game.player.rate_cents = public_rate_tolerance(&game.market) + 3.0;
    game.player.customers = 1_000.0;
    game.player.distribution_capacity = 1_010.0;
    game.player.generation_capacity_mwh = 200.0;
    game.player.reliability = 0.68;
    game.player.asset_base = 100_000.0;
    game.player.debt = 90_000.0;

    let lines = signal_lines(&game);
    let joined = lines.join("\n");

    assert_eq!(lines.len(), 6);
    assert!(joined.contains("Shock"));
    assert!(joined.contains("Rate"));
    assert!(joined.contains("Capacity"));
    assert!(joined.contains("Generation"));
    assert!(joined.contains("Debt"));
    assert!(joined.contains("Reliability"));
    assert!(!joined.contains("Trend"));
}

#[test]
fn dashboard_surfaces_integration_burden() {
    let mut game = Game::with_seed(151);
    game.integration_strain = 0.34;
    game.acquisition_cooldown = 3;

    let signals = signal_lines(&game).join("\n");
    let pipeline = project_lines(&game).join("\n");

    assert!(signals.contains("Integration"));
    assert!(signals.contains("heavy operating burden"));
    assert!(pipeline.contains("3q left"));
    assert!(pipeline.contains("high burden"));
}

#[test]
fn outcome_details_wrap_within_box_width() {
    let details = "Repeated outages pushed regulators and lenders to move the company into managed restructuring.";

    let lines = wrap_plain_text(details, 42);

    assert!(lines.len() > 1);
    assert!(lines.iter().all(|line| visible_width(line) <= 42));
}
