use super::render::*;
use super::*;

pub(super) struct BoardTarget {
    pub(super) label: &'static str,
    pub(super) quarter: u32,
    pub(super) share: f64,
    pub(super) reliability: f64,
    pub(super) leverage: f64,
}

pub(super) struct ScorecardReviewTargets {
    pub(super) share: f64,
    pub(super) reliability: f64,
    pub(super) leverage: f64,
    pub(super) regional: bool,
}

pub(super) fn scorecard_review_targets(game: &Game) -> ScorecardReviewTargets {
    if game.review_completed {
        ScorecardReviewTargets {
            share: REGIONAL_MANDATE_SHARE_TARGET,
            reliability: REGIONAL_MANDATE_RELIABILITY_TARGET,
            leverage: REGIONAL_MANDATE_LEVERAGE_LIMIT,
            regional: true,
        }
    } else {
        ScorecardReviewTargets {
            share: SHARE_TARGET,
            reliability: RELIABILITY_TARGET,
            leverage: LEVERAGE_LIMIT,
            regional: false,
        }
    }
}

pub(super) fn board_overview_lines(game: &Game) -> Vec<String> {
    if game.sandbox_mode {
        return vec![
            format!(
                "{} {}   {} {}",
                muted("Current"),
                styled(BOLD_CYAN, game.date_label()),
                muted("Mode"),
                styled(BOLD, "Sandbox")
            ),
            "Board reviews and mandate checkpoints are disabled for this run.".to_string(),
            "Operating failures still apply: service collapse, receivership, and hostile takeover."
                .to_string(),
        ];
    }

    let next = next_board_target(game);
    vec![
        format!(
            "{} {}   {} {}",
            muted("Current"),
            styled(BOLD_CYAN, game.date_label()),
            muted("Next"),
            styled(BOLD, next.label)
        ),
        format!(
            "{} {} | {} {} | {} {}",
            muted("Target"),
            styled(BOLD_GREEN, format!("{:.0}% share", next.share * 100.0)),
            muted("reliability"),
            styled(BOLD_GREEN, format!("{:.0}%+", next.reliability * 100.0)),
            muted("debt/assets"),
            styled(BOLD_GREEN, format!("<={:.0}%", next.leverage * 100.0))
        ),
        format!(
            "{} {} | {} {} | {} {}",
            muted("Now"),
            styled(
                share_tone(game.market_share()),
                format!("{:.0}% share", game.market_share() * 100.0)
            ),
            muted("reliability"),
            styled(
                reliability_tone(game.player.reliability),
                format!("{:.0}%", game.player.reliability * 100.0)
            ),
            muted("debt/assets"),
            styled(
                leverage_tone(game.player.debt_to_assets()),
                format!("{:.0}%", game.player.debt_to_assets() * 100.0)
            )
        ),
    ]
}

pub(super) fn board_milestone_lines(game: &Game) -> Vec<String> {
    if game.sandbox_mode {
        return vec![
            signal_line("Board", CYAN, "no active review milestones"),
            signal_line("Growth", GREEN, "expand when cash and lenders permit"),
            signal_line("Operations", GREEN, "service and solvency still matter"),
        ];
    }

    let target = next_board_target(game);
    vec![
        format!(
            "{} {}",
            muted("Checkpoint"),
            styled(BOLD_CYAN, target.label)
        ),
        board_metric_line("Share", game.market_share(), target.share, true),
        board_metric_line(
            "Reliability",
            game.player.reliability,
            target.reliability,
            true,
        ),
        board_metric_line(
            "Debt/assets",
            game.player.debt_to_assets(),
            target.leverage,
            false,
        ),
    ]
}

