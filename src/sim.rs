const DEFAULT_CAMPAIGN_QUARTERS: u32 = 20;
pub const GENERATION_PROJECT_COST: f64 = 18_500.0;
pub const GENERATION_PROJECT_CAPACITY_MWH: f64 = 185.0;
pub const DISTRIBUTION_PROJECT_COST: f64 = 9_500.0;
pub const DISTRIBUTION_PROJECT_CAPACITY: f64 = 230.0;
pub const MIN_GENERATION_PROJECT_MWH: f64 = 60.0;
pub const MAX_GENERATION_PROJECT_MWH: f64 = 650.0;
pub const MIN_DISTRIBUTION_PROJECT_CUSTOMERS: f64 = 80.0;
pub const MAX_DISTRIBUTION_PROJECT_CUSTOMERS: f64 = 900.0;
const BASE_ANNUAL_RATE: f64 = 0.052;
const BASE_CREDIT_SPREAD: f64 = 0.018;
const MAINTENANCE_REFERENCE_ASSET_BASE: f64 = 78_000.0;

#[derive(Clone, Debug)]
pub struct Game {
    pub quarter: u32,
    pub campaign_quarters: u32,
    pub market: Market,
    pub macro_state: MacroEnvironment,
    pub player: Utility,
    pub competitors: Vec<Utility>,
    pub pending_projects: Vec<Project>,
    pub acquisition_cooldown: u32,
    pub active_shocks: Vec<ActiveShock>,
    pub startup_index: u32,
    pub last_report: Option<QuarterReport>,
    pub outcome: Option<Outcome>,
    rng: Rng,
}

#[derive(Clone, Copy, Debug)]
pub struct InitialVariance {
    pub amplitude: f64,
}

