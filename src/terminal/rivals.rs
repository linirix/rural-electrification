use super::render::*;
use super::*;

pub(super) fn competitor_lines(game: &Game) -> Vec<String> {
    if game.competitors.is_empty() {
        return vec!["No active rivals.".to_string()];
    }

    let mut lines = Vec::new();
    let total_connected = game.total_connected_customers().max(1.0);
    let name_width = if dashboard_width() >= 112 { 22 } else { 15 };
    for (index, competitor) in game.competitors.iter().enumerate() {
        let (acquisition_note, acquisition_tone) = acquisition_status(game, index);
        let (health_label, health_tone) = rival_health_label(competitor);

        let rate_tone = if competitor.rate_cents + 0.4 < game.player.rate_cents {
            RED
        } else if competitor.rate_cents > game.player.rate_cents + 0.4 {
            GREEN
        } else {
            CYAN
        };
        let name = shorten_plain(&competitor.name, name_width);
        lines.push(format!(
            "{}. {}   {} {}   {} {}   {} {}   {} {}",
            styled(DIM, index + 1),
            styled(BOLD, format!("{name:<name_width$}")),
            muted("share"),
            styled(
                BOLD,
                format!("{:.0}%", competitor.customers / total_connected * 100.0)
            ),
            muted("rate"),
            styled(rate_tone, format!("{:.1}c", competitor.rate_cents)),
            muted("health"),
            styled(health_tone, health_label),
            muted("buy"),
            styled(acquisition_tone, acquisition_note)
        ));
    }
    lines
}

pub(super) fn rival_overview_lines(game: &Game) -> Vec<String> {
    vec![
        format!(
            "{} {}   {} {}   {} {}",
            muted("Your rate"),
            styled(BOLD_CYAN, format!("{:.1}c", game.player.rate_cents)),
            muted("Market avg"),
            styled(CYAN, format!("{:.1}c", market_average_rate(game))),
            muted("Rivals"),
            styled(BOLD, game.competitors.len())
        ),
        format!(
            "{} {}   {} {}",
            muted("Your share"),
            styled(
                share_tone(game.market_share()),
                format!("{:.0}%", game.market_share() * 100.0)
            ),
            muted("Connected market"),
            styled(
                BOLD,
                format!("{:.0} accounts", game.total_connected_customers())
            )
        ),
        format!(
            "{} {}   {} {}   {} {}",
            muted("Cash"),
            styled(cash_tone(game.player.cash), money(game.player.cash)),
            muted("Borrowing room"),
            styled(
                borrowing_room_tone(game.borrowing_room()),
                money(game.borrowing_room())
            ),
            muted("Market cap"),
            styled(BOLD, money(game.player.market_cap()))
        ),
        "Undiligenced buys use public estimates and close risk; diligence freezes exact terms but alerts the target.".to_string(),
    ]
}