pub(super) fn board_risk_lines(game: &Game) -> Vec<String> {
    if game.sandbox_mode {
        return vec![
            signal_line(
                "Service",
                reliability_tone(game.player.reliability),
                "market access still depends on reliability",
            ),
            signal_line(
                "Capital",
                leverage_tone(game.player.debt_to_assets()),
                "lenders can still force receivership",
            ),
            signal_line(
                "Control",
                ownership_tone(game.player_ownership()),
                "weak valuation can still invite a hostile bid",
            ),
        ];
    }

    let target = next_board_target(game);
    let mut lines = Vec::new();
    if let Some(warning) = review_gate_warning(game) {
        lines.push(warning);
    }
    let share_gap = target.share - game.market_share();
    if share_gap > 0.06 {
        lines.push(signal_line(
            "Growth",
            YELLOW,
            format!("share needs {}", points(share_gap)),
        ));
    } else if share_gap <= 0.0 {
        lines.push(signal_line("Growth", GREEN, "share milestone already met"));
    } else {
        lines.push(signal_line("Growth", CYAN, "share gap is manageable"));
    }

    let headroom = game.player.capacity_headroom(&game.market);
    if headroom < 70.0 {
        lines.push(signal_line("Capacity", RED, "network may block growth"));
    } else if headroom < 140.0 {
        lines.push(signal_line("Capacity", YELLOW, "plan expansion soon"));
    } else {
        lines.push(signal_line("Capacity", GREEN, "room for customer growth"));
    }

    if game.player.reliability < 0.50 {
        lines.push(signal_line(
            "Service",
            RED,
            "market access is at risk from outages",
        ));
    } else if game.player.reliability < target.reliability {
        lines.push(signal_line("Service", RED, "maintenance before review"));
    } else if game.player.reliability < target.reliability + 0.04 {
        lines.push(signal_line("Service", YELLOW, "thin reliability cushion"));
    } else {
        lines.push(signal_line("Service", GREEN, "reliability cushion intact"));
    }

    let leverage = game.player.debt_to_assets();
    if leverage > target.leverage {
        lines.push(signal_line("Capital", RED, "delever before checkpoint"));
    } else if game.borrowing_room() > 25_000.0 {
        lines.push(signal_line("Capital", GREEN, "borrowing room available"));
    } else {
        lines.push(signal_line("Capital", YELLOW, "funding room is limited"));
    }

    let rate_support = game.player_rate_support_ratio();
    if rate_support < REVIEW_MIN_RATE_SUPPORT_RATIO {
        lines.push(signal_line(
            "Earnings",
            RED,
            format!(
                "{:.0}% of break-even; rate path is not durable",
                rate_support * 100.0
            ),
        ));
    } else if let Some(coverage) = game.player_last_interest_coverage() {
        if coverage < REVIEW_MIN_INTEREST_COVERAGE {
            lines.push(signal_line(
                "Earnings",
                RED,
                format!("interest coverage only {}", format_coverage(coverage)),
            ));
        } else if coverage < 1.30 {
            lines.push(signal_line(
                "Earnings",
                YELLOW,
                format!("thin interest coverage at {}", format_coverage(coverage)),
            ));
        } else {
            lines.push(signal_line(
                "Earnings",
                GREEN,
                "rate and debt service look durable",
            ));
        }
    } else {
        lines.push(signal_line(
            "Earnings",
            CYAN,
            "first operating report pending",
        ));
    }

    lines
}

pub(super) fn board_path_lines(game: &Game) -> Vec<String> {
    if game.sandbox_mode {
        return vec![
            styled(BOLD_CYAN, "Sandbox path"),
            "No Year 5 or Year 10 board outcome will end this run.".to_string(),
            "Use your own goals: durable profits, regional scale, founder wealth, or market leadership."
                .to_string(),
        ];
    }

    board_targets()
        .iter()
        .map(|target| {
            let status = if game.quarter >= target.quarter {
                let met = game.market_share() >= target.share
                    && game.player.reliability >= target.reliability
                    && game.player.debt_to_assets() <= target.leverage;
                if met {
                    styled(BOLD_GREEN, "met")
                } else {
                    styled(BOLD_RED, "missed")
                }
            } else if target.quarter == next_board_target(game).quarter {
                styled(BOLD_CYAN, "next")
            } else {
                styled(DIM, "later")
            };
            format!(
                "{} {:<10} | share {:.0}% | rel {:.0}% | debt/assets <= {:.0}%",
                status,
                target.label,
                target.share * 100.0,
                target.reliability * 100.0,
                target.leverage * 100.0
            )
        })
        .collect()
}

pub(super) fn should_show_regional_status(game: &Game) -> bool {
    !game.sandbox_mode
        && !game.regional_mandate_completed
        && game.quarter < REGIONAL_MANDATE_QUARTER
}