#[derive(Clone, Debug)]
pub struct ActiveShock {
    pub kind: ShockKind,
    pub quarters_remaining: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ShockKind {
    RateFreeze,
    DemandRecession,
    DemandBoom,
    InputCostShock,
}

#[derive(Clone, Debug)]
pub struct Market {
    pub territory: String,
    pub start_year: u32,
    pub addressable_customers: f64,
    pub electrification: f64,
    pub avg_mwh_per_customer: f64,
    pub variable_cost_per_mwh: f64,
    pub standard_rate_cents: f64,
    pub civic_patience: f64,
}

#[derive(Clone, Debug)]
pub struct MacroEnvironment {
    pub annual_base_rate: f64,
    pub credit_spread: f64,
    pub demand_index: f64,
    pub cost_pressure: f64,
}

#[derive(Clone, Debug)]
pub struct Utility {
    pub name: String,
    pub cash: f64,
    pub debt: f64,
    pub shares: f64,
    pub stock_price: f64,
    pub customers: f64,
    pub generation_capacity_mwh: f64,
    pub distribution_capacity: f64,
    pub rate_cents: f64,
    pub reputation: f64,
    pub reliability: f64,
    pub marketing_momentum: f64,
    pub asset_base: f64,
    pub last_quarter_customers: f64,
}

#[derive(Clone, Debug)]
pub struct Project {
    pub name: String,
    pub kind: ProjectKind,
    pub quarters_remaining: u32,
}

#[derive(Clone, Debug)]
pub enum ProjectKind {
    Generation { capacity_mwh: f64 },
    Distribution { customer_capacity: f64 },
}

#[derive(Clone, Debug)]
pub enum Decision {
    BuildGeneration { capacity_mwh: f64 },
    BuildDistribution { customer_capacity: f64 },
    Marketing { spend: f64 },
    IssueStock { amount: f64 },
    BuyBackStock { amount: f64 },
    Borrow { amount: f64 },
    RepayDebt { amount: f64 },
    Acquire { competitor_index: usize },
    AdjustRate { delta_cents: f64 },
    Maintenance { spend: f64 },
}

#[derive(Clone, Debug)]
pub struct QuarterReport {
    pub label: String,
    pub revenue: f64,
    pub operating_cost: f64,
    pub interest: f64,
    pub profit: f64,
    pub new_customers: f64,
    pub lost_customers: f64,
    pub lost_customer_rate: f64,
    pub market_share: f64,
    pub prior_market_share: f64,
    pub events: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct FirmFinances {
    pub revenue: f64,
    pub operating_cost: f64,
    pub interest: f64,
    pub profit: f64,
    pub served_mwh: f64,
    pub unmet_demand_ratio: f64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutcomeKind {
    Victory,
    Defeat,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub kind: OutcomeKind,
    pub headline: String,
    pub details: String,
}

#[derive(Clone, Debug)]
struct Rng {
    state: u64,
}

impl Game {
    pub fn new() -> Self {
        Self::with_seed(fresh_seed())
    }

    pub fn with_seed(seed: u64) -> Self {
        Self::with_seed_and_initial_variance(seed, InitialVariance::default())
    }

    pub fn with_seed_and_initial_variance(seed: u64, initial_variance: InitialVariance) -> Self {
        let mut rng = Rng::new(seed);
        let market = Market {
            territory: "Core Market".to_string(),
            start_year: 1,
            addressable_customers: varied_pct(&mut rng, 6_500.0, 0.08, initial_variance).round(),
            electrification: varied_abs(&mut rng, 0.14, 0.018, initial_variance).clamp(0.10, 0.18),
            avg_mwh_per_customer: varied_pct(&mut rng, 0.22, 0.08, initial_variance)
                .clamp(0.18, 0.27),
            variable_cost_per_mwh: varied_pct(&mut rng, 26.0, 0.07, initial_variance)
                .clamp(21.0, 31.0),
            standard_rate_cents: varied_abs(&mut rng, 10.4, 0.35, initial_variance)
                .clamp(9.5, 11.3),
            civic_patience: varied_abs(&mut rng, 0.76, 0.04, initial_variance).clamp(0.64, 0.88),
        };

        Self {
            quarter: 0,
            campaign_quarters: DEFAULT_CAMPAIGN_QUARTERS,
            market,
            macro_state: MacroEnvironment {
                annual_base_rate: varied_abs(&mut rng, BASE_ANNUAL_RATE, 0.006, initial_variance)
                    .clamp(0.038, 0.070),
                credit_spread: varied_abs(&mut rng, BASE_CREDIT_SPREAD, 0.005, initial_variance)
                    .clamp(0.008, 0.032),
                demand_index: varied_abs(&mut rng, 0.0, 0.12, initial_variance).clamp(-0.22, 0.22),
                cost_pressure: varied_abs(&mut rng, 0.0, 0.008, initial_variance)
                    .clamp(-0.014, 0.018),
            },
            player: starting_utility(
                "Metro Consolidated",
                34_000.0,
                20_000.0,
                2_000.0,
                24.0,
                160.0,
                72.0,
                275.0,
                10.5,
                55.0,
                0.82,
                0.08,
                78_000.0,
                initial_variance,
                &mut rng,
            ),
            competitors: vec![
                starting_utility(
                    "North Loop Power",
                    18_500.0,
                    12_000.0,
                    0.0,
                    0.0,
                    260.0,
                    118.0,
                    360.0,
                    10.1,
                    58.0,
                    0.81,
                    0.04,
                    96_000.0,
                    initial_variance,
                    &mut rng,
                ),
                starting_utility(
                    "District Current",
                    12_000.0,
                    8_000.0,
                    0.0,
                    0.0,
                    210.0,
                    95.0,
                    300.0,
                    9.8,
                    52.0,
                    0.76,
                    0.05,
                    71_000.0,
                    initial_variance,
                    &mut rng,
                ),
                starting_utility(
                    "Metro Light",
                    7_500.0,
                    6_500.0,
                    0.0,
                    0.0,
                    110.0,
                    50.0,
                    160.0,
                    11.2,
                    43.0,
                    0.71,
                    0.02,
                    42_000.0,
                    initial_variance,
                    &mut rng,
                ),
            ],
            pending_projects: Vec::new(),
            acquisition_cooldown: 0,
            active_shocks: Vec::new(),
            startup_index: 0,
            last_report: None,
            outcome: None,
            rng,
        }
    }

    pub fn date_label(&self) -> String {
        let year = self.quarter / 4 + 1;
        let quarter = self.quarter % 4 + 1;
        format!("Year {year} Q{quarter}")
    }

    pub fn apply_decision(&mut self, decision: Decision) -> Result<String, String> {
        if self.outcome.is_some() {
            return Err("The market review is already over.".to_string());
        }

        match decision {
            Decision::BuildGeneration { capacity_mwh } => {
                let capacity_mwh =
                    capacity_mwh.clamp(MIN_GENERATION_PROJECT_MWH, MAX_GENERATION_PROJECT_MWH);
                let cost = generation_project_cost(capacity_mwh);
                let quarters = generation_project_duration(capacity_mwh);
                self.require_cash(cost)?;
                self.player.cash -= cost;
                self.player.asset_base += cost;
                self.pending_projects.push(Project {
                    name: "generation expansion".to_string(),
                    kind: ProjectKind::Generation { capacity_mwh },
                    quarters_remaining: quarters,
                });
                Ok(format!(
                    "Started a generation expansion for {}. It will add {:.0} MWh/quarter in {} quarter(s).",
                    money(cost),
                    capacity_mwh,
                    quarters
                ))
            }
            Decision::BuildDistribution { customer_capacity } => {
                let customer_capacity = customer_capacity.clamp(
                    MIN_DISTRIBUTION_PROJECT_CUSTOMERS,
                    MAX_DISTRIBUTION_PROJECT_CUSTOMERS,
                );
                let cost = distribution_project_cost(customer_capacity);
                let quarters = distribution_project_duration(customer_capacity);
                self.require_cash(cost)?;
                self.player.cash -= cost;
                self.player.asset_base += cost;
                self.pending_projects.push(Project {
                    name: "distribution expansion".to_string(),
                    kind: ProjectKind::Distribution { customer_capacity },
                    quarters_remaining: quarters,
                });
                Ok(format!(
                    "Started a distribution expansion for {}. It will add room for {:.0} customers in {} quarter(s).",
                    money(cost),
                    customer_capacity,
                    quarters
                ))
            }
            Decision::Marketing { spend } => {
                let spend = spend.clamp(1_000.0, 25_000.0);
                self.require_cash(spend)?;
                self.player.cash -= spend;
                self.player.marketing_momentum += (spend / 5_000.0) * 0.13;
                self.player.reputation =
                    (self.player.reputation + spend / 4_000.0).clamp(0.0, 100.0);
                Ok(format!(
                    "Spent {} on customer acquisition, financing offers, and sales coverage.",
                    money(spend)
                ))
            }
            Decision::IssueStock { amount } => {
                let amount = positive_amount(amount, "stock issuance")?;
                let old_market_cap = self.player.market_cap().max(1.0);
                let old_shares = self.player.shares.max(1.0);
                let pressure = amount / old_market_cap;
                let pressure_squared = pressure * pressure;
                let finance_pressure = self.macro_state.financing_pressure();
                let issue_discount =
                    (0.03 + pressure * 0.05 + pressure_squared * 0.08 + finance_pressure * 0.055)
                        .clamp(0.03, 0.65);
                let issue_price = (self.player.stock_price * (1.0 - issue_discount)).max(1.0);
                let new_shares = amount / issue_price;
                let fee_rate =
                    0.025 + pressure * 0.012 + pressure_squared * 0.020 + finance_pressure * 0.012;
                if fee_rate >= 0.75 {
                    return Err(format!(
                        "The market cannot absorb that stock sale. Expected fees would consume {:.0}% of gross proceeds; try a smaller issue.",
                        fee_rate * 100.0
                    ));
                }
                let underwriting_cost = amount * fee_rate;
                let net_proceeds = amount - underwriting_cost;
                let post_money_price = (old_market_cap + net_proceeds) / (old_shares + new_shares);
                let signal_drag = (pressure * 0.020 + pressure_squared * 0.040).clamp(0.0, 0.30);
                self.player.cash += net_proceeds;
                self.player.shares += new_shares;
                self.player.stock_price = (post_money_price * (1.0 - signal_drag)).max(1.0);
                self.player.reputation =
                    (self.player.reputation - pressure * 1.5 - pressure_squared * 1.5)
                        .clamp(0.0, 100.0);
                Ok(format!(
                    "Issued {:.0} shares at ${:.2}. Gross {}, net cash {} after fees; dilution reset stock to ${:.2}.",
                    new_shares,
                    issue_price,
                    money(amount),
                    money(net_proceeds),
                    self.player.stock_price
                ))
            }
            Decision::BuyBackStock { amount } => {
                if self.player.shares <= 500.0 {
                    return Err(
                        "The company has too few public shares to buy back more.".to_string()
                    );
                }
                let amount = positive_amount(amount, "stock buyback")?;
                let old_market_cap = self.player.market_cap().max(1.0);
                let pressure = amount / old_market_cap;
                let repurchase_premium = 0.02 + pressure.min(3.0) * 0.06;
                let repurchase_price =
                    (self.player.stock_price * (1.0 + repurchase_premium)).max(1.0);
                let max_shares = (self.player.shares - 500.0).max(0.0);
                let shares_bought = (amount / repurchase_price).min(max_shares);
                if shares_bought < 1.0 {
                    return Err(
                        "The buyback is too small to retire a meaningful share count.".to_string(),
                    );
                }
                let actual_spend = shares_bought * repurchase_price;
                self.require_cash(actual_spend)?;
                let remaining_shares = self.player.shares - shares_bought;
                let remaining_equity_value = (old_market_cap - actual_spend).max(remaining_shares);
                let theoretical_price = remaining_equity_value / remaining_shares.max(1.0);
                let confidence_lift = (pressure * 0.05).clamp(0.0, 0.10);
                self.player.cash -= actual_spend;
                self.player.shares = remaining_shares;
                self.player.stock_price = (theoretical_price * (1.0 + confidence_lift)).max(1.0);
                Ok(format!(
                    "Bought back {:.0} shares at ${:.2}, spending {}. Shares outstanding now {:.0}; stock is ${:.2}.",
                    shares_bought,
                    repurchase_price,
                    money(actual_spend),
                    self.player.shares,
                    self.player.stock_price
                ))
            }
            Decision::Borrow { amount } => {
                let amount = positive_amount(amount, "debt issuance")?;
                let debt_capacity = self.borrowing_room();
                if amount > debt_capacity {
                    return Err(format!(
                        "Bankers will only extend about {} more under current credit conditions.",
                        money(debt_capacity)
                    ));
                }
                self.player.cash += amount;
                self.player.debt += amount;
                let leverage = self.player.debt_to_assets();
                let annual_rate = self.macro_state.annual_interest_rate_for(&self.player);
                if leverage > 0.72 {
                    self.player.reputation = (self.player.reputation - 1.8).clamp(0.0, 100.0);
                }
                Ok(format!(
                    "Borrowed {} at a current floating rate of {:.1}% annual. Debt/assets now {:.0}%.",
                    money(amount),
                    annual_rate * 100.0,
                    leverage * 100.0
                ))
            }
            Decision::RepayDebt { amount } => {
                if self.player.debt <= 0.0 {
                    return Err("There is no debt outstanding.".to_string());
                }
                let amount = positive_amount(amount, "debt repayment")?;
                let payment = amount.min(self.player.debt);
                self.require_cash(payment)?;
                self.player.cash -= payment;
                self.player.debt -= payment;
                if self.player.debt > 0.0 {
                    Ok(format!(
                        "Repaid {} of debt. Debt/assets now {:.0}%; floating rate now {:.1}% annual.",
                        money(payment),
                        self.player.debt_to_assets() * 100.0,
                        self.macro_state.annual_interest_rate_for(&self.player) * 100.0
                    ))
                } else {
                    Ok("Repaid all outstanding debt.".to_string())
                }
            }
            Decision::Acquire { competitor_index } => {
                if self.acquisition_cooldown > 0 {
                    return Err(format!(
                        "Integration capacity is tied up for {} more quarter(s) before another acquisition.",
                        self.acquisition_cooldown
                    ));
                }

                if competitor_index >= self.competitors.len() {
                    return Err("No competitor has that number.".to_string());
                }

                if self.competitors.len() == 1 {
                    return Err(
                        "The market regulator will not approve the last independent rival's sale."
                            .to_string(),
                    );
                }

                let price = self.acquisition_price(competitor_index);
                self.require_cash(price)?;
                let acquired = self.competitors.remove(competitor_index);
                let acquired_name = acquired.name.clone();
                let absorbed_cash = acquired.cash * 0.80;
                let assumed_debt = acquired.debt * 0.75;
                self.player.cash -= price;
                self.player.cash += absorbed_cash;
                self.player.debt += assumed_debt;
                // Integration losses keep roll-ups from being pure scale arbitrage.
                self.player.customers += acquired.customers * 0.92;
                self.player.generation_capacity_mwh += acquired.generation_capacity_mwh * 0.86;
                self.player.distribution_capacity += acquired.distribution_capacity * 0.90;
                self.player.asset_base += acquired.asset_base * 0.72;
                self.player.reputation = weighted_average(
                    self.player.reputation,
                    self.player.customers,
                    acquired.reputation,
                    acquired.customers * 0.65,
                )
                .clamp(0.0, 100.0);
                self.player.reliability = weighted_average(
                    self.player.reliability,
                    self.player.generation_capacity_mwh,
                    acquired.reliability,
                    acquired.generation_capacity_mwh * 0.70,
                )
                .clamp(0.35, 0.98);
                self.player.reliability = (self.player.reliability - 0.035).clamp(0.35, 0.98);
                self.player.reputation = (self.player.reputation - 1.5).clamp(0.0, 100.0);
                self.acquisition_cooldown = 3;
                Ok(format!(
                    "Acquired {acquired_name} for {} (net of {} absorbed cash, plus {} assumed debt).",
                    money(price),
                    money(absorbed_cash),
                    money(assumed_debt)
                ))
            }
            Decision::AdjustRate { delta_cents } => {
                if delta_cents > 0.0 && self.rate_frozen() {
                    return Err(
                        "The market-wide rate freeze blocks any rate increase right now."
                            .to_string(),
                    );
                }
                let old = self.player.rate_cents;
                self.player.rate_cents = (self.player.rate_cents + delta_cents).clamp(7.0, 15.0);
                let actual_delta = self.player.rate_cents - old;
                if actual_delta > 0.0 {
                    self.player.reputation =
                        (self.player.reputation - actual_delta * 1.8).clamp(0.0, 100.0);
                    Ok(format!(
                        "Raised the rate from {:.1}c to {:.1}c per kWh.",
                        old, self.player.rate_cents
                    ))
                } else if actual_delta < 0.0 {
                    self.player.reputation =
                        (self.player.reputation + (-actual_delta) * 0.9).clamp(0.0, 100.0);
                    Ok(format!(
                        "Cut the rate from {:.1}c to {:.1}c per kWh.",
                        old, self.player.rate_cents
                    ))
                } else {
                    Ok(format!(
                        "The rate remains {:.1}c per kWh.",
                        self.player.rate_cents
                    ))
                }
            }
            Decision::Maintenance { spend } => {
                let spend = spend.clamp(1_500.0, 20_000.0);
                self.require_cash(spend)?;
                self.player.cash -= spend;
                let gain = maintenance_reliability_gain(&self.player, spend);
                self.player.reliability = (self.player.reliability + gain).clamp(0.35, 0.98);
                self.player.reputation = (self.player.reputation
                    + maintenance_reputation_gain(&self.player, spend))
                .clamp(0.0, 100.0);
                Ok(format!(
                    "Spent {} on reliability work; gained {:.1} reliability points.",
                    money(spend),
                    gain * 100.0
                ))
            }
        }
    }

    pub fn advance_quarter(&mut self) -> QuarterReport {
        if let Some(outcome) = &self.outcome {
            let report = QuarterReport {
                label: self.date_label(),
                revenue: 0.0,
                operating_cost: 0.0,
                interest: 0.0,
                profit: 0.0,
                new_customers: 0.0,
                lost_customers: 0.0,
                lost_customer_rate: 0.0,
                market_share: self.market_share(),
                prior_market_share: self
                    .last_report
                    .as_ref()
                    .map(|report| report.market_share)
                    .unwrap_or_else(|| self.market_share()),
                events: vec![format!("{} {}", outcome.headline, outcome.details)],
            };
            self.last_report = Some(report.clone());
            return report;
        }

        let label = self.date_label();
        let starting_customers = self.player.customers;
        let starting_market_share = self.market_share();
        let mut events = Vec::new();

        self.advance_shocks(&mut events);
        self.advance_macro_environment(&mut events);
        self.maybe_spawn_startup(&mut events);
        self.complete_projects(&mut events);
        self.grow_market(&mut events);
        self.competitor_plans(&mut events);

        self.player.marketing_momentum *= 0.55;
        for competitor in &mut self.competitors {
            competitor.marketing_momentum *= 0.62;
        }

        self.player.last_quarter_customers = self.player.customers;
        for competitor in &mut self.competitors {
            competitor.last_quarter_customers = competitor.customers;
        }

        let lost_customers = self.customer_churn(&mut events);
        self.allocate_new_customers(&mut events);

        let cost_multiplier = self.cost_shock_multiplier();
        let player_finances = settle_utility(
            &mut self.player,
            &self.market,
            &self.macro_state,
            cost_multiplier,
            true,
            &mut events,
        );
        let competitor_count = self.competitors.len();
        for index in 0..competitor_count {
            let competitor = &mut self.competitors[index];
            settle_utility(
                competitor,
                &self.market,
                &self.macro_state,
                cost_multiplier,
                false,
                &mut events,
            );
        }

        self.update_stock_price(&player_finances);
        self.quarter += 1;
        if self.acquisition_cooldown > 0 {
            self.acquisition_cooldown -= 1;
        }
        self.check_outcome(&player_finances);

        let report = QuarterReport {
            label,
            revenue: player_finances.revenue,
            operating_cost: player_finances.operating_cost,
            interest: player_finances.interest,
            profit: player_finances.profit,
            new_customers: (self.player.customers - starting_customers + lost_customers).max(0.0),
            lost_customers,
            lost_customer_rate: if starting_customers > 0.0 {
                lost_customers / starting_customers
            } else {
                0.0
            },
            market_share: self.market_share(),
            prior_market_share: starting_market_share,
            events,
        };
        self.last_report = Some(report.clone());
        report
    }

    pub fn market_share(&self) -> f64 {
        let total = self.total_connected_customers();
        if total <= 0.0 {
            0.0
        } else {
            self.player.customers / total
        }
    }

    pub fn addressable_share(&self) -> f64 {
        self.player.customers / self.market.addressable_customers.max(1.0)
    }

    pub fn total_connected_customers(&self) -> f64 {
        self.player.customers
            + self
                .competitors
                .iter()
                .map(|competitor| competitor.customers)
                .sum::<f64>()
    }

    pub fn serviceable_customers(&self) -> f64 {
        self.market.serviceable_customers()
    }

    pub fn borrowing_room(&self) -> f64 {
        (self.player.asset_base * self.macro_state.borrowing_limit_ratio() - self.player.debt)
            .max(0.0)
    }

    pub fn player_annual_interest_rate(&self) -> f64 {
        self.macro_state.annual_interest_rate_for(&self.player)
    }

    pub fn acquisition_price(&self, competitor_index: usize) -> f64 {
        let competitor = &self.competitors[competitor_index];
        let customer_value = competitor.customers * 70.0;
        let generation_value = competitor.generation_capacity_mwh * 36.0;
        let line_value = competitor.distribution_capacity * 11.0;
        let market_position_value = competitor.reputation * 90.0;
        let premium = 8_000.0 + competitor.reliability * 4_000.0;
        let share = self.market_share();
        let consolidation_premium = 1.0 + share * 0.4 + share * share * 1.3;
        (customer_value + generation_value + line_value + market_position_value + premium)
            * consolidation_premium
    }

    fn require_cash(&self, amount: f64) -> Result<(), String> {
        if self.player.cash + 0.01 < amount {
            Err(format!(
                "You need {}, but cash on hand is only {}.",
                money(amount),
                money(self.player.cash)
            ))
        } else {
            Ok(())
        }
    }

    pub fn rate_frozen(&self) -> bool {
        self.shock_active(|kind| matches!(kind, ShockKind::RateFreeze))
    }

    pub fn cost_shock_multiplier(&self) -> f64 {
        if self.shock_active(|kind| matches!(kind, ShockKind::InputCostShock)) {
            1.18
        } else {
            1.0
        }
    }

    pub fn demand_shock_multiplier(&self) -> f64 {
        let mut multiplier = 1.0;
        for shock in &self.active_shocks {
            match shock.kind {
                ShockKind::DemandRecession => multiplier *= 0.55,
                ShockKind::DemandBoom => multiplier *= 1.65,
                _ => {}
            }
        }
        multiplier
    }

    fn shock_active(&self, predicate: impl Fn(&ShockKind) -> bool) -> bool {
        self.active_shocks
            .iter()
            .any(|shock| predicate(&shock.kind))
    }

    fn advance_shocks(&mut self, events: &mut Vec<String>) {
        let mut remaining = Vec::with_capacity(self.active_shocks.len());
        for mut shock in self.active_shocks.drain(..) {
            shock.quarters_remaining = shock.quarters_remaining.saturating_sub(1);
            if shock.quarters_remaining == 0 {
                events.push(shock_expired_message(&shock.kind));
            } else {
                remaining.push(shock);
            }
        }
        self.active_shocks = remaining;

        if self.rng.chance(0.045)
            && !self.shock_active(|kind| matches!(kind, ShockKind::RateFreeze))
        {
            self.active_shocks.push(ActiveShock {
                kind: ShockKind::RateFreeze,
                quarters_remaining: 2 + (self.rng.next_u64() % 2) as u32,
            });
            events.push("A market-wide rate freeze took effect.".to_string());
        }

        if self.rng.chance(0.04)
            && !self.shock_active(|kind| {
                matches!(kind, ShockKind::DemandRecession | ShockKind::DemandBoom)
            })
        {
            self.active_shocks.push(ActiveShock {
                kind: ShockKind::DemandRecession,
                quarters_remaining: 2 + (self.rng.next_u64() % 2) as u32,
            });
            events.push("A regional slowdown cut new connection prospects sharply.".to_string());
        }

        if self.rng.chance(0.035)
            && !self.shock_active(|kind| {
                matches!(kind, ShockKind::DemandBoom | ShockKind::DemandRecession)
            })
        {
            self.active_shocks.push(ActiveShock {
                kind: ShockKind::DemandBoom,
                quarters_remaining: 1 + (self.rng.next_u64() % 2) as u32,
            });
            events.push(
                "A surge in industrial demand opened a wave of new connection prospects."
                    .to_string(),
            );
        }

        if self.rng.chance(0.04)
            && !self.shock_active(|kind| matches!(kind, ShockKind::InputCostShock))
        {
            self.active_shocks.push(ActiveShock {
                kind: ShockKind::InputCostShock,
                quarters_remaining: 2,
            });
            events.push(
                "Fuel and equipment costs spiked; expect operating cost pressure.".to_string(),
            );
        }

        if self.rng.chance(0.025) {
            let capacity_loss = 0.08 + self.rng.range(0.0, 0.07);
            let lost = self.player.generation_capacity_mwh * capacity_loss;
            self.player.generation_capacity_mwh =
                (self.player.generation_capacity_mwh - lost).max(20.0);
            self.player.reliability = (self.player.reliability - 0.07).clamp(0.35, 0.98);
            self.player.reputation = (self.player.reputation - 4.0).clamp(0.0, 100.0);
            events.push(format!(
                "An equipment fire knocked {:.0} MWh/quarter of generation offline.",
                lost
            ));
        }
    }

    fn maybe_spawn_startup(&mut self, events: &mut Vec<String>) {
        if self.competitors.len() >= 5 {
            return;
        }

        let player_share = self.market_share();
        let market_avg_rate = self.average_rate();
        let unserved_demand =
            (self.market.serviceable_customers() - self.total_connected_customers()).max(0.0);
        let consolidated = player_share > 0.55;
        let high_rates = market_avg_rate > 11.2;
        let unmet_demand = unserved_demand > self.market.addressable_customers * 0.05;

        if !(consolidated || high_rates) || !unmet_demand {
            return;
        }

        let probability = if consolidated && high_rates {
            0.16
        } else if consolidated {
            0.10
        } else {
            0.07
        };

        if !self.rng.chance(probability) {
            return;
        }

        let undercut = (market_avg_rate - 0.6).clamp(8.5, 13.0);
        self.startup_index += 1;
        let name = startup_name(self.startup_index);
        let starting_customers = 30.0 + self.rng.range(0.0, 25.0);
        let startup = Utility {
            name: name.clone(),
            cash: 6_000.0 + self.rng.range(0.0, 2_500.0),
            debt: 3_500.0 + self.rng.range(0.0, 2_000.0),
            shares: 0.0,
            stock_price: 0.0,
            customers: starting_customers,
            generation_capacity_mwh: 26.0 + self.rng.range(0.0, 14.0),
            distribution_capacity: 90.0 + self.rng.range(0.0, 40.0),
            rate_cents: undercut,
            reputation: 47.0 + self.rng.range(0.0, 6.0),
            reliability: 0.74 + self.rng.range(0.0, 0.05),
            marketing_momentum: 0.10,
            asset_base: 22_000.0 + self.rng.range(0.0, 5_000.0),
            last_quarter_customers: starting_customers,
        };
        self.competitors.push(startup);
        events.push(format!(
            "{} entered the market at {:.1}c/kWh, drawing rate-sensitive customers.",
            name, undercut
        ));
    }

    fn advance_macro_environment(&mut self, events: &mut Vec<String>) {
        let old = self.macro_state.clone();

        self.macro_state.annual_base_rate = (self.macro_state.annual_base_rate
            + self.rng.range(-0.004, 0.0055))
        .clamp(0.032, 0.105);
        self.macro_state.credit_spread = (self.macro_state.credit_spread * 0.78
            + BASE_CREDIT_SPREAD * 0.22
            + self.rng.range(-0.0035, 0.0048))
        .clamp(0.008, 0.075);
        self.macro_state.demand_index =
            (self.macro_state.demand_index * 0.58 + self.rng.range(-0.32, 0.36)).clamp(-0.60, 0.70);
        self.macro_state.cost_pressure = (self.macro_state.cost_pressure * 0.66
            + self.rng.range(-0.014, 0.020))
        .clamp(-0.030, 0.060);

        if self.rng.chance(0.07) {
            self.macro_state.credit_spread =
                (self.macro_state.credit_spread + self.rng.range(0.010, 0.026)).clamp(0.008, 0.085);
        }
        if self.rng.chance(0.06) {
            self.macro_state.demand_index =
                (self.macro_state.demand_index - self.rng.range(0.20, 0.42)).clamp(-0.70, 0.70);
        }
        if self.rng.chance(0.06) {
            self.macro_state.cost_pressure = (self.macro_state.cost_pressure
                + self.rng.range(0.020, 0.045))
            .clamp(-0.030, 0.080);
        }

        let old_credit = old.benchmark_credit_rate();
        let new_credit = self.macro_state.benchmark_credit_rate();
        if new_credit - old_credit > 0.012 {
            events.push(format!(
                "Credit markets tightened; benchmark debt now prices near {:.1}% before leverage premiums.",
                new_credit * 100.0
            ));
        } else if old_credit - new_credit > 0.010 {
            events.push(format!(
                "Credit markets eased; benchmark debt now prices near {:.1}% before leverage premiums.",
                new_credit * 100.0
            ));
        }

        if self.macro_state.demand_index < -0.42 && old.demand_index >= -0.42 {
            events.push("Demand conditions weakened across the service territory.".to_string());
        } else if self.macro_state.demand_index > 0.45 && old.demand_index <= 0.45 {
            events.push("Demand conditions strengthened across the service territory.".to_string());
        }

        if self.macro_state.cost_pressure > 0.045 && old.cost_pressure <= 0.045 {
            events.push(
                "Input costs moved sharply higher for fuel, labor, and equipment.".to_string(),
            );
        }
    }

    fn complete_projects(&mut self, events: &mut Vec<String>) {
        let mut remaining = Vec::new();
        for mut project in self.pending_projects.drain(..) {
            project.quarters_remaining = project.quarters_remaining.saturating_sub(1);
            if project.quarters_remaining == 0 {
                match project.kind {
                    ProjectKind::Generation { capacity_mwh } => {
                        self.player.generation_capacity_mwh += capacity_mwh;
                        self.player.reliability =
                            (self.player.reliability + 0.025).clamp(0.35, 0.98);
                        events.push(format!(
                            "Your {} came online, adding {:.0} MWh/quarter of generating capacity.",
                            project.name, capacity_mwh
                        ));
                    }
                    ProjectKind::Distribution { customer_capacity } => {
                        self.player.distribution_capacity += customer_capacity;
                        self.player.reputation = (self.player.reputation + 1.2).clamp(0.0, 100.0);
                        events.push(format!(
                            "The {} opened service to about {:.0} additional customers.",
                            project.name, customer_capacity
                        ));
                    }
                }
            } else {
                remaining.push(project);
            }
        }
        self.pending_projects = remaining;
    }

    fn grow_market(&mut self, events: &mut Vec<String>) {
        let demand_cycle = self.macro_state.demand_index;
        let address_growth = 1.006 + self.rng.range(0.0, 0.006) + demand_cycle * 0.003;
        self.market.addressable_customers *= address_growth;

        let adoption_gain = (0.015
            + self.rng.range(0.0, 0.007)
            + (self.player.reputation - 50.0).max(0.0) / 7_000.0
            + demand_cycle * 0.010)
            .clamp(0.004, 0.034);
        self.market.electrification =
            (self.market.electrification + adoption_gain).clamp(0.05, 0.68);
        self.market.avg_mwh_per_customer *=
            (1.004 + self.rng.range(0.0, 0.004) + demand_cycle * 0.002).clamp(0.996, 1.012);
        self.market.variable_cost_per_mwh *=
            (1.0 + self.rng.range(-0.008, 0.010) + self.macro_state.cost_pressure)
                .clamp(0.965, 1.085);

        if self.rng.chance(0.18) {
            events.push(
                "Demand rose as more homes and businesses shifted to electric service.".to_string(),
            );
        }
    }

    fn competitor_plans(&mut self, events: &mut Vec<String>) {
        let player_rate = self.player.rate_cents;
        let player_share = self.market_share();
        let market = self.market.clone();
        let demand_signal = self.macro_state.demand_index;
        let competitor_count = self.competitors.len();
        let rate_frozen = self.rate_frozen();

        let total_now = self.total_connected_customers().max(1.0);
        let total_last = (self.player.last_quarter_customers
            + self
                .competitors
                .iter()
                .map(|competitor| competitor.last_quarter_customers)
                .sum::<f64>())
        .max(1.0);

        for index in 0..competitor_count {
            let routine_marketing_chance = self.rng.chance(0.24);
            let routine_marketing_jitter = self.rng.range(0.0, 2_000.0);
            let expansion_cost_jitter = self.rng.range(0.0, 2_000.0);
            let expansion_lines_jitter = self.rng.range(0.0, 45.0);
            let expansion_gen_jitter = self.rng.range(0.0, 30.0);
            let proactive_expansion_chance = self.rng.chance(0.22);
            let strategic_debt_chance = self.rng.chance(0.32);

            let competitor = &mut self.competitors[index];

            let share_now = competitor.customers / total_now;
            let share_last = competitor.last_quarter_customers / total_last;
            let lost_share =
                competitor.last_quarter_customers > 0.0 && share_now < share_last - 0.005;
            let player_undercutting = player_rate + 0.4 < competitor.rate_cents;
            let player_premium_pricing = player_rate > competitor.rate_cents + 0.5;
            let near_capacity = competitor.capacity_headroom(&market) < competitor.customers * 0.06;

            if lost_share && player_undercutting && competitor.rate_cents > 8.4 {
                let cut = 0.4_f64.min(competitor.rate_cents - 8.4);
                if cut > 0.0 {
                    competitor.rate_cents -= cut;
                    events.push(format!(
                        "{} cut rates to {:.1}c/kWh to defend share.",
                        competitor.name, competitor.rate_cents
                    ));
                }
            } else if !rate_frozen
                && player_premium_pricing
                && near_capacity
                && competitor.rate_cents < 13.4
            {
                let raise = 0.3_f64.min(13.5 - competitor.rate_cents);
                if raise > 0.0 {
                    competitor.rate_cents += raise;
                    events.push(format!(
                        "{} raised rates to {:.1}c/kWh, riding strong demand.",
                        competitor.name, competitor.rate_cents
                    ));
                }
            }

            if lost_share && competitor.cash > 4_500.0 {
                let spend = (competitor.cash * 0.18).clamp(2_500.0, 6_500.0);
                competitor.cash -= spend;
                competitor.marketing_momentum += spend / 12_000.0;
                competitor.reputation = (competitor.reputation + spend / 5_500.0).clamp(0.0, 100.0);
                events.push(format!(
                    "{} launched a retention campaign to win back customers.",
                    competitor.name
                ));
            } else if routine_marketing_chance && competitor.cash > 2_500.0 {
                let spend = 1_600.0 + routine_marketing_jitter;
                competitor.cash -= spend;
                competitor.marketing_momentum += spend / 18_000.0;
                competitor.reputation = (competitor.reputation + spend / 7_000.0).clamp(0.0, 100.0);
            }

            let headroom = competitor.capacity_headroom(&market);
            let reactive_pressure = headroom < market.addressable_customers * 0.025;
            let proactive_growth =
                proactive_expansion_chance && (demand_signal > 0.10 || player_share > 0.40);
            if (reactive_pressure || proactive_growth) && competitor.cash > 8_500.0 {
                let line_cost = 6_000.0 + expansion_cost_jitter;
                competitor.cash -= line_cost;
                competitor.asset_base += line_cost;
                competitor.distribution_capacity += 115.0 + expansion_lines_jitter;
                competitor.generation_capacity_mwh += 42.0 + expansion_gen_jitter;
                events.push(format!(
                    "{} expanded generation and distribution capacity.",
                    competitor.name
                ));
            }

            let leverage = competitor.debt / competitor.asset_base.max(1.0);
            let losing_to_player = player_share > 0.45 && lost_share;
            if losing_to_player && leverage < 0.65 && strategic_debt_chance {
                let raise = 12_000.0 + competitor.asset_base * 0.05;
                competitor.cash += raise;
                competitor.debt += raise;
                events.push(format!(
                    "{} raised debt to fund a counter-expansion.",
                    competitor.name
                ));
            }

            if competitor.reliability < 0.78 && competitor.cash > 3_500.0 {
                let spend = 4_000.0_f64.min(competitor.cash * 0.30);
                competitor.cash -= spend;
                let gain = maintenance_reliability_gain(competitor, spend);
                competitor.reliability = (competitor.reliability + gain).clamp(0.35, 0.98);
            }
        }
    }

    fn customer_churn(&mut self, events: &mut Vec<String>) -> f64 {
        let player_alternative_rate = self.rival_average_rate_for_player();
        let competitor_alternative_rates = (0..self.competitors.len())
            .map(|index| self.alternative_rate_for_competitor(index))
            .collect::<Vec<_>>();

        let player_loss = churn_for(&self.player, player_alternative_rate);
        self.player.customers -= player_loss;

        let mut churn_pool = player_loss;
        for (competitor, alternative_rate) in self
            .competitors
            .iter_mut()
            .zip(competitor_alternative_rates)
        {
            let loss = churn_for(competitor, alternative_rate);
            competitor.customers -= loss;
            churn_pool += loss;
        }

        if churn_pool > 0.0 {
            self.allocate_customers(churn_pool, events, "switched accounts");
        }

        player_loss
    }

    fn rival_average_rate_for_player(&self) -> f64 {
        weighted_rate(
            self.competitors
                .iter()
                .map(|competitor| (competitor.rate_cents, competitor.customers)),
            self.market.standard_rate_cents,
        )
    }

    fn alternative_rate_for_competitor(&self, excluded_index: usize) -> f64 {
        let rivals = std::iter::once((self.player.rate_cents, self.player.customers)).chain(
            self.competitors
                .iter()
                .enumerate()
                .filter_map(|(index, competitor)| {
                    if index == excluded_index {
                        None
                    } else {
                        Some((competitor.rate_cents, competitor.customers))
                    }
                }),
        );
        weighted_rate(rivals, self.market.standard_rate_cents)
    }

    fn allocate_new_customers(&mut self, events: &mut Vec<String>) {
        let serviceable = self.serviceable_customers();
        let connected = self.total_connected_customers();
        let natural_demand = (serviceable - connected).max(0.0);
        let base_prospects =
            (natural_demand * 0.36 + self.market.addressable_customers * 0.006).max(18.0);
        let prospects = base_prospects * self.demand_shock_multiplier();
        self.allocate_customers(prospects, events, "new connections");
    }

    fn allocate_customers(&mut self, prospects: f64, events: &mut Vec<String>, label: &str) {
        if prospects <= 0.0 {
            return;
        }

        let scores = self.competition_scores();
        let score_total: f64 = scores.iter().sum();
        if score_total <= 0.0 {
            return;
        }

        let mut won = Vec::with_capacity(scores.len());
        for score in &scores {
            won.push(prospects * *score / score_total);
        }

        let player_headroom = self.player.capacity_headroom(&self.market);
        let player_gain = won[0].min(player_headroom.max(0.0));
        self.player.customers += player_gain;

        for (index, competitor) in self.competitors.iter_mut().enumerate() {
            let headroom = competitor.capacity_headroom(&self.market);
            let gain = won[index + 1].min(headroom.max(0.0));
            competitor.customers += gain;
        }

        if player_gain >= 8.0 {
            events.push(format!(
                "Your sales and service teams captured {:.0} {label}.",
                player_gain
            ));
        }
    }

    fn competition_scores(&self) -> Vec<f64> {
        let average_rate = self.average_rate();
        let mut scores = Vec::with_capacity(self.competitors.len() + 1);
        scores.push(self.player.competition_score(&self.market, average_rate));
        for competitor in &self.competitors {
            scores.push(competitor.competition_score(&self.market, average_rate));
        }
        scores
    }

    fn average_rate(&self) -> f64 {
        let mut weighted = self.player.rate_cents * self.player.customers;
        let mut customers = self.player.customers;
        for competitor in &self.competitors {
            weighted += competitor.rate_cents * competitor.customers;
            customers += competitor.customers;
        }
        if customers <= 0.0 {
            self.market.standard_rate_cents
        } else {
            weighted / customers
        }
    }

    fn update_stock_price(&mut self, finances: &FirmFinances) {
        let shares = self.player.shares.max(1.0);
        let book_equity =
            (self.player.asset_base + self.player.cash - self.player.debt).max(shares);
        let book_value_per_share = book_equity / shares;

        let annualized_profit = finances.profit * 4.0;
        let multiple = self.earnings_multiple();
        let earnings_value_per_share = annualized_profit.max(0.0) * multiple / shares;

        let fundamental_price =
            (book_value_per_share * 0.5 + earnings_value_per_share * 0.5).max(1.0);
        let gap = fundamental_price - self.player.stock_price;
        let reverted = self.player.stock_price + gap * 0.22;

        let macro_signal = self.macro_state.valuation_signal();
        let noise = self.rng.range(-0.025, 0.025);
        self.player.stock_price = (reverted * (1.0 + macro_signal + noise)).max(1.0);
    }

    fn earnings_multiple(&self) -> f64 {
        let base = 7.0;
        let growth_bonus = (self.player.customers / 250.0).clamp(0.0, 5.0);
        let reliability_bonus = (self.player.reliability - 0.78) * 4.0;
        let leverage_drag = (self.player.debt_to_assets() - 0.65).max(0.0) * 5.0;
        let macro_drag = (self.macro_state.benchmark_credit_rate() - 0.07) * 30.0;
        (base + growth_bonus + reliability_bonus - leverage_drag - macro_drag).clamp(2.5, 18.0)
    }

    fn check_outcome(&mut self, finances: &FirmFinances) {
        if self.outcome.is_some() {
            return;
        }

        if self.player.reliability < 0.44 {
            self.outcome = Some(Outcome {
                kind: OutcomeKind::Defeat,
                headline: "Market Access Lost".to_string(),
                details: "Repeated outages pushed regulators and lenders to move the company into managed restructuring."
                    .to_string(),
            });
            return;
        }

        if self.quarter > 5 && self.player.debt_to_assets() > 1.22 && finances.profit < 0.0 {
            self.outcome = Some(Outcome {
                kind: OutcomeKind::Defeat,
                headline: "Bankers Forced Receivership".to_string(),
                details: "The company could not finance expansion and interest at the same time."
                    .to_string(),
            });
            return;
        }

        if self.quarter >= self.campaign_quarters {
            let share = self.market_share();
            let healthy_balance_sheet = self.player.debt_to_assets() <= 0.95;
            if share >= 0.45 && self.player.reliability >= 0.72 && healthy_balance_sheet {
                self.outcome = Some(Outcome {
                    kind: OutcomeKind::Victory,
                    headline: "Market Lead Secured".to_string(),
                    details: format!(
                        "You finished with {:.0}% of connected accounts, solid reliability, and a balance sheet lenders can still underwrite.",
                        share * 100.0
                    ),
                });
            } else {
                self.outcome = Some(Outcome {
                    kind: OutcomeKind::Defeat,
                    headline: "Board Lost Confidence".to_string(),
                    details: format!(
                        "After Year 5 you held {:.0}% of connected accounts. The board wanted a clearer path to durable market leadership.",
                        share * 100.0
                    ),
                });
            }
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl InitialVariance {
    pub const fn fixed() -> Self {
        Self { amplitude: 0.0 }
    }

    pub const fn new(amplitude: f64) -> Self {
        Self { amplitude }
    }

    fn scale(self) -> f64 {
        self.amplitude.clamp(0.0, 2.0)
    }
}

impl Default for InitialVariance {
    fn default() -> Self {
        Self::new(1.0)
    }
}

impl Market {
    pub fn serviceable_customers(&self) -> f64 {
        self.addressable_customers * self.electrification
    }
}

impl MacroEnvironment {
    pub fn benchmark_credit_rate(&self) -> f64 {
        self.annual_base_rate + self.credit_spread
    }

    pub fn annual_interest_rate_for(&self, utility: &Utility) -> f64 {
        let leverage = utility.debt_to_assets();
        let leverage_premium = (leverage - 0.45).max(0.0).powf(1.35) * 0.18;
        let distress_premium = (leverage - 0.85).max(0.0) * 0.34;
        (self.benchmark_credit_rate() + leverage_premium + distress_premium).clamp(0.035, 0.28)
    }

    pub fn borrowing_limit_ratio(&self) -> f64 {
        let rate_stress = (self.annual_base_rate - BASE_ANNUAL_RATE).max(0.0) * 2.0;
        let spread_stress = (self.credit_spread - BASE_CREDIT_SPREAD).max(0.0) * 3.0;
        (0.97 - rate_stress - spread_stress).clamp(0.66, 0.98)
    }

    pub fn financing_pressure(&self) -> f64 {
        let credit_pressure = (self.benchmark_credit_rate() - 0.07) / 0.10;
        let demand_pressure = -self.demand_index * 0.30;
        (credit_pressure + demand_pressure).clamp(0.0, 1.0)
    }

    pub fn valuation_signal(&self) -> f64 {
        let rate_drag = (self.benchmark_credit_rate() - 0.07) * 0.35;
        let demand_signal = self.demand_index * 0.025;
        let cost_drag = self.cost_pressure * 0.35;
        (demand_signal - rate_drag - cost_drag).clamp(-0.045, 0.035)
    }

    pub fn credit_label(&self) -> &'static str {
        let rate = self.benchmark_credit_rate();
        if rate >= 0.105 {
            "tight"
        } else if rate <= 0.055 {
            "easy"
        } else {
            "normal"
        }
    }

    pub fn demand_label(&self) -> &'static str {
        if self.demand_index >= 0.25 {
            "strong"
        } else if self.demand_index <= -0.25 {
            "soft"
        } else {
            "steady"
        }
    }

    pub fn cost_label(&self) -> &'static str {
        if self.cost_pressure >= 0.030 {
            "rising"
        } else if self.cost_pressure <= -0.012 {
            "falling"
        } else {
            "stable"
        }
    }
}

impl Utility {
    pub fn market_cap(&self) -> f64 {
        self.shares * self.stock_price
    }

    pub fn debt_to_assets(&self) -> f64 {
        self.debt / self.asset_base.max(1.0)
    }

    pub fn customer_capacity(&self, market: &Market) -> f64 {
        let generation_customer_capacity =
            self.generation_capacity_mwh / market.avg_mwh_per_customer.max(0.05);
        self.distribution_capacity.min(generation_customer_capacity)
    }

    pub fn capacity_headroom(&self, market: &Market) -> f64 {
        self.customer_capacity(market) - self.customers
    }

    pub fn utilization(&self, market: &Market) -> f64 {
        let demanded = self.customers * market.avg_mwh_per_customer;
        demanded / self.generation_capacity_mwh.max(1.0)
    }

    pub fn demanded_mwh(&self, market: &Market) -> f64 {
        self.customers * market.avg_mwh_per_customer
    }

    pub fn generation_reserve_mwh(&self, market: &Market) -> f64 {
        self.generation_capacity_mwh - self.demanded_mwh(market)
    }

    pub fn firm_generation_reserve_mwh(&self, market: &Market) -> f64 {
        self.generation_capacity_mwh * self.reliability.max(0.45) - self.demanded_mwh(market)
    }

    fn competition_score(&self, market: &Market, average_rate: f64) -> f64 {
        let rate_component = ((average_rate - self.rate_cents) * 0.16).clamp(-0.60, 0.55);
        let reliability_component = (self.reliability - 0.74) * 1.65;
        let reputation_component = (self.reputation - 50.0) / 85.0;
        let capacity_component = (self.capacity_headroom(market)
            / (market.addressable_customers * 0.08))
            .clamp(-0.35, 0.45);
        let marketing_component = self.marketing_momentum.clamp(0.0, 0.85);
        (1.0 + rate_component
            + reliability_component
            + reputation_component
            + capacity_component
            + marketing_component)
            .max(0.05)
    }
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.state
    }

    fn next_f64(&mut self) -> f64 {
        let value = self.next_u64() >> 11;
        value as f64 / ((1_u64 << 53) as f64)
    }

    fn range(&mut self, min: f64, max: f64) -> f64 {
        min + (max - min) * self.next_f64()
    }

    fn chance(&mut self, probability: f64) -> bool {
        self.next_f64() < probability.clamp(0.0, 1.0)
    }
}

fn fresh_seed() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0xE1EC_0001)
        ^ 0x9E37_79B9_7F4A_7C15
}

