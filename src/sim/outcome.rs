use super::*;

impl Game {
    fn review_sustainability(&self, finances: &FirmFinances) -> ReviewSustainability {
        let break_even_rate = self.player_break_even_rate_cents();
        let rate_support_ratio = if break_even_rate <= 0.0 {
            f64::INFINITY
        } else {
            self.player.rate_cents / break_even_rate
        };
        let interest_coverage = interest_coverage(finances.profit, finances.interest);
        let passes = finances.profit >= REVIEW_MIN_PROFIT
            && rate_support_ratio >= REVIEW_MIN_RATE_SUPPORT_RATIO
            && interest_coverage >= REVIEW_MIN_INTEREST_COVERAGE;

        ReviewSustainability {
            passes,
            break_even_rate,
            rate_support_ratio,
            interest_coverage,
        }
    }

    fn hostile_takeover_bidder_name(&self) -> Option<String> {
        if self.quarter <= HOSTILE_TAKEOVER_MIN_QUARTER || self.player.asset_base <= 0.0 {
            return None;
        }

        let market_cap_to_assets = self.player.market_cap() / self.player.asset_base.max(1.0);
        if market_cap_to_assets >= HOSTILE_TAKEOVER_MARKET_CAP_TO_ASSETS
            || self.player.debt_to_assets() <= HOSTILE_TAKEOVER_DEBT_TO_ASSETS
        {
            return None;
        }

        self.competitors
            .iter()
            .filter(|competitor| {
                self.rival_takeover_financing_capacity(competitor)
                    >= HOSTILE_TAKEOVER_RIVAL_FINANCING_CAPACITY
            })
            .max_by(|left, right| {
                self.rival_takeover_financing_capacity(left)
                    .total_cmp(&self.rival_takeover_financing_capacity(right))
            })
            .map(|competitor| competitor.name.clone())
    }

    fn rival_takeover_financing_capacity(&self, competitor: &Utility) -> f64 {
        let macro_limit = self.macro_state.borrowing_limit_ratio();
        let service_drag = (0.76 - competitor.reliability).clamp(0.0, 0.20) * 0.34;
        let reputation_drag = ((48.0 - competitor.reputation) / 100.0).clamp(0.0, 0.18) * 0.36;
        let limit = (macro_limit - service_drag - reputation_drag).clamp(0.36, macro_limit);
        let borrowing_capacity = (competitor.asset_base * limit - competitor.debt).max(0.0);
        competitor.cash.max(0.0) + borrowing_capacity * HOSTILE_TAKEOVER_RIVAL_DEBT_CAPACITY_WEIGHT
    }