#[cfg(test)]
pub(super) fn regional_status_lines(game: &Game) -> Vec<String> {
    let pending = adjacent_pending_quarters(game);
    let mut lines = vec![
        format!(
            "{} {} / {}   {} {}",
            muted("Territories"),
            styled(
                expansion_tone(game.adjacent_expansions),
                game.adjacent_expansions
            ),
            muted(REGIONAL_MANDATE_EXPANSION_TARGET),
            muted("mandate"),
            styled(
                if game.regional_mandate_met() {
                    BOLD_GREEN
                } else {
                    BOLD_CYAN
                },
                regional_due_label(game)
            )
        ),
        format!(
            "{} {}   {} {}",
            muted("Regional share"),
            styled(
                share_tone(game.market_share()),
                format!(
                    "{:.0}% / {:.0}%",
                    game.market_share() * 100.0,
                    REGIONAL_MANDATE_SHARE_TARGET * 100.0
                )
            ),
            muted("Integration"),
            styled(
                regional_integration_tone(game.regional_integration),
                format!("{:.0} pts", game.regional_integration * 100.0)
            )
        ),
    ];

    if let Some(quarters) = pending {
        lines.push(format!(
            "{} {}",
            muted("Pending"),
            styled(YELLOW, format!("adjacent territory opens in {quarters}q"))
        ));
    } else if game.regional_integration > 0.12 {
        lines.push(format!(
            "{} {}",
            muted("Follow-up"),
            styled(
                YELLOW,
                "marketing and maintenance reduce regional integration burden"
            )
        ));
    } else if game.review_completed && !game.regional_mandate_completed {
        lines.push(format!(
            "{} {}",
            muted("Next step"),
            styled(CYAN, "expand and defend service quality before Year 10")
        ));
    } else if !game.review_completed {
        lines.push(format!(
            "{} {}",
            muted("Plan ahead"),
            styled(
                DIM,
                format!(
                    "Y5 review unlocks adjacent expansion (need {:.0}% share, {:.0}% reliability, debt/assets <= {:.0}%)",
                    REGIONAL_MANDATE_SHARE_TARGET * 100.0,
                    REGIONAL_MANDATE_RELIABILITY_TARGET * 100.0,
                    REGIONAL_MANDATE_LEVERAGE_LIMIT * 100.0
                )
            )
        ));
    }

    lines
}

pub(super) fn regional_mandate_lines(game: &Game) -> Vec<String> {
    vec![
        format!(
            "{} {}   {} {}",
            muted("Due"),
            styled(BOLD_CYAN, regional_due_label(game)),
            muted("Status"),
            styled(
                if game.regional_mandate_met() {
                    BOLD_GREEN
                } else {
                    YELLOW
                },
                if game.regional_mandate_met() {
                    "on mandate"
                } else {
                    "not yet secure"
                }
            )
        ),
        board_metric_line(
            "Share",
            game.market_share(),
            REGIONAL_MANDATE_SHARE_TARGET,
            true,
        ),
        board_metric_line(
            "Reliability",
            game.player.reliability,
            REGIONAL_MANDATE_RELIABILITY_TARGET,
            true,
        ),
        board_metric_line(
            "Debt/assets",
            game.player.debt_to_assets(),
            REGIONAL_MANDATE_LEVERAGE_LIMIT,
            false,
        ),
        format!(
            "{} {} / {}   {} {}",
            styled(BOLD, "Territories"),
            styled(
                expansion_tone(game.adjacent_expansions),
                game.adjacent_expansions
            ),
            muted(REGIONAL_MANDATE_EXPANSION_TARGET),
            styled(BOLD, "Integration"),
            styled(
                regional_integration_tone(game.regional_integration),
                format!("{:.0} pts", game.regional_integration * 100.0)
            )
        ),
    ]
}

pub(super) fn regional_due_label(game: &Game) -> String {
    if game.regional_mandate_completed {
        return "mandate reviewed".to_string();
    }
    if game.quarter >= REGIONAL_MANDATE_QUARTER {
        return "review due now".to_string();
    }
    let remaining = game.regional_mandate_due_in();
    if remaining == 1 {
        "Year 10, 1 quarter left".to_string()
    } else {
        format!("Year 10, {remaining} quarters left")
    }
}

#[cfg(test)]
fn adjacent_pending_quarters(game: &Game) -> Option<u32> {
    game.pending_projects.iter().find_map(|project| {
        matches!(project.kind, crate::ProjectKind::AdjacentTerritory { .. })
            .then_some(project.quarters_remaining)
    })
}

pub(super) fn expansion_tone(expansions: u32) -> &'static str {
    if expansions >= REGIONAL_MANDATE_EXPANSION_TARGET {
        BOLD_GREEN
    } else if expansions > 0 {
        YELLOW
    } else {
        RED
    }
}

fn regional_integration_tone(value: f64) -> &'static str {
    if value <= 0.12 {
        BOLD_GREEN
    } else if value <= 0.28 {
        YELLOW
    } else {
        RED
    }
}