pub(super) fn rival_detail_lines(game: &Game) -> Vec<String> {
    if game.competitors.is_empty() {
        return vec!["No active rivals.".to_string()];
    }

    let total_connected = game.total_connected_customers().max(1.0);
    let mut lines = Vec::new();
    for (index, competitor) in game.competitors.iter().enumerate() {
        let share = competitor.customers / total_connected;
        let customer_delta = competitor.customers - competitor.last_quarter_customers;
        let rate_gap = competitor.rate_cents - game.player.rate_cents;
        let rate_tone = if rate_gap <= -0.4 {
            RED
        } else if rate_gap >= 0.4 {
            GREEN
        } else {
            CYAN
        };
        let (acquisition_note, acquisition_tone) = acquisition_status(game, index);
        let has_diligence = game.has_diligence(index);

        lines.push(format!(
            "{}. {}",
            styled(DIM, index + 1),
            styled(BOLD, &competitor.name)
        ));
        lines.push(format!(
            "   {} {} ({})   {} {}   {} {} ({})",
            muted("customers"),
            styled(BOLD, format!("{:.0}", competitor.customers)),
            styled(
                rival_customer_delta_tone(customer_delta),
                signed_count(customer_delta)
            ),
            muted("share"),
            styled(BOLD, format!("{:.0}%", share * 100.0)),
            muted("rate"),
            styled(rate_tone, format!("{:.1}c", competitor.rate_cents)),
            styled(rate_tone, format!("{:+.1}c vs you", rate_gap))
        ));
        if has_diligence {
            let headroom = competitor.capacity_headroom(&game.market);
            let firm_reserve = competitor.firm_generation_reserve_mwh(&game.market);
            let quarters = game
                .diligence_reports
                .iter()
                .find(|report| report.competitor_name == competitor.name)
                .map(|report| report.quarters_remaining)
                .unwrap_or(0);
            lines.push(format!(
                "   {} {}   {} {}   {} {}",
                muted("reliability"),
                styled(
                    reliability_tone(competitor.reliability),
                    format!("{:.0}%", competitor.reliability * 100.0)
                ),
                muted("headroom"),
                styled(headroom_tone(headroom), format!("{:.0}", headroom.max(0.0))),
                muted("firm reserve"),
                styled(
                    reserve_tone(firm_reserve),
                    format!("{:+.0} MWh", firm_reserve)
                )
            ));
            lines.push(format!(
                "   {} {}   {} {}   {} {}",
                muted("cash"),
                styled(cash_tone(competitor.cash), money(competitor.cash)),
                muted("debt/assets"),
                styled(
                    leverage_tone(competitor.debt_to_assets()),
                    format!("{:.0}%", competitor.debt_to_assets() * 100.0)
                ),
                muted("assets"),
                styled(BOLD, money(competitor.asset_base))
            ));
            lines.push(format!(
                "   {} {}   {} {}",
                muted("reputation"),
                styled(
                    reputation_tone(competitor.reputation),
                    format!("{:.0}", competitor.reputation)
                ),
                muted("diligence"),
                styled(GREEN, format!("{quarters}q left"))
            ));
        } else {
            let cost = game.diligence_cost(index).unwrap_or(0.0);
            let (health_label, health_tone) = rival_health_label(competitor);
            lines.push(format!(
                "   {} {}   {} {}   {} {}",
                muted("health"),
                styled(health_tone, health_label),
                muted("financials"),
                styled(DIM, "public estimates only"),
                muted("diligence"),
                styled(YELLOW, money(cost))
            ));
        }
        lines.push(format!(
            "   {} {}",
            muted("buy"),
            styled(acquisition_tone, acquisition_note)
        ));
        lines.extend(rival_acquisition_lines(game, index));
        if let Some(event) = recent_rival_event(game, &competitor.name) {
            for (line_index, wrapped) in wrap_plain_text(&event, content_width().saturating_sub(12))
                .iter()
                .enumerate()
            {
                if line_index == 0 {
                    lines.push(format!("   {} {wrapped}", muted("recent")));
                } else {
                    lines.push(format!("          {wrapped}"));
                }
            }
        }
        if index + 1 < game.competitors.len() {
            lines.push(String::new());
        }
    }

    lines
}

pub(super) fn rival_acquisition_lines(game: &Game, competitor_index: usize) -> Vec<String> {
    let Some(terms) = (if game.has_diligence(competitor_index) {
        game.acquisition_terms(competitor_index)
    } else {
        game.public_acquisition_estimate(competitor_index)
    }) else {
        return Vec::new();
    };

    let mut lines = Vec::new();
    if !game.has_diligence(competitor_index) {
        let cost = game.diligence_cost(competitor_index).unwrap_or(0.0);
        let (low, high) = public_acquisition_range(game, competitor_index);
        lines.push(format!(
            "   {} {}-{}   {} {}",
            muted("M&A estimate"),
            styled(YELLOW, money(low)),
            styled(YELLOW, money(high)),
            muted("next"),
            styled(BOLD_CYAN, format!("diligence {}", competitor_index + 1))
        ));
        lines.push(format!(
            "   {} public estimate only; buy now accepts close risk, or diligence freezes exact terms ({})",
            muted("terms"),
            money(cost)
        ));
        lines.push(acquisition_underwriting_line(game, competitor_index, false));
        return lines;
    }

    lines.push(format!(
        "   {} {}   {} {}   {} {}",
        muted("M&A price"),
        styled(BOLD_YELLOW, money(terms.price)),
        muted("net cost"),
        styled(cash_tone(terms.post_cash), money(terms.net_cash_cost)),
        muted("assume debt"),
        styled(YELLOW, money(terms.assumed_debt))
    ));
    lines.push(format!(
        "   {} {}   {} {}   {} {}",
        muted("post cash"),
        styled(cash_tone(terms.post_cash), money(terms.post_cash)),
        muted("post debt/assets"),
        styled(
            leverage_tone(terms.post_debt_to_assets),
            format!("{:.0}%", terms.post_debt_to_assets * 100.0)
        ),
        muted("post debt"),
        styled(YELLOW, money(terms.post_debt))
    ));
    lines.push(format!(
        "   {} +{:.0} cust, +{:.0} MWh/q, +{:.0} line capacity; integration haircuts included",
        muted("adds"),
        terms.acquired_customers,
        terms.acquired_generation_capacity_mwh,
        terms.acquired_distribution_capacity
    ));
    if terms.public_interest_concession > 0.0 {
        lines.push(format!(
            "   {} {} public-interest concession; post-deal share about {:.0}%",
            muted("review"),
            styled(YELLOW, money(terms.public_interest_concession)),
            terms.post_market_share * 100.0
        ));
    }
    lines.push(acquisition_underwriting_line(game, competitor_index, true));

    if game.competitors.len() == 1 {
        lines.push(format!(
            "   {} {}",
            muted("status"),
            styled(RED, "blocked: final independent rival is protected")
        ));
    } else if game.acquisition_cooldown > 0 {
        lines.push(format!(
            "   {} {}",
            muted("status"),
            styled(
                YELLOW,
                format!(
                    "unavailable for {} more quarter(s) during integration",
                    game.acquisition_cooldown
                )
            )
        ));
    } else if game.player.cash + 0.01 < terms.price {
        lines.push(format!(
            "   {} {}",
            muted("funding"),
            styled(
                BOLD_RED,
                format!(
                    "need {} more cash to close",
                    money(terms.price - game.player.cash)
                )
            )
        ));
    } else if terms.post_debt_to_assets > 0.95 {
        lines.push(format!(
            "   {} {}",
            muted("funding"),
            styled(
                BOLD_YELLOW,
                "would leave balance sheet above board leverage limit"
            )
        ));
    } else {
        lines.push(format!(
            "   {} {}",
            muted("funding"),
            styled(BOLD_GREEN, "closeable with current cash")
        ));
    }

    lines
}