fn starting_utility(
    name: &str,
    cash: f64,
    debt: f64,
    shares: f64,
    stock_price: f64,
    customers: f64,
    generation_capacity_mwh: f64,
    distribution_capacity: f64,
    rate_cents: f64,
    reputation: f64,
    reliability: f64,
    marketing_momentum: f64,
    asset_base: f64,
    initial_variance: InitialVariance,
    rng: &mut Rng,
) -> Utility {
    let varied_debt = varied_pct(rng, debt, 0.18, initial_variance).max(0.0);
    let varied_customers = varied_pct(rng, customers, 0.12, initial_variance)
        .round()
        .max(20.0);
    let varied_asset_base =
        varied_pct(rng, asset_base, 0.12, initial_variance).max(varied_debt * 1.8);

    Utility {
        name: name.to_string(),
        cash: varied_pct(rng, cash, 0.18, initial_variance).max(1_500.0),
        debt: varied_debt,
        shares,
        stock_price: varied_pct(rng, stock_price, 0.12, initial_variance).max(stock_price.min(1.0)),
        customers: varied_customers,
        generation_capacity_mwh: varied_pct(rng, generation_capacity_mwh, 0.10, initial_variance)
            .max(20.0),
        distribution_capacity: varied_pct(rng, distribution_capacity, 0.12, initial_variance)
            .max(50.0),
        rate_cents: varied_abs(rng, rate_cents, 0.35, initial_variance).clamp(7.5, 14.5),
        reputation: varied_abs(rng, reputation, 6.0, initial_variance).clamp(20.0, 85.0),
        reliability: varied_abs(rng, reliability, 0.045, initial_variance).clamp(0.55, 0.94),
        marketing_momentum: varied_abs(rng, marketing_momentum, 0.025, initial_variance)
            .clamp(0.0, 0.18),
        asset_base: varied_asset_base,
        last_quarter_customers: varied_customers,
    }
}

