use super::render::*;
use super::*;

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
        "Undiligenced buys use public estimates and close risk; diligence locks a time-limited purchase option.".to_string(),
    ]
}

pub(super) fn rival_detail_lines(game: &Game) -> Vec<String> {
    if game.competitors.is_empty() {
        return vec!["No active rivals.".to_string()];
    }

    let total_connected = game.total_connected_customers().max(1.0);
    let mut lines = Vec::new();
    let ranked = ranked_competitor_indices(game);
    for (rank_position, index) in ranked.iter().copied().enumerate() {
        let competitor = &game.competitors[index];
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
        let has_diligence = game.has_diligence(index);

        lines.push(format!(
            "{}. {}",
            styled(DIM, rank_position + 1),
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
            lines.push(diligence_option_line(game, index, quarters));
            lines.push(format!(
                "   {} {}",
                muted("reputation"),
                styled(
                    reputation_tone(competitor.reputation),
                    format!("{:.0}", competitor.reputation)
                )
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
        if rank_position + 1 < ranked.len() {
            lines.push(String::new());
        }
    }

    lines
}

fn diligence_option_line(game: &Game, competitor_index: usize, quarters_remaining: u32) -> String {
    let price = game
        .acquisition_terms(competitor_index)
        .map(|terms| terms.price)
        .or_else(|| game.acquisition_price(competitor_index))
        .unwrap_or(0.0);
    format!(
        "   {} {}   {} {}   {} {}",
        muted("diligence"),
        styled(GREEN, "complete"),
        muted("option price"),
        styled(BOLD_YELLOW, money(price)),
        muted("window"),
        styled(GREEN, diligence_window_label(quarters_remaining))
    )
}

fn diligence_window_label(quarters_remaining: u32) -> String {
    if quarters_remaining == 1 {
        "1q left".to_string()
    } else {
        format!("{quarters_remaining}q left")
    }
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
        let (low, high) = public_acquisition_range(game, competitor_index);
        let rank = competitor_rank(game, competitor_index).unwrap_or(competitor_index + 1);
        lines.push(format!(
            "   {} {}-{}   {} {}",
            muted("M&A estimate"),
            styled(YELLOW, money(low)),
            styled(YELLOW, money(high)),
            muted("next"),
            styled(BOLD_CYAN, format!("diligence {rank}"))
        ));
        lines.push(format!(
            "   {} net cost {}, est post cash {}, est leverage {}",
            muted("estimate"),
            styled(YELLOW, money(terms.net_cash_cost)),
            styled(cash_tone(terms.post_cash), money(terms.post_cash)),
            styled(
                leverage_tone(terms.post_debt_to_assets),
                format!("{:.0}%", terms.post_debt_to_assets * 100.0)
            )
        ));
        if let Some(competitor) = game.competitors.get(competitor_index) {
            lines.push(public_acquisition_service_line(game, competitor, &terms));
        }
        lines.push(acquisition_funding_line(game, &terms, false));
        if let Some(line) = acquisition_rate_anchor_line(game, competitor_index) {
            lines.push(line);
        }
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
    if let Some(competitor) = game.competitors.get(competitor_index) {
        lines.push(acquisition_service_line(game, competitor, &terms));
    }
    if let Some(line) = acquisition_rate_anchor_line(game, competitor_index) {
        lines.push(line);
    }
    if terms.public_interest_concession > 0.0 {
        lines.push(format!(
            "   {} {} public-interest concession; post-deal share about {:.0}%",
            muted("review"),
            styled(YELLOW, money(terms.public_interest_concession)),
            terms.post_market_share * 100.0
        ));
    }

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
    } else {
        lines.push(acquisition_funding_line(game, &terms, true));
    }

    lines
}

fn public_acquisition_service_line(
    game: &Game,
    competitor: &Utility,
    terms: &AcquisitionTerms,
) -> String {
    let (health_label, health_tone) = rival_health_label(competitor);
    let burden = acquisition_projected_integration_burden(game, terms);
    format!(
        "   {} public health {}; integration burden {}; exact service quality requires diligence",
        styled(acquisition_integration_burden_tone(burden), "service"),
        styled(health_tone, health_label),
        styled(
            acquisition_integration_burden_tone(burden),
            acquisition_integration_burden_label(burden)
        )
    )
}

fn acquisition_service_line(game: &Game, competitor: &Utility, terms: &AcquisitionTerms) -> String {
    let projected_reliability = acquisition_projected_reliability(game, competitor, terms);
    let burden = acquisition_projected_integration_burden(game, terms);
    let tone = acquisition_service_tone(projected_reliability);

    format!(
        "   {} post-close reliability about {}; service {}; integration burden {}",
        styled(tone, "service"),
        styled(tone, format!("{:.0}%", projected_reliability * 100.0)),
        styled(
            tone,
            acquisition_service_cushion_text(projected_reliability)
        ),
        styled(
            acquisition_integration_burden_tone(burden),
            acquisition_integration_burden_label(burden)
        )
    )
}

fn acquisition_funding_line(game: &Game, terms: &AcquisitionTerms, exact_terms: bool) -> String {
    let label = if exact_terms {
        "funding"
    } else {
        "est funding"
    };
    let caveat = if exact_terms {
        ""
    } else {
        "; close price can move"
    };
    if game.player.cash + 0.01 < terms.price {
        format!(
            "   {} {}",
            muted(label),
            styled(
                BOLD_RED,
                format!(
                    "need {} more cash to close{caveat}",
                    money(terms.price - game.player.cash)
                )
            )
        )
    } else if terms.post_debt_to_assets > 0.95 {
        format!(
            "   {} {}",
            muted(label),
            styled(
                BOLD_YELLOW,
                format!("would leave balance sheet above board leverage limit{caveat}")
            )
        )
    } else {
        format!(
            "   {} {}",
            muted(label),
            styled(BOLD_GREEN, format!("closeable with current cash{caveat}"))
        )
    }
}

fn acquisition_rate_anchor_line(game: &Game, competitor_index: usize) -> Option<String> {
    let lift = game.acquisition_rate_anchor_lift_cents(competitor_index)?;
    if lift < 0.05 {
        return None;
    }
    let tone = if lift >= 0.25 { BOLD_YELLOW } else { YELLOW };
    Some(format!(
        "   {} removing this low-rate alternative may lift public tolerance by about {}",
        muted("rate anchor"),
        styled(tone, format!("{lift:.1}c"))
    ))
}

pub(super) fn dashboard_competitor_lines(game: &Game) -> Vec<String> {
    if game.competitors.is_empty() {
        return vec!["No active rivals.".to_string()];
    }

    let total_connected = game.total_connected_customers().max(1.0);
    let name_width = if dashboard_width() >= 132 { 21 } else { 15 };
    let visible_rivals = 3;
    let mut lines = vec![format!(
        "{} {}  {}  {}  {} {}",
        styled(DIM, format!("{:>1}", "#")),
        styled(DIM, format!("{:<name_width$}", "rival")),
        styled(DIM, format!("{:>5}", "share")),
        styled(DIM, format!("{:>5}", "rate")),
        styled(DIM, format!("{:<10}", "health")),
        styled(DIM, "buy")
    )];

    for (rank_position, index) in ranked_competitor_indices(game)
        .into_iter()
        .take(visible_rivals)
        .enumerate()
    {
        let competitor = &game.competitors[index];
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
            "{} {}  {}  {}  {} {}",
            styled(DIM, format!("{:>1}", rank_position + 1)),
            styled(BOLD, format!("{name:<name_width$}")),
            styled(
                BOLD,
                format!(
                    "{:>5}",
                    format!("{:.0}%", competitor.customers / total_connected * 100.0)
                )
            ),
            styled(
                rate_tone,
                format!("{:>5}", format!("{:.1}c", competitor.rate_cents))
            ),
            styled(health_tone, format!("{health_label:<10}")),
            styled(acquisition_tone, acquisition_note)
        ));
    }

    if game.competitors.len() > visible_rivals {
        let remaining = game.competitors.len() - visible_rivals;
        lines.push(format!(
            "{} use 'rivals' for {remaining} more",
            muted("More")
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
        (
            game.acquisition_price(competitor_index)
                .map(money)
                .unwrap_or_else(|| "unavailable".to_string()),
            CYAN,
        )
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