    pub(super) fn check_outcome(&mut self, finances: &FirmFinances) {
        if self.outcome.is_some() {
            return;
        }

        if self.player.reliability < 0.44 {
            self.outcome = Some(Outcome {
                kind: OutcomeKind::Defeat,
                headline: "Market Access Lost".to_string(),
                details: "Repeated outages pushed regulators and lenders to move the company into managed restructuring."
                    .to_string(),
                can_continue: false,
            });
            return;
        }

        if self.quarter > 5
            && self.player.debt_to_assets() > RECEIVERSHIP_DEBT_TO_ASSETS
            && finances.profit < 0.0
        {
            self.outcome = Some(Outcome {
                kind: OutcomeKind::Defeat,
                headline: "Bankers Forced Receivership".to_string(),
                details: "The company could not finance expansion and interest at the same time."
                    .to_string(),
                can_continue: false,
            });
            return;
        }

        if self.quarter > 5
            && self.acquisition_stress >= ACQUISITION_STRESS_RECEIVERSHIP
            && finances.profit < 0.0
            && (self.player.debt_to_assets() > 0.92 || self.player.cash < -2_500.0)
        {
            self.outcome = Some(Outcome {
                kind: OutcomeKind::Defeat,
                headline: "Bankers Forced Receivership".to_string(),
                details: "A strained acquisition left lenders unwilling to fund losses, integration costs, and debt service at the same time."
                    .to_string(),
                can_continue: false,
            });
            return;
        }

        let coverage = interest_coverage(finances.profit, finances.interest);
        if self.quarter > 5
            && self.player.debt_to_assets() > DISTRESSED_COVERAGE_RECEIVERSHIP_DEBT_TO_ASSETS
            && coverage < DISTRESSED_COVERAGE_RECEIVERSHIP
            && (self.player.cash < 8_000.0 || finances.profit < -2_500.0)
        {
            self.outcome = Some(Outcome {
                kind: OutcomeKind::Defeat,
                headline: "Bankers Forced Receivership".to_string(),
                details: "Lenders lost confidence after leverage stayed high while earnings no longer covered debt service."
                    .to_string(),
                can_continue: false,
            });
            return;
        }

        if let Some(bidder_name) = self.hostile_takeover_bidder_name() {
            self.outcome = Some(Outcome {
                kind: OutcomeKind::Defeat,
                headline: "Hostile Takeover".to_string(),
                details: format!(
                    "{} moved to acquire {} after the market valued the company below its asset base while leverage left the board with few defenses.",
                    bidder_name, self.player.name
                ),
                can_continue: false,
            });
            return;
        }

        if self.sandbox_mode {
            return;
        }

        if !self.review_completed && self.quarter >= self.campaign_quarters {
            let share = self.market_share();
            let healthy_balance_sheet = self.player.debt_to_assets() <= 0.95;
            let sustainability = self.review_sustainability(finances);
            self.review_completed = true;
            if share >= 0.45
                && self.player.reliability >= REVIEW_MIN_RELIABILITY
                && healthy_balance_sheet
                && sustainability.passes
            {
                self.outcome = Some(Outcome {
                    kind: OutcomeKind::Victory,
                    headline: "Market Lead Secured".to_string(),
                    details: format!(
                        "You finished with {:.0}% of connected accounts, solid reliability, sustainable rates, and {}.",
                        share * 100.0,
                        review_interest_coverage_summary(
                            self.player.debt,
                            sustainability.interest_coverage
                        )
                    ) + &owner_return_summary(self),
                    can_continue: true,
                });
            } else {
                self.outcome = Some(Outcome {
                    kind: OutcomeKind::Defeat,
                    headline: "Board Lost Confidence".to_string(),
                    details: year_five_review_failure_details(
                        self,
                        finances,
                        share,
                        sustainability,
                    ),
                    can_continue: true,
                });
            }
        }

        if self.review_completed
            && !self.regional_mandate_completed
            && self.quarter >= REGIONAL_MANDATE_QUARTER
        {
            self.regional_mandate_completed = true;
            if self.regional_mandate_met() {
                self.outcome = Some(Outcome {
                    kind: OutcomeKind::Victory,
                    headline: "Regional Platform Secured".to_string(),
                    details: format!(
                        "By Year 10 Metro held {:.0}% share across {} adjacent territories while keeping reliability and leverage inside the regional mandate.",
                        self.market_share() * 100.0,
                        self.adjacent_expansions
                    ) + &owner_return_summary(self),
                    can_continue: true,
                });
            } else {
                self.outcome = Some(Outcome {
                    kind: OutcomeKind::Defeat,
                    headline: "Regional Mandate Missed".to_string(),
                    details: regional_mandate_failure_details(self),
                    can_continue: true,
                });
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct ReviewSustainability {
    passes: bool,
    break_even_rate: f64,
    rate_support_ratio: f64,
    interest_coverage: f64,
}

pub(super) fn interest_coverage(profit: f64, interest: f64) -> f64 {
    if interest <= 1.0 {
        f64::INFINITY
    } else {
        (profit + interest) / interest
    }
}

fn review_interest_coverage_summary(debt: f64, coverage: f64) -> String {
    if debt <= 1.0 {
        "no outstanding debt, so interest coverage was not a constraint".to_string()
    } else if coverage.is_infinite() {
        "no material debt service, so interest coverage was not a constraint".to_string()
    } else {
        format!("{coverage:.1}x interest coverage")
    }
}

fn owner_return_summary(game: &Game) -> String {
    format!(
        " Founder ownership is {:.1}%, worth {} at the closing stock price.",
        game.player_ownership() * 100.0,
        money(game.player_wealth())
    )
}

fn year_five_review_failure_details(
    game: &Game,
    finances: &FirmFinances,
    share: f64,
    sustainability: ReviewSustainability,
) -> String {
    let mut gaps = Vec::new();
    if share < 0.45 {
        gaps.push(format!("share {:.0}% vs 45%", share * 100.0));
    }
    if game.player.reliability < REVIEW_MIN_RELIABILITY {
        gaps.push(format!(
            "service reliability {:.0}% vs {:.0}%",
            game.player.reliability * 100.0,
            REVIEW_MIN_RELIABILITY * 100.0
        ));
    }
    if game.player.debt_to_assets() > 0.95 {
        gaps.push(format!(
            "debt/assets {:.0}% vs 95%",
            game.player.debt_to_assets() * 100.0
        ));
    }
    if finances.profit < REVIEW_MIN_PROFIT {
        gaps.push(format!("profit ${:.1}k", finances.profit / 1_000.0));
    }
    if sustainability.rate_support_ratio < REVIEW_MIN_RATE_SUPPORT_RATIO {
        gaps.push(format!(
            "rate support {:.0}% vs {:.0}%",
            sustainability.rate_support_ratio * 100.0,
            REVIEW_MIN_RATE_SUPPORT_RATIO * 100.0
        ));
    }
    if sustainability.interest_coverage < REVIEW_MIN_INTEREST_COVERAGE {
        gaps.push(format!(
            "interest coverage {:.1}x vs {:.1}x",
            sustainability.interest_coverage, REVIEW_MIN_INTEREST_COVERAGE
        ));
    }

    let gap_summary = if gaps.is_empty() {
        "No single review metric missed by much, but the combined profile did not look durable."
            .to_string()
    } else {
        format!("Review gaps: {}.", gaps.join("; "))
    };
    let scale_note = if share >= 0.70 && !gaps.is_empty() {
        " Scale alone was not enough: the board treated unreliable service, weak rate support, or strained financing as evidence that the franchise was overextended."
    } else {
        ""
    };

    format!(
        "{} After Year 5 you held {:.0}% of connected accounts. The board wanted durable leadership: share, service, leverage, and rates that can support the cost base. Current rate support was {:.0}% of break-even ({:.1}c) with {}.{}",
        gap_summary,
        share * 100.0,
        sustainability.rate_support_ratio.min(9.99) * 100.0,
        sustainability.break_even_rate,
        review_interest_coverage_summary(game.player.debt, sustainability.interest_coverage),
        scale_note
    )
}

fn regional_mandate_failure_details(game: &Game) -> String {
    let share = game.market_share();
    let debt_to_assets = game.player.debt_to_assets();
    let mut gaps = Vec::new();
    if share < REGIONAL_MANDATE_SHARE_TARGET {
        gaps.push(format!(
            "share {:.0}% vs {:.0}%",
            share * 100.0,
            REGIONAL_MANDATE_SHARE_TARGET * 100.0
        ));
    }
    if game.adjacent_expansions < REGIONAL_MANDATE_EXPANSION_TARGET {
        gaps.push(format!(
            "territories {} vs {}",
            game.adjacent_expansions, REGIONAL_MANDATE_EXPANSION_TARGET
        ));
    }
    if game.player.reliability < REGIONAL_MANDATE_RELIABILITY_TARGET {
        gaps.push(format!(
            "reliability {:.0}% vs {:.0}%",
            game.player.reliability * 100.0,
            REGIONAL_MANDATE_RELIABILITY_TARGET * 100.0
        ));
    }
    if debt_to_assets > REGIONAL_MANDATE_LEVERAGE_LIMIT {
        gaps.push(format!(
            "debt/assets {:.0}% vs {:.0}%",
            debt_to_assets * 100.0,
            REGIONAL_MANDATE_LEVERAGE_LIMIT * 100.0
        ));
    }

    let gap_summary = if gaps.is_empty() {
        "No individual gate missed by much, but the board did not see a durable regional platform."
            .to_string()
    } else {
        format!("Mandate gaps: {}.", gaps.join("; "))
    };
    let scale_note = if share >= 0.70 && !gaps.is_empty() {
        " A large customer base did not offset the missing operating commitments."
    } else {
        ""
    };

    format!(
        "{} By Year 10 Metro held {:.0}% share with {} adjacent territories. The board wanted {:.0}% share, {} territories, {:.0}% reliability, and debt/assets below {:.0}%.{}",
        gap_summary,
        share * 100.0,
        game.adjacent_expansions,
        REGIONAL_MANDATE_SHARE_TARGET * 100.0,
        REGIONAL_MANDATE_EXPANSION_TARGET,
        REGIONAL_MANDATE_RELIABILITY_TARGET * 100.0,
        REGIONAL_MANDATE_LEVERAGE_LIMIT * 100.0,
        scale_note
    )
}