fn varied_pct(rng: &mut Rng, base: f64, percent: f64, initial_variance: InitialVariance) -> f64 {
    base * (1.0 + rng.range(-percent, percent) * initial_variance.scale())
}

fn varied_abs(rng: &mut Rng, base: f64, amount: f64, initial_variance: InitialVariance) -> f64 {
    base + rng.range(-amount, amount) * initial_variance.scale()
}

fn maintenance_reliability_gain(utility: &Utility, spend: f64) -> f64 {
    (spend / 22_000.0)
        * (1.05_f64 - utility.reliability).max(0.05)
        * maintenance_asset_scale(utility)
}

fn maintenance_reputation_gain(utility: &Utility, spend: f64) -> f64 {
    spend / 5_500.0 * maintenance_asset_scale(utility)
}

fn maintenance_asset_scale(utility: &Utility) -> f64 {
    (MAINTENANCE_REFERENCE_ASSET_BASE / utility.asset_base.max(20_000.0))
        .sqrt()
        .clamp(0.35, 1.45)
}

fn settle_utility(
    utility: &mut Utility,
    market: &Market,
    macro_state: &MacroEnvironment,
    cost_multiplier: f64,
    is_player: bool,
    events: &mut Vec<String>,
) -> FirmFinances {
    let demanded_mwh = utility.customers * market.avg_mwh_per_customer;
    let served_mwh =
        demanded_mwh.min(utility.generation_capacity_mwh * utility.reliability.max(0.45));
    let unmet_demand_ratio = if demanded_mwh <= 0.0 {
        0.0
    } else {
        1.0 - served_mwh / demanded_mwh
    };

    let revenue = served_mwh * utility.rate_cents * 10.0;
    let maintenance_load = if utility.reliability < 0.72 {
        1.14
    } else {
        1.0
    };
    let operating_cost = (served_mwh * market.variable_cost_per_mwh * maintenance_load
        + 850.0
        + utility.customers * 2.65
        + utility.asset_base * 0.0105)
        * cost_multiplier;
    let interest = utility.debt * macro_state.annual_interest_rate_for(utility) / 4.0;
    let profit = revenue - operating_cost - interest;

    utility.cash += profit;
    if utility.cash < 0.0 {
        utility.debt += -utility.cash;
        utility.cash = 0.0;
        utility.reputation = (utility.reputation - 1.8).clamp(0.0, 100.0);
    }

    utility.reliability = (utility.reliability - 0.014).clamp(0.35, 0.98);

    if unmet_demand_ratio > 0.04 {
        utility.reliability = (utility.reliability - unmet_demand_ratio * 0.22).clamp(0.35, 0.98);
        utility.reputation = (utility.reputation - unmet_demand_ratio * 10.0).clamp(0.0, 100.0);
        if is_player {
            events.push(format!(
                "Overloaded generation left {:.0}% of demand unmet and hurt public confidence.",
                unmet_demand_ratio * 100.0
            ));
        }
    } else {
        utility.reliability = (utility.reliability + 0.004).clamp(0.35, 0.98);
    }

    FirmFinances {
        revenue,
        operating_cost,
        interest,
        profit,
        served_mwh,
        unmet_demand_ratio,
    }
}

