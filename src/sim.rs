const DEFAULT_CAMPAIGN_QUARTERS: u32 = 20;
pub const GENERATION_PROJECT_COST: f64 = 18_500.0;
pub const GENERATION_PROJECT_CAPACITY_MWH: f64 = 185.0;
pub const DISTRIBUTION_PROJECT_COST: f64 = 9_500.0;
pub const DISTRIBUTION_PROJECT_CAPACITY: f64 = 230.0;
pub const MIN_GENERATION_PROJECT_MWH: f64 = 60.0;
pub const MAX_GENERATION_PROJECT_MWH: f64 = 650.0;
pub const MIN_DISTRIBUTION_PROJECT_CUSTOMERS: f64 = 80.0;
pub const MAX_DISTRIBUTION_PROJECT_CUSTOMERS: f64 = 900.0;

#[derive(Clone, Debug)]
pub struct Game {
    pub quarter: u32,
    pub campaign_quarters: u32,
    pub market: Market,
    pub player: Utility,
    pub competitors: Vec<Utility>,
    pub pending_projects: Vec<Project>,
    pub acquisition_cooldown: u32,
    pub last_report: Option<QuarterReport>,
    pub outcome: Option<Outcome>,
    rng: Rng,
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
    pub market_share: f64,
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
        Self::with_seed(0xE1EC_0001)
    }

    pub fn with_seed(seed: u64) -> Self {
        let market = Market {
            territory: "Core Market".to_string(),
            start_year: 1,
            addressable_customers: 6_500.0,
            electrification: 0.14,
            avg_mwh_per_customer: 0.22,
            variable_cost_per_mwh: 26.0,
            standard_rate_cents: 10.4,
            civic_patience: 0.76,
        };

        Self {
            quarter: 0,
            campaign_quarters: DEFAULT_CAMPAIGN_QUARTERS,
            market,
            player: Utility {
                name: "Metro Consolidated".to_string(),
                cash: 34_000.0,
                debt: 20_000.0,
                shares: 2_000.0,
                stock_price: 24.0,
                customers: 160.0,
                generation_capacity_mwh: 72.0,
                distribution_capacity: 275.0,
                rate_cents: 10.5,
                reputation: 55.0,
                reliability: 0.82,
                marketing_momentum: 0.08,
                asset_base: 78_000.0,
            },
            competitors: vec![
                Utility {
                    name: "North Loop Power".to_string(),
                    cash: 18_500.0,
                    debt: 12_000.0,
                    shares: 0.0,
                    stock_price: 0.0,
                    customers: 260.0,
                    generation_capacity_mwh: 118.0,
                    distribution_capacity: 360.0,
                    rate_cents: 10.1,
                    reputation: 58.0,
                    reliability: 0.81,
                    marketing_momentum: 0.04,
                    asset_base: 96_000.0,
                },
                Utility {
                    name: "District Current".to_string(),
                    cash: 12_000.0,
                    debt: 8_000.0,
                    shares: 0.0,
                    stock_price: 0.0,
                    customers: 210.0,
                    generation_capacity_mwh: 95.0,
                    distribution_capacity: 300.0,
                    rate_cents: 9.8,
                    reputation: 52.0,
                    reliability: 0.76,
                    marketing_momentum: 0.05,
                    asset_base: 71_000.0,
                },
                Utility {
                    name: "Metro Light".to_string(),
                    cash: 7_500.0,
                    debt: 6_500.0,
                    shares: 0.0,
                    stock_price: 0.0,
                    customers: 110.0,
                    generation_capacity_mwh: 50.0,
                    distribution_capacity: 160.0,
                    rate_cents: 11.2,
                    reputation: 43.0,
                    reliability: 0.71,
                    marketing_momentum: 0.02,
                    asset_base: 42_000.0,
                },
            ],
            pending_projects: Vec::new(),
            acquisition_cooldown: 0,
            last_report: None,
            outcome: None,
            rng: Rng::new(seed),
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
                let issue_discount = (0.03 + pressure * 0.08).clamp(0.03, 0.35);
                let issue_price = (self.player.stock_price * (1.0 - issue_discount)).max(1.0);
                let new_shares = amount / issue_price;
                let underwriting_cost = amount * (0.025 + pressure.min(4.0) * 0.015);
                let net_proceeds = amount - underwriting_cost;
                let post_money_price = (old_market_cap + net_proceeds) / (old_shares + new_shares);
                let signal_drag = (pressure * 0.025).clamp(0.0, 0.12);
                self.player.cash += net_proceeds;
                self.player.shares += new_shares;
                self.player.stock_price = (post_money_price * (1.0 - signal_drag)).clamp(6.0, 85.0);
                self.player.reputation =
                    (self.player.reputation - pressure.min(2.5) * 2.0).clamp(0.0, 100.0);
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
                self.player.stock_price =
                    (theoretical_price * (1.0 + confidence_lift)).clamp(6.0, 85.0);
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
                let debt_capacity = self.player.asset_base * 0.95 - self.player.debt;
                if amount > debt_capacity {
                    return Err(format!(
                        "Bankers will only extend about {} more against the present asset base.",
                        money(debt_capacity.max(0.0))
                    ));
                }
                self.player.cash += amount;
                self.player.debt += amount;
                let leverage = self.player.debt_to_assets();
                if leverage > 0.72 {
                    self.player.reputation = (self.player.reputation - 1.8).clamp(0.0, 100.0);
                }
                Ok(format!(
                    "Borrowed {} at a floating 7 percent annual rate. Debt/assets now {:.0}%.",
                    money(amount),
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
                Ok(format!(
                    "Repaid {} of debt. Debt/assets now {:.0}%.",
                    money(payment),
                    self.player.debt_to_assets() * 100.0
                ))
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
                self.player.cash -= price;
                self.player.debt += acquired.debt * 0.65;
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
                    "Acquired {acquired_name} for {}. Integration losses trimmed its customer book, but its capacity and accounts are now part of your network.",
                    money(price)
                ))
            }
            Decision::AdjustRate { delta_cents } => {
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
                self.player.reliability =
                    (self.player.reliability + spend / 28_000.0).clamp(0.35, 0.98);
                self.player.reputation =
                    (self.player.reputation + spend / 5_500.0).clamp(0.0, 100.0);
                Ok(format!(
                    "Spent {} on reliability work, spare parts, and service response.",
                    money(spend)
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
                market_share: self.market_share(),
                events: vec![format!("{} {}", outcome.headline, outcome.details)],
            };
            self.last_report = Some(report.clone());
            return report;
        }

        let label = self.date_label();
        let starting_customers = self.player.customers;
        let mut events = Vec::new();

        self.complete_projects(&mut events);
        self.grow_market(&mut events);
        self.competitor_plans(&mut events);

        self.player.marketing_momentum *= 0.55;
        for competitor in &mut self.competitors {
            competitor.marketing_momentum *= 0.62;
        }

        let lost_customers = self.customer_churn(&mut events);
        self.allocate_new_customers(&mut events);

        let player_finances = settle_utility(&mut self.player, &self.market, true, &mut events);
        let competitor_count = self.competitors.len();
        for index in 0..competitor_count {
            let competitor = &mut self.competitors[index];
            settle_utility(competitor, &self.market, false, &mut events);
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
            market_share: self.market_share(),
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

    pub fn acquisition_price(&self, competitor_index: usize) -> f64 {
        let competitor = &self.competitors[competitor_index];
        let customer_value = competitor.customers * 70.0;
        let generation_value = competitor.generation_capacity_mwh * 36.0;
        let line_value = competitor.distribution_capacity * 11.0;
        let market_position_value = competitor.reputation * 90.0;
        let premium = 8_000.0 + competitor.reliability * 4_000.0;
        let consolidation_premium = 1.0 + self.market_share() * 0.55;
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
        let address_growth = 1.006 + self.rng.range(0.0, 0.006);
        self.market.addressable_customers *= address_growth;

        let adoption_gain =
            0.015 + self.rng.range(0.0, 0.007) + (self.player.reputation - 50.0).max(0.0) / 7_000.0;
        self.market.electrification =
            (self.market.electrification + adoption_gain).clamp(0.05, 0.68);
        self.market.avg_mwh_per_customer *= 1.004 + self.rng.range(0.0, 0.004);
        self.market.variable_cost_per_mwh *= 1.0 + self.rng.range(-0.010, 0.014);

        if self.rng.chance(0.18) {
            events.push(
                "Demand rose as more homes and businesses shifted to electric service.".to_string(),
            );
        }
    }

    fn competitor_plans(&mut self, events: &mut Vec<String>) {
        for competitor in &mut self.competitors {
            let headroom = competitor.capacity_headroom(&self.market);
            let customer_pressure = headroom < self.market.addressable_customers * 0.025;
            if customer_pressure && competitor.cash > 8_500.0 {
                let line_cost = 6_000.0 + self.rng.range(0.0, 2_000.0);
                competitor.cash -= line_cost;
                competitor.asset_base += line_cost;
                competitor.distribution_capacity += 115.0 + self.rng.range(0.0, 45.0);
                competitor.generation_capacity_mwh += 42.0 + self.rng.range(0.0, 30.0);
                events.push(format!(
                    "{} expanded generation and distribution capacity.",
                    competitor.name
                ));
            }

            if self.rng.chance(0.28) && competitor.cash > 2_500.0 {
                let spend = 1_600.0 + self.rng.range(0.0, 2_000.0);
                competitor.cash -= spend;
                competitor.marketing_momentum += spend / 18_000.0;
                competitor.reputation = (competitor.reputation + spend / 7_000.0).clamp(0.0, 100.0);
            }

            if competitor.reliability < 0.68 && competitor.cash > 3_000.0 {
                competitor.cash -= 3_000.0;
                competitor.reliability = (competitor.reliability + 0.08).clamp(0.35, 0.96);
            }
        }
    }

    fn customer_churn(&mut self, events: &mut Vec<String>) -> f64 {
        let average_rate = self.average_rate();
        let player_loss = churn_for(&self.player, average_rate);
        self.player.customers -= player_loss;

        let mut churn_pool = player_loss;
        for competitor in &mut self.competitors {
            let loss = churn_for(competitor, average_rate);
            competitor.customers -= loss;
            churn_pool += loss;
        }

        if churn_pool > 0.0 {
            self.allocate_customers(churn_pool, events, "switched accounts");
        }

        player_loss
    }

    fn allocate_new_customers(&mut self, events: &mut Vec<String>) {
        let serviceable = self.serviceable_customers();
        let connected = self.total_connected_customers();
        let natural_demand = (serviceable - connected).max(0.0);
        let prospects =
            (natural_demand * 0.36 + self.market.addressable_customers * 0.006).max(18.0);
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
        let market_cap = self.player.market_cap().max(1.0);
        let annualized_earnings_yield = (finances.profit * 4.0 / market_cap).clamp(-0.10, 0.12);
        let growth_signal = (self.player.customers / 160.0 - 1.0).clamp(-0.10, 0.18) * 0.10;
        let leverage_drag = (self.player.debt_to_assets() - 0.65).max(0.0) * 0.08;
        let reliability_signal = (self.player.reliability - 0.80) * 0.05;
        let noise = self.rng.range(-0.018, 0.018);
        let change =
            annualized_earnings_yield + growth_signal + reliability_signal + noise - leverage_drag;
        self.player.stock_price = (self.player.stock_price * (1.0 + change)).clamp(6.0, 85.0);
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

impl Market {
    pub fn serviceable_customers(&self) -> f64 {
        self.addressable_customers * self.electrification
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

fn settle_utility(
    utility: &mut Utility,
    market: &Market,
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
    let operating_cost = served_mwh * market.variable_cost_per_mwh * maintenance_load
        + 850.0
        + utility.customers * 2.65
        + utility.asset_base * 0.0105;
    let interest = utility.debt * 0.0175;
    let profit = revenue - operating_cost - interest;

    utility.cash += profit;
    if utility.cash < 0.0 {
        utility.debt += -utility.cash;
        utility.cash = 0.0;
        utility.reputation = (utility.reputation - 1.8).clamp(0.0, 100.0);
    }

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
        utility.reliability = (utility.reliability + 0.008).clamp(0.35, 0.98);
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
    let rate_pressure = (utility.rate_cents - average_rate).max(0.0) * 0.006;
    let outage_pressure = (0.78 - utility.reliability).max(0.0) * 0.055;
    let reputation_pressure = (48.0 - utility.reputation).max(0.0) * 0.0008;
    let churn_rate = 0.010 + rate_pressure + outage_pressure + reputation_pressure;
    utility.customers * churn_rate.clamp(0.006, 0.075)
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
    fn stock_issuance_has_no_fixed_proceeds_cap() {
        let mut game = Game::with_seed(14);
        let starting_cash = game.player.cash;

        game.apply_decision(Decision::IssueStock { amount: 120_000.0 })
            .unwrap();

        assert!(game.player.cash > starting_cash + 100_000.0);
        assert!(game.player.shares > 2_000.0);
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
        assert!(game.player.stock_price >= 6.0);
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