fn acquisition_underwriting_line(
    game: &Game,
    competitor_index: usize,
    exact_terms: bool,
) -> String {
    let score = game
        .acquisition_stress_score(competitor_index)
        .unwrap_or(0.0);
    let label = Game::acquisition_stress_label(score);
    let note = if exact_terms {
        "terms reviewed"
    } else {
        "public file"
    };
    format!(
        "   {} {} ({note})",
        muted("underwriting"),
        styled(acquisition_underwriting_tone(score), label)
    )
}

fn acquisition_underwriting_tone(score: f64) -> &'static str {
    match Game::acquisition_stress_label(score) {
        "distressed" => BOLD_RED,
        "strained" => BOLD_YELLOW,
        "guarded" => YELLOW,
        _ => GREEN,
    }
}

pub(super) fn dashboard_competitor_lines(game: &Game) -> Vec<String> {
    let mut lines = competitor_lines(game);
    let visible_rivals = if dashboard_width() >= 112 { 4 } else { 3 };
    if game.competitors.len() > visible_rivals {
        lines.truncate(visible_rivals);
        lines.push(format!(
            "{} use 'rivals' for {} more",
            muted("More:"),
            game.competitors.len() - visible_rivals
        ));
    }
    lines
}

pub(super) fn acquisition_status(game: &Game, competitor_index: usize) -> (String, &'static str) {
    if game.competitors.len() == 1 {
        ("blocked".to_string(), RED)
    } else if game.acquisition_cooldown > 0 {
        (format!("{}q wait", game.acquisition_cooldown), YELLOW)
    } else if !game.has_diligence(competitor_index) {
        ("estimate".to_string(), YELLOW)
    } else {
        (money(game.acquisition_price(competitor_index)), CYAN)
    }
}

pub(super) fn public_acquisition_range(game: &Game, competitor_index: usize) -> (f64, f64) {
    let Some(terms) = game.public_acquisition_estimate(competitor_index) else {
        return (0.0, 0.0);
    };
    let uncertainty = (0.22 + (game.market_share() - 0.45).max(0.0) * 0.35).clamp(0.20, 0.42);
    (
        (terms.price * (1.0 - uncertainty)).max(1_000.0),
        (terms.price * (1.0 + uncertainty)).max(1_000.0),
    )
}

pub(super) fn recent_rival_event(game: &Game, competitor_name: &str) -> Option<String> {
    game.last_report.as_ref().and_then(|report| {
        report
            .events
            .iter()
            .rev()
            .find(|event| event.contains(competitor_name))
            .cloned()
    })
}

pub(super) fn signed_count(value: f64) -> String {
    format!("{:+.0}", value)
}

pub(super) fn rival_customer_delta_tone(value: f64) -> &'static str {
    if value > 20.0 {
        BOLD_RED
    } else if value > 0.0 {
        BOLD_YELLOW
    } else if value < 0.0 {
        BOLD_GREEN
    } else {
        DIM
    }
}