fn churn_for(utility: &Utility, average_rate: f64) -> f64 {
    utility.customers * churn_rate_for(utility, average_rate)
}

fn churn_rate_for(utility: &Utility, average_rate: f64) -> f64 {
    let rate_gap = utility.rate_cents - average_rate;
    let rate_pressure = rate_gap.max(0.0) * 0.006;
    let rate_retention = (-rate_gap).max(0.0) * 0.0025;
    let outage_pressure = (0.78 - utility.reliability).max(0.0) * 0.055;
    let reliability_retention = (utility.reliability - 0.84).max(0.0) * 0.020;
    let reputation_pressure = (48.0 - utility.reputation).max(0.0) * 0.0008;
    let reputation_retention = (utility.reputation - 62.0).max(0.0) * 0.00012;
    let churn_rate = 0.010 + rate_pressure + outage_pressure + reputation_pressure;
    (churn_rate - rate_retention - reliability_retention - reputation_retention).clamp(0.004, 0.075)
}

fn weighted_rate(rates: impl Iterator<Item = (f64, f64)>, fallback: f64) -> f64 {
    let mut weighted = 0.0;
    let mut customers = 0.0;
    for (rate, customer_count) in rates {
        weighted += rate * customer_count;
        customers += customer_count;
    }
    if customers <= 0.0 {
        fallback
    } else {
        weighted / customers
    }
}