pub(super) fn integration_strain_label(value: f64) -> &'static str {
    if value >= 0.50 {
        "severe"
    } else if value >= 0.30 {
        "high"
    } else if value >= 0.12 {
        "moderate"
    } else if value > 0.01 {
        "light"
    } else {
        "clear"
    }
}

pub(super) fn integration_strain_tone(value: f64) -> &'static str {
    if value >= 0.50 {
        RED
    } else if value >= 0.30 {
        BOLD_YELLOW
    } else if value >= 0.08 {
        YELLOW
    } else {
        GREEN
    }
}

pub(super) fn review_gate_warning(game: &Game) -> Option<String> {
    if game.sandbox_mode || game.regional_mandate_completed {
        return None;
    }

    let targets = scorecard_review_targets(game);
    let share = game.market_share();
    let mut gaps = Vec::new();
    if share < targets.share {
        gaps.push("share");
    }
    if game.player.reliability < targets.reliability {
        gaps.push("service");
    }
    if game.player.debt_to_assets() > targets.leverage {
        gaps.push("leverage");
    }
    if !targets.regional && game.player_rate_support_ratio() < REVIEW_MIN_RATE_SUPPORT_RATIO {
        gaps.push("rate support");
    }
    if gaps.is_empty() {
        return None;
    }

    if share >= targets.share {
        Some(signal_line(
            "Review",
            RED,
            format!("share is not enough; {} short", gaps.join(", ")),
        ))
    } else {
        Some(signal_line(
            "Review",
            YELLOW,
            format!("active gates: {}", gaps.join(", ")),
        ))
    }
}

pub(super) fn board_metric_line(
    label: &str,
    current: f64,
    target: f64,
    higher_is_better: bool,
) -> String {
    let met = if higher_is_better {
        current >= target
    } else {
        current <= target
    };
    let tone = if met { BOLD_GREEN } else { BOLD_YELLOW };
    let note = if met {
        if higher_is_better {
            format!("{} cushion", points(current - target))
        } else {
            format!("{} room", points(target - current))
        }
    } else if higher_is_better {
        format!("{} short", points(target - current))
    } else {
        format!("{} over", points(current - target))
    };
    format!(
        "{} {} / {}   {}",
        styled(BOLD, label),
        styled(tone, format!("{:.0}%", current * 100.0)),
        muted(format!("{:.0}%", target * 100.0)),
        muted(note)
    )
}

pub(super) fn next_board_target(game: &Game) -> BoardTarget {
    if game.review_completed && !game.regional_mandate_completed {
        return BoardTarget {
            label: "Y10 Mandate",
            quarter: REGIONAL_MANDATE_QUARTER,
            share: REGIONAL_MANDATE_SHARE_TARGET,
            reliability: REGIONAL_MANDATE_RELIABILITY_TARGET,
            leverage: REGIONAL_MANDATE_LEVERAGE_LIMIT,
        };
    }

    if game.review_completed {
        return BoardTarget {
            label: "Continuation",
            quarter: game.quarter,
            share: SHARE_TARGET,
            reliability: RELIABILITY_TARGET,
            leverage: LEVERAGE_LIMIT,
        };
    }

    board_targets()
        .into_iter()
        .find(|target| game.quarter < target.quarter)
        .unwrap_or(BoardTarget {
            label: "Final",
            quarter: game.campaign_quarters,
            share: SHARE_TARGET,
            reliability: RELIABILITY_TARGET,
            leverage: LEVERAGE_LIMIT,
        })
}

pub(super) fn board_targets() -> [BoardTarget; 4] {
    [
        BoardTarget {
            label: "Year 2",
            quarter: 8,
            share: 0.28,
            reliability: 0.74,
            leverage: 1.05,
        },
        BoardTarget {
            label: "Year 3",
            quarter: 12,
            share: 0.34,
            reliability: 0.74,
            leverage: 1.00,
        },
        BoardTarget {
            label: "Year 4",
            quarter: 16,
            share: 0.40,
            reliability: 0.73,
            leverage: 0.98,
        },
        BoardTarget {
            label: "Final",
            quarter: 20,
            share: SHARE_TARGET,
            reliability: RELIABILITY_TARGET,
            leverage: LEVERAGE_LIMIT,
        },
    ]
}

pub(super) fn points(value: f64) -> String {
    format!("{:.0} pts", value.abs() * 100.0)
}