fn shock_expired_message(kind: &ShockKind) -> String {
    match kind {
        ShockKind::RateFreeze => "The market-wide rate freeze expired.".to_string(),
        ShockKind::DemandRecession => {
            "Demand conditions normalized after the slowdown.".to_string()
        }
        ShockKind::DemandBoom => "The industrial demand surge tapered off.".to_string(),
        ShockKind::InputCostShock => "Fuel and equipment cost pressure eased.".to_string(),
    }
}

fn startup_name(index: u32) -> String {
    const NAMES: &[&str] = &[
        "Bright Path Power",
        "Voltaic Cooperative",
        "Switchback Energy",
        "Civic Spark",
        "New Current Co.",
        "Clear Grid Partners",
        "Greenline Utility",
        "Halcyon Power",
    ];
    NAMES[((index as usize).saturating_sub(1)) % NAMES.len()].to_string()
}

fn weighted_average(left: f64, left_weight: f64, right: f64, right_weight: f64) -> f64 {
    let total = left_weight + right_weight;
    if total <= 0.0 {
        left
    } else {
        (left * left_weight + right * right_weight) / total
    }
}

fn positive_amount(amount: f64, label: &str) -> Result<f64, String> {
    if amount.is_finite() && amount > 0.0 {
        Ok(amount)
    } else {
        Err(format!("The {label} amount must be positive."))
    }
}

pub fn money(value: f64) -> String {
    if value.abs() >= 1_000.0 {
        format!("${:.1}k", value / 1_000.0)
    } else {
        format!("${value:.0}")
    }
}

pub fn generation_project_cost(capacity_mwh: f64) -> f64 {
    scaled_project_cost(
        capacity_mwh,
        GENERATION_PROJECT_CAPACITY_MWH,
        GENERATION_PROJECT_COST,
        0.82,
        MIN_GENERATION_PROJECT_MWH,
        MAX_GENERATION_PROJECT_MWH,
    )
}

pub fn distribution_project_cost(customer_capacity: f64) -> f64 {
    scaled_project_cost(
        customer_capacity,
        DISTRIBUTION_PROJECT_CAPACITY,
        DISTRIBUTION_PROJECT_COST,
        0.84,
        MIN_DISTRIBUTION_PROJECT_CUSTOMERS,
        MAX_DISTRIBUTION_PROJECT_CUSTOMERS,
    )
}

pub fn generation_project_duration(capacity_mwh: f64) -> u32 {
    let capacity_mwh = capacity_mwh.clamp(MIN_GENERATION_PROJECT_MWH, MAX_GENERATION_PROJECT_MWH);
    if capacity_mwh <= GENERATION_PROJECT_CAPACITY_MWH * 1.35 {
        2
    } else if capacity_mwh <= GENERATION_PROJECT_CAPACITY_MWH * 2.40 {
        3
    } else {
        4
    }
}

pub fn distribution_project_duration(customer_capacity: f64) -> u32 {
    let customer_capacity = customer_capacity.clamp(
        MIN_DISTRIBUTION_PROJECT_CUSTOMERS,
        MAX_DISTRIBUTION_PROJECT_CUSTOMERS,
    );
    if customer_capacity <= DISTRIBUTION_PROJECT_CAPACITY * 1.50 {
        1
    } else if customer_capacity <= DISTRIBUTION_PROJECT_CAPACITY * 2.75 {
        2
    } else {
        3
    }
}

fn scaled_project_cost(
    requested: f64,
    default_size: f64,
    default_cost: f64,
    scale_exponent: f64,
    min_size: f64,
    max_size: f64,
) -> f64 {
    let requested = requested.clamp(min_size, max_size);
    default_cost * (requested / default_size).powf(scale_exponent)
}

#[cfg(test)]
mod tests {
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
        assert!(
            tight.macro_state.borrowing_limit_ratio() < normal.macro_state.borrowing_limit_ratio()
        );
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
}
