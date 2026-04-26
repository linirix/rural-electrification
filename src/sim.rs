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
const ACQUISITION_CASH_ABSORPTION: f64 = 0.80;
const ACQUISITION_DEBT_ASSUMPTION: f64 = 0.75;
const NEW_GENERATION_EQUIPMENT_RELIABILITY: f64 = 0.965;
const NEW_GENERATION_RELIABILITY_WEIGHT: f64 = 0.35;
const MAX_GENERATION_RELIABILITY_LIFT: f64 = 0.085;
const MIN_STOCK_PRICE: f64 = 0.25;
const FRESH_EQUITY_CASH_MARKET_RECOGNITION: f64 = 0.35;
const BUYBACK_CASH_MARKET_RECOGNITION: f64 = 0.45;
const VARIABLE_COST_REVERSION: f64 = 0.14;
const VARIABLE_COST_PRESSURE_SENSITIVITY: f64 = 2.35;
const VARIABLE_COST_FLOOR_MULTIPLE: f64 = 0.72;
const VARIABLE_COST_CEILING_MULTIPLE: f64 = 1.65;
const MAX_CREDIBLE_RATE_CENTS: f64 = 25.0;
pub const MAX_PUBLIC_RATE_PREMIUM_CENTS: f64 = 5.0;
const RECEIVERSHIP_DEBT_TO_ASSETS: f64 = 1.05;
const MAX_RELIABILITY: f64 = 0.98;
const MIN_MAINTENANCE_RELIABILITY_GAIN: f64 = 0.0005;
const DILIGENCE_DURATION_QUARTERS: u32 = 3;
const BOARD_CHECKPOINT_QUARTER: u32 = 8;
const BOARD_CHECKPOINT_SHARE_TARGET: f64 = 0.32;
const BOARD_CHECKPOINT_REPUTATION_PENALTY: f64 = 3.0;
const BOARD_CHECKPOINT_EQUITY_FATIGUE: f64 = 0.30;

mod attribution;
mod competitors;
mod economics;

use attribution::{AttributionContext, quarter_attributions};
use competitors::draw_competitor_name;
#[cfg(test)]
use competitors::{COMPETITOR_NAME_POOL, StartupEntryReason};
use economics::{
    churn_for_market_with_floor, high_rate_excess, maintenance_reliability_gain,
    maintenance_reputation_gain, positive_amount, settle_utility, weighted_average, weighted_rate,
};
#[cfg(test)]
use economics::{churn_rate_for, churn_rate_for_market};
pub use economics::{
    distribution_project_cost, distribution_project_duration, generation_project_cost,
    generation_project_duration, money, public_rate_tolerance,
};

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
    pub integration_strain: f64,
    pub active_shocks: Vec<ActiveShock>,
    pub startup_index: u32,
    pub review_completed: bool,
    pub equity_market_fatigue: f64,
    pub diligence_reports: Vec<DiligenceReport>,
    pub last_report: Option<QuarterReport>,
    pub outcome: Option<Outcome>,
    rng: Rng,
}

#[derive(Clone, Copy, Debug)]
pub struct InitialVariance {
    pub amplitude: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CustomerAllocationKind {
    NewConnections,
    SwitchedAccounts,
}

impl CustomerAllocationKind {
    fn label(self) -> &'static str {
        match self {
            CustomerAllocationKind::NewConnections => "new connections",
            CustomerAllocationKind::SwitchedAccounts => "switched accounts",
        }
    }
}

#[derive(Clone, Debug)]
pub struct ActiveShock {
    pub kind: ShockKind,
    pub quarters_remaining: u32,
}

#[derive(Clone, Debug)]
pub struct DiligenceReport {
    pub competitor_name: String,
    pub quarters_remaining: u32,
    pub quoted_terms: AcquisitionTerms,
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
    pub baseline_variable_cost_per_mwh: f64,
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
    Diligence { competitor_index: usize },
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
    pub attributions: Vec<String>,
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

#[derive(Clone, Debug)]
pub struct AcquisitionTerms {
    pub price: f64,
    pub absorbed_cash: f64,
    pub assumed_debt: f64,
    pub net_cash_cost: f64,
    pub acquired_customers: f64,
    pub acquired_generation_capacity_mwh: f64,
    pub acquired_distribution_capacity: f64,
    pub acquired_asset_base: f64,
    pub public_interest_concession: f64,
    pub post_market_share: f64,
    pub post_cash: f64,
    pub post_debt: f64,
    pub post_asset_base: f64,
    pub post_debt_to_assets: f64,
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
    /// True for formal review outcomes where the company can continue into sandbox play.
    /// False for terminal operating failures such as receivership or lost market access.
    pub can_continue: bool,
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
        let mut name_rng = Rng::new(seed ^ 0x9E37_79B9_7F4A_7C15);
        let mut used_names = vec!["Metro Consolidated".to_string()];
        let first_competitor_name = draw_competitor_name(&mut name_rng, &used_names, 1);
        used_names.push(first_competitor_name.clone());
        let second_competitor_name = draw_competitor_name(&mut name_rng, &used_names, 2);
        used_names.push(second_competitor_name.clone());
        let third_competitor_name = draw_competitor_name(&mut name_rng, &used_names, 3);

        let addressable_customers = varied_pct(&mut rng, 6_500.0, 0.08, initial_variance).round();
        let electrification = varied_abs(&mut rng, 0.14, 0.018, initial_variance).clamp(0.10, 0.18);
        let avg_mwh_per_customer =
            varied_pct(&mut rng, 0.22, 0.08, initial_variance).clamp(0.18, 0.27);
        let variable_cost_per_mwh =
            varied_pct(&mut rng, 26.0, 0.07, initial_variance).clamp(21.0, 31.0);
        let standard_rate_cents =
            varied_abs(&mut rng, 10.4, 0.35, initial_variance).clamp(9.5, 11.3);
        let civic_patience = varied_abs(&mut rng, 0.76, 0.04, initial_variance).clamp(0.64, 0.88);
        let market = Market {
            territory: "Core Market".to_string(),
            start_year: 1,
            addressable_customers,
            electrification,
            avg_mwh_per_customer,
            variable_cost_per_mwh,
            baseline_variable_cost_per_mwh: variable_cost_per_mwh,
            standard_rate_cents,
            civic_patience,
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
                    &first_competitor_name,
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
                    &second_competitor_name,
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
                    &third_competitor_name,
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
            integration_strain: 0.0,
            active_shocks: Vec::new(),
            startup_index: 0,
            review_completed: false,
            equity_market_fatigue: 0.0,
            diligence_reports: Vec::new(),
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
            return Err(
                "The market review is paused on a final result. Use 'continue' to keep playing if the review allows it."
                    .to_string(),
            );
        }

        match decision {
            Decision::BuildGeneration { capacity_mwh } => {
                let capacity_mwh = positive_amount(capacity_mwh, "generation project size")?;
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
                    "Started a generation expansion for {}. It will add {:.0} MWh/quarter in {} quarter(s) and refresh fleet reliability when online.",
                    money(cost),
                    capacity_mwh,
                    quarters
                ))
            }
            Decision::BuildDistribution { customer_capacity } => {
                let customer_capacity =
                    positive_amount(customer_capacity, "distribution project size")?;
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
                let spend = positive_amount(spend, "marketing spend")?;
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
                let financing_base = self.equity_issuance_base();
                let pressure = amount / financing_base;
                let effective_pressure = pressure + self.equity_market_fatigue;
                let effective_pressure_squared = effective_pressure * effective_pressure;
                let finance_pressure = self.macro_state.financing_pressure();
                let issue_discount = (0.03
                    + effective_pressure * 0.05
                    + effective_pressure_squared * 0.08
                    + finance_pressure * 0.055)
                    .clamp(0.03, 0.65);
                let raw_issue_price = self.player.stock_price * (1.0 - issue_discount);
                if raw_issue_price <= MIN_STOCK_PRICE * 1.05 && amount > financing_base * 0.15 {
                    return Err(
                        "The market cannot absorb that stock sale at a viable price; try a smaller issue or rebuild valuation."
                            .to_string(),
                    );
                }
                let issue_price = raw_issue_price.max(MIN_STOCK_PRICE);
                let new_shares = amount / issue_price;
                let fee_rate = 0.025
                    + effective_pressure * 0.012
                    + effective_pressure_squared * 0.020
                    + finance_pressure * 0.012;
                if fee_rate >= 0.75 {
                    return Err(format!(
                        "The market cannot absorb that stock sale. Expected fees would consume {:.0}% of gross proceeds; try a smaller issue.",
                        fee_rate * 100.0
                    ));
                }
                let underwriting_cost = amount * fee_rate;
                let net_proceeds = amount - underwriting_cost;
                let post_money_price = (old_market_cap
                    + net_proceeds * FRESH_EQUITY_CASH_MARKET_RECOGNITION)
                    / (old_shares + new_shares);
                let signal_drag = (effective_pressure * 0.020 + effective_pressure_squared * 0.040)
                    .clamp(0.0, 0.30);
                self.player.cash += net_proceeds;
                self.player.shares += new_shares;
                self.player.stock_price =
                    (post_money_price * (1.0 - signal_drag)).max(MIN_STOCK_PRICE);
                self.player.reputation = (self.player.reputation
                    - effective_pressure * 1.5
                    - effective_pressure_squared * 1.5)
                    .clamp(0.0, 100.0);
                self.equity_market_fatigue =
                    (self.equity_market_fatigue + pressure * 0.80).clamp(0.0, 6.0);
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
                let old_shares = self.player.shares.max(1.0);
                let pressure = amount / old_market_cap;
                let repurchase_premium =
                    (0.015 + pressure.min(1.6) * 0.085 + pressure.powf(1.20) * 0.012)
                        .clamp(0.015, 0.30);
                let repurchase_price =
                    (self.player.stock_price * (1.0 + repurchase_premium)).max(MIN_STOCK_PRICE);
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
                let remaining_cash = self.player.cash - actual_spend;
                let float_retired = shares_bought / old_shares;
                let actual_pressure = actual_spend / old_market_cap;
                let market_cap_after_cash_use = (old_market_cap
                    - actual_spend * BUYBACK_CASH_MARKET_RECOGNITION)
                    .max(remaining_shares * MIN_STOCK_PRICE);
                let capital_return_signal =
                    (float_retired * 0.30 + actual_pressure.min(1.0) * 0.04).clamp(0.0, 0.16);
                let liquidity_drag =
                    ((8_000.0 - remaining_cash).max(0.0) / 8_000.0 * 0.09).clamp(0.0, 0.09);
                let overextension_drag =
                    ((actual_spend / self.player.cash.max(1.0)) - 0.60).max(0.0) * 0.12;
                let stress_drag = (liquidity_drag + overextension_drag).clamp(0.0, 0.18);
                let post_buyback_market_cap =
                    market_cap_after_cash_use * (1.0 + capital_return_signal) * (1.0 - stress_drag);
                self.player.cash -= actual_spend;
                self.player.shares = remaining_shares;
                self.player.stock_price =
                    (post_buyback_market_cap / remaining_shares.max(1.0)).max(MIN_STOCK_PRICE);
                Ok(format!(
                    "Bought back {:.0} shares at ${:.2}, spending {} and retiring {:.1}% of the float. Shares outstanding now {:.0}; stock is ${:.2}.",
                    shares_bought,
                    repurchase_price,
                    money(actual_spend),
                    float_retired * 100.0,
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
            Decision::Diligence { competitor_index } => {
                if competitor_index >= self.competitors.len() {
                    return Err("No competitor has that number.".to_string());
                }
                let cost = self
                    .diligence_cost(competitor_index)
                    .expect("competitor index checked before diligence");
                self.require_cash(cost)?;
                self.player.cash -= cost;
                let name = self.competitors[competitor_index].name.clone();
                if let Some(report) = self
                    .diligence_reports
                    .iter_mut()
                    .find(|report| report.competitor_name == name && report.quarters_remaining > 0)
                {
                    report.quarters_remaining = DILIGENCE_DURATION_QUARTERS;
                } else {
                    let quoted_terms = self
                        .current_acquisition_terms(competitor_index)
                        .expect("competitor index checked before diligence terms quote");
                    self.diligence_reports
                        .retain(|report| report.competitor_name != name);
                    self.diligence_reports.push(DiligenceReport {
                        competitor_name: name.clone(),
                        quarters_remaining: DILIGENCE_DURATION_QUARTERS,
                        quoted_terms,
                    });
                }
                Ok(format!(
                    "Completed diligence on {name} for {}. Exact acquisition terms are available for {} quarter(s).",
                    money(cost),
                    DILIGENCE_DURATION_QUARTERS
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

                if !self.has_diligence(competitor_index) {
                    let cost = self
                        .diligence_cost(competitor_index)
                        .expect("competitor index checked before diligence prompt");
                    return Err(format!(
                        "Run 'diligence {}' first. It costs {} and reveals exact closing terms before you commit.",
                        competitor_index + 1,
                        money(cost)
                    ));
                }

                let terms = self
                    .acquisition_terms(competitor_index)
                    .expect("competitor index checked before acquisition");
                self.require_cash(terms.price)?;
                let acquired = self.competitors.remove(competitor_index);
                let acquired_name = acquired.name.clone();
                let starting_customers = self.player.customers;
                let starting_generation_capacity = self.player.generation_capacity_mwh;
                let combined_reputation = weighted_average(
                    self.player.reputation,
                    starting_customers,
                    acquired.reputation,
                    terms.acquired_customers * 0.65,
                )
                .clamp(0.0, 100.0);
                let combined_reliability = weighted_average(
                    self.player.reliability,
                    starting_generation_capacity,
                    acquired.reliability,
                    terms.acquired_generation_capacity_mwh * 0.70,
                )
                .clamp(0.35, 0.98);
                self.player.cash -= terms.price;
                self.player.cash += terms.absorbed_cash;
                self.player.debt += terms.assumed_debt;
                // Integration losses keep roll-ups from being pure scale arbitrage.
                self.player.customers += terms.acquired_customers;
                self.player.generation_capacity_mwh += terms.acquired_generation_capacity_mwh;
                self.player.distribution_capacity += terms.acquired_distribution_capacity;
                self.player.asset_base += terms.acquired_asset_base;
                self.player.reputation = combined_reputation;
                self.player.reliability = combined_reliability;
                self.player.reliability = (self.player.reliability - 0.035).clamp(0.35, 0.98);
                self.player.reputation = (self.player.reputation - 1.5).clamp(0.0, 100.0);
                self.acquisition_cooldown =
                    acquisition_integration_cooldown(starting_customers, terms.acquired_customers);
                self.integration_strain = (self.integration_strain
                    + acquisition_integration_strain(starting_customers, terms.acquired_customers))
                .clamp(0.0, 1.6);
                self.diligence_reports
                    .retain(|report| report.competitor_name != acquired_name);
                Ok(format!(
                    "Acquired {acquired_name} for {} (net of {} absorbed cash, plus {} assumed debt). Integration will take {} quarter(s).",
                    money(terms.price),
                    money(terms.absorbed_cash),
                    money(terms.assumed_debt),
                    self.acquisition_cooldown
                ))
            }
            Decision::AdjustRate { delta_cents } => {
                if !delta_cents.is_finite() {
                    return Err("The rate change must be a finite number.".to_string());
                }
                if delta_cents > 0.0 && self.rate_frozen() {
                    return Err(
                        "The market-wide rate freeze blocks any rate increase right now."
                            .to_string(),
                    );
                }
                let old = self.player.rate_cents;
                let requested_rate = self.player.rate_cents + delta_cents;
                if requested_rate <= 0.0 {
                    return Err(
                        "Rates must stay above 0.0c/kWh; use a positive tariff.".to_string()
                    );
                }
                let public_ceiling =
                    public_rate_tolerance(&self.market) + MAX_PUBLIC_RATE_PREMIUM_CENTS;
                if delta_cents > 0.0 && requested_rate > public_ceiling {
                    return Err(format!(
                        "A {:.1}c rate is beyond public tolerance. Keep rates at or below {:.1}c/kWh or raise gradually.",
                        requested_rate, public_ceiling
                    ));
                }
                if requested_rate > MAX_CREDIBLE_RATE_CENTS {
                    return Err(format!(
                        "A {:.1}c rate is not a credible tariff. Keep rates at or below {:.1}c and let market consequences do the rest.",
                        requested_rate, MAX_CREDIBLE_RATE_CENTS
                    ));
                }
                self.player.rate_cents = requested_rate;
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
                let spend = positive_amount(spend, "maintenance spend")?;
                let spend = spend.clamp(1_500.0, 20_000.0);
                let gain = maintenance_reliability_gain(&self.player, spend);
                let new_reliability = (self.player.reliability + gain).clamp(0.35, MAX_RELIABILITY);
                let actual_gain = new_reliability - self.player.reliability;
                if actual_gain < MIN_MAINTENANCE_RELIABILITY_GAIN {
                    return Err("Reliability is already at the 98% operating cap; maintenance would not improve service enough to justify spending.".to_string());
                }
                self.require_cash(spend)?;
                self.player.cash -= spend;
                self.player.reliability = new_reliability;
                self.player.reputation = (self.player.reputation
                    + maintenance_reputation_gain(&self.player, spend))
                .clamp(0.0, 100.0);
                Ok(format!(
                    "Spent {} on reliability work; gained {:.1} reliability points.",
                    money(spend),
                    actual_gain * 100.0
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
                attributions: vec![
                    "The market review is paused on a final result; no new operating quarter was processed."
                        .to_string(),
                ],
                events: vec![format!("{} {}", outcome.headline, outcome.details)],
            };
            self.last_report = Some(report.clone());
            return report;
        }

        let label = self.date_label();
        let starting_customers = self.player.customers;
        let starting_market_share = self.market_share();
        let starting_total_connected = self.total_connected_customers();
        let starting_reliability = self.player.reliability;
        let starting_headroom = self.player.capacity_headroom(&self.market);
        let mut events = Vec::new();

        self.advance_shocks(&mut events);
        self.advance_macro_environment(&mut events);
        self.maybe_spawn_startup(&mut events);
        self.maybe_merge_rivals(&mut events);
        self.maybe_rival_counteroffensive(&mut events);
        self.complete_projects(&mut events);
        self.grow_market(&mut events);
        self.competitor_plans(&mut events);
        self.apply_integration_strain(&mut events);

        self.player.marketing_momentum *= 0.55;
        self.equity_market_fatigue *= 0.55;
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
        self.advance_diligence_reports();
        self.apply_board_checkpoint(&mut events);
        self.check_outcome(&player_finances);

        let ending_customers = self.player.customers;
        let ending_total_connected = self.total_connected_customers();
        let ending_market_share = self.market_share();
        let ending_reliability = self.player.reliability;
        let ending_headroom = self.player.capacity_headroom(&self.market);
        let lost_customer_rate = if starting_customers > 0.0 {
            lost_customers / starting_customers
        } else {
            0.0
        };
        let attributions = quarter_attributions(AttributionContext {
            starting_customers,
            ending_customers,
            starting_total_connected,
            ending_total_connected,
            starting_market_share,
            ending_market_share,
            starting_reliability,
            ending_reliability,
            starting_headroom,
            ending_headroom,
            lost_customer_rate,
            rate_gap_to_rivals: self.player.rate_cents - self.rival_average_rate_for_player(),
            finances: &player_finances,
            active_shocks: &self.active_shocks,
        });

        let report = QuarterReport {
            label,
            revenue: player_finances.revenue,
            operating_cost: player_finances.operating_cost,
            interest: player_finances.interest,
            profit: player_finances.profit,
            new_customers: (ending_customers - starting_customers + lost_customers).max(0.0),
            lost_customers,
            lost_customer_rate,
            market_share: ending_market_share,
            prior_market_share: starting_market_share,
            attributions,
            events,
        };
        self.last_report = Some(report.clone());
        report
    }

    pub fn continue_after_review(&mut self) -> Result<String, String> {
        let Some(outcome) = &self.outcome else {
            return Err("There is no completed market review to continue from.".to_string());
        };

        if !outcome.can_continue {
            return Err(
                "This outcome is terminal; the company cannot continue operating.".to_string(),
            );
        }

        let headline = outcome.headline.clone();
        self.outcome = None;
        Ok(format!(
            "Continuing after {headline}. The formal review is complete; operations now continue without a fixed end date."
        ))
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

    fn equity_issuance_base(&self) -> f64 {
        let market_cap = self.player.market_cap().max(1.0);
        let cash_adjusted_cap = market_cap - self.player.cash.max(0.0) * 0.60;
        let asset_support = self.player.asset_base.max(1.0) * 0.65;
        cash_adjusted_cap
            .max(asset_support)
            .max(self.player.shares * MIN_STOCK_PRICE)
            .max(1.0)
    }

    pub fn player_annual_interest_rate(&self) -> f64 {
        self.macro_state.annual_interest_rate_for(&self.player)
    }

    pub fn acquisition_price(&self, competitor_index: usize) -> f64 {
        self.acquisition_terms(competitor_index)
            .expect("competitor index out of range")
            .price
    }

    pub fn has_diligence(&self, competitor_index: usize) -> bool {
        let Some(competitor) = self.competitors.get(competitor_index) else {
            return false;
        };

        self.diligence_reports.iter().any(|report| {
            report.quarters_remaining > 0 && report.competitor_name == competitor.name
        })
    }

    pub fn diligence_cost(&self, competitor_index: usize) -> Option<f64> {
        let competitor = self.competitors.get(competitor_index)?;
        let scale_cost = competitor.customers * 8.0 + competitor.asset_base * 0.018;
        let complexity_cost = competitor.debt_to_assets().max(0.0) * 1_800.0;
        let concentration_cost = (self.market_share() - 0.45).max(0.0) * 8_000.0;
        Some((2_800.0 + scale_cost + complexity_cost + concentration_cost).clamp(3_000.0, 14_000.0))
    }

    pub fn acquisition_terms(&self, competitor_index: usize) -> Option<AcquisitionTerms> {
        if let Some(report) = self.active_diligence_report(competitor_index) {
            return Some(self.terms_with_current_financing_context(report.quoted_terms.clone()));
        }

        self.current_acquisition_terms(competitor_index)
    }

    fn active_diligence_report(&self, competitor_index: usize) -> Option<&DiligenceReport> {
        let competitor = self.competitors.get(competitor_index)?;
        self.diligence_reports.iter().find(|report| {
            report.quarters_remaining > 0 && report.competitor_name == competitor.name
        })
    }

    fn current_acquisition_terms(&self, competitor_index: usize) -> Option<AcquisitionTerms> {
        let competitor = self.competitors.get(competitor_index)?;
        let acquired_customers = competitor.customers * 0.92;
        let acquired_generation_capacity_mwh = competitor.generation_capacity_mwh * 0.86;
        let acquired_distribution_capacity = competitor.distribution_capacity * 0.90;
        let acquired_asset_base = competitor.asset_base * 0.72;
        let customer_value = competitor.customers * 70.0;
        let generation_value = competitor.generation_capacity_mwh * 36.0;
        let line_value = competitor.distribution_capacity * 11.0;
        let market_position_value = competitor.reputation * 90.0;
        let premium = 8_000.0 + competitor.reliability * 4_000.0;
        let share = self.market_share();
        let consolidation_premium = acquisition_consolidation_premium(share);
        let enterprise_value =
            (customer_value + generation_value + line_value + market_position_value + premium)
                * consolidation_premium;
        let total_connected = self.total_connected_customers().max(1.0);
        let post_player_customers = self.player.customers + acquired_customers;
        let post_total_connected = (total_connected - competitor.customers + acquired_customers)
            .max(post_player_customers)
            .max(1.0);
        let post_market_share = post_player_customers / post_total_connected;
        let public_interest_concession = if post_market_share > 0.50 {
            let excess_share = post_market_share - 0.50;
            enterprise_value * excess_share.powf(1.10) * 1.35 + acquired_customers * 18.0
        } else {
            0.0
        };
        let net_balance_sheet_adjustment = competitor.cash * ACQUISITION_CASH_ABSORPTION
            - competitor.debt * ACQUISITION_DEBT_ASSUMPTION;
        let price = (enterprise_value + public_interest_concession + net_balance_sheet_adjustment)
            .max(1_000.0);
        let absorbed_cash = competitor.cash * ACQUISITION_CASH_ABSORPTION;
        let assumed_debt = competitor.debt * ACQUISITION_DEBT_ASSUMPTION;
        let net_cash_cost = price - absorbed_cash;
        let post_cash = self.player.cash - price + absorbed_cash;
        let post_debt = self.player.debt + assumed_debt;
        let post_asset_base = self.player.asset_base + acquired_asset_base;
        let post_debt_to_assets = post_debt / post_asset_base.max(1.0);

        Some(AcquisitionTerms {
            price,
            absorbed_cash,
            assumed_debt,
            net_cash_cost,
            acquired_customers,
            acquired_generation_capacity_mwh,
            acquired_distribution_capacity,
            acquired_asset_base,
            public_interest_concession,
            post_market_share,
            post_cash,
            post_debt,
            post_asset_base,
            post_debt_to_assets,
        })
    }

    fn terms_with_current_financing_context(
        &self,
        mut terms: AcquisitionTerms,
    ) -> AcquisitionTerms {
        terms.post_cash = self.player.cash - terms.price + terms.absorbed_cash;
        terms.post_debt = self.player.debt + terms.assumed_debt;
        terms.post_asset_base = self.player.asset_base + terms.acquired_asset_base;
        terms.post_debt_to_assets = terms.post_debt / terms.post_asset_base.max(1.0);
        terms
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

    pub fn rate_freeze_probability(&self) -> f64 {
        let player_excess = high_rate_excess(&self.player, &self.market);
        let market_excess = (self.average_rate() - public_rate_tolerance(&self.market)).max(0.0);
        (0.035
            + (1.0 - self.market.civic_patience) * 0.025
            + player_excess * 0.035
            + market_excess * 0.025)
            .clamp(0.02, 0.38)
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

        let rate_freeze_probability = self.rate_freeze_probability();
        if self.rng.chance(rate_freeze_probability)
            && !self.shock_active(|kind| matches!(kind, ShockKind::RateFreeze))
        {
            self.active_shocks.push(ActiveShock {
                kind: ShockKind::RateFreeze,
                quarters_remaining: 2 + (self.rng.next_u64() % 2) as u32,
            });
            if high_rate_excess(&self.player, &self.market) > 0.5 {
                events.push(
                    "A market-wide rate freeze took effect after pressure over high rates."
                        .to_string(),
                );
            } else {
                events.push("A market-wide rate freeze took effect.".to_string());
            }
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
            let target = self.rng.next_index(self.competitors.len() + 1);
            let capacity_loss = 0.08 + self.rng.range(0.0, 0.07);
            if target == 0 {
                let lost = apply_equipment_fire_damage(&mut self.player, capacity_loss);
                events.push(format!(
                    "An equipment fire at Metro knocked {:.0} MWh/quarter of generation offline.",
                    lost
                ));
            } else if let Some(competitor) = self.competitors.get_mut(target - 1) {
                let name = competitor.name.clone();
                let lost = apply_equipment_fire_damage(competitor, capacity_loss);
                events.push(format!(
                    "An equipment fire at {name} knocked {:.0} MWh/quarter of rival generation offline.",
                    lost
                ));
            }
        }
    }

    fn advance_diligence_reports(&mut self) {
        for report in &mut self.diligence_reports {
            report.quarters_remaining = report.quarters_remaining.saturating_sub(1);
        }
        self.prune_diligence_reports();
    }

    fn prune_diligence_reports(&mut self) {
        self.diligence_reports.retain(|report| {
            report.quarters_remaining > 0
                && self
                    .competitors
                    .iter()
                    .any(|competitor| competitor.name == report.competitor_name)
        });
    }

    fn apply_board_checkpoint(&mut self, events: &mut Vec<String>) {
        if self.quarter != BOARD_CHECKPOINT_QUARTER
            || self.review_completed
            || self.market_share() >= BOARD_CHECKPOINT_SHARE_TARGET
        {
            return;
        }

        self.player.reputation =
            (self.player.reputation - BOARD_CHECKPOINT_REPUTATION_PENALTY).clamp(0.0, 100.0);
        self.equity_market_fatigue =
            (self.equity_market_fatigue + BOARD_CHECKPOINT_EQUITY_FATIGUE).clamp(0.0, 6.0);
        events.push(format!(
            "Board confidence weakened at the Year 2 checkpoint: market share below {:.0}% hurt reputation and made fresh equity harder to sell.",
            BOARD_CHECKPOINT_SHARE_TARGET * 100.0
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
                        let old_reliability = self.player.reliability;
                        let old_generation_capacity = self.player.generation_capacity_mwh;
                        let new_reliability = generation_reliability_after_new_capacity(
                            old_reliability,
                            old_generation_capacity,
                            capacity_mwh,
                        );
                        self.player.generation_capacity_mwh += capacity_mwh;
                        self.player.reliability = new_reliability;
                        let reliability_lift = (new_reliability - old_reliability).max(0.0);
                        events.push(format!(
                            "Your {} came online, adding {:.0} MWh/quarter of generating capacity and lifting reliability by {:.1} points.",
                            project.name,
                            capacity_mwh,
                            reliability_lift * 100.0
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

    fn apply_integration_strain(&mut self, events: &mut Vec<String>) {
        if self.integration_strain <= 0.01 {
            self.integration_strain = 0.0;
            return;
        }

        let reliability_drag = (0.006 + self.integration_strain * 0.018).min(0.035);
        let reputation_drag = (0.8 + self.integration_strain * 2.8).min(5.0);
        let marketing_drag = (1.0 - self.integration_strain * 0.08).clamp(0.75, 1.0);
        self.player.reliability = (self.player.reliability - reliability_drag).clamp(0.35, 0.98);
        self.player.reputation = (self.player.reputation - reputation_drag).clamp(0.0, 100.0);
        self.player.marketing_momentum *= marketing_drag;

        if self.integration_strain >= 0.25 {
            events.push(format!(
                "Acquisition integration strained service quality and reputation by {:.1} points.",
                reputation_drag
            ));
        }

        self.integration_strain *= 0.62;
        if self.integration_strain < 0.05 {
            self.integration_strain = 0.0;
        }
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
        let baseline_cost = self.market.baseline_variable_cost_per_mwh.max(1.0);
        let pressure_target = baseline_cost
            * (1.0 + self.macro_state.cost_pressure * VARIABLE_COST_PRESSURE_SENSITIVITY);
        let bounded_target = pressure_target.clamp(
            baseline_cost * VARIABLE_COST_FLOOR_MULTIPLE,
            baseline_cost * VARIABLE_COST_CEILING_MULTIPLE,
        );
        let random_cost_noise = self.rng.range(-0.006, 0.007);
        self.market.variable_cost_per_mwh = (self.market.variable_cost_per_mwh
            + (bounded_target - self.market.variable_cost_per_mwh) * VARIABLE_COST_REVERSION)
            * (1.0 + random_cost_noise);
        self.market.variable_cost_per_mwh = self.market.variable_cost_per_mwh.clamp(
            baseline_cost * VARIABLE_COST_FLOOR_MULTIPLE,
            baseline_cost * VARIABLE_COST_CEILING_MULTIPLE,
        );

        if self.rng.chance(0.18) {
            events.push(
                "Demand rose as more homes and businesses shifted to electric service.".to_string(),
            );
        }
    }

    fn customer_churn(&mut self, events: &mut Vec<String>) -> f64 {
        let player_alternative_rate = self.rival_average_rate_for_player();
        let competitor_alternative_rates = (0..self.competitors.len())
            .map(|index| self.alternative_rate_for_competitor(index))
            .collect::<Vec<_>>();
        let natural_churn_floor = self.natural_churn_floor();

        let player_loss = churn_for_market_with_floor(
            &self.player,
            player_alternative_rate,
            &self.market,
            natural_churn_floor,
        );
        self.player.customers -= player_loss;
        let mut player_switch_gain = self.allocate_customers_from(
            player_loss,
            CustomerAllocationKind::SwitchedAccounts,
            Some(0),
        );

        let competitor_losses = self
            .competitors
            .iter()
            .zip(competitor_alternative_rates)
            .map(|(competitor, alternative_rate)| {
                churn_for_market_with_floor(
                    competitor,
                    alternative_rate,
                    &self.market,
                    natural_churn_floor,
                )
            })
            .collect::<Vec<_>>();

        for (index, loss) in competitor_losses.into_iter().enumerate() {
            self.competitors[index].customers -= loss;
            player_switch_gain += self.allocate_customers_from(
                loss,
                CustomerAllocationKind::SwitchedAccounts,
                Some(index + 1),
            );
        }

        if player_switch_gain >= 8.0 {
            events.push(format!(
                "Your sales and service teams captured {:.0} {}.",
                player_switch_gain,
                CustomerAllocationKind::SwitchedAccounts.label()
            ));
        }

        player_loss
    }

    fn natural_churn_floor(&mut self) -> f64 {
        let patience_pressure = (1.0 - self.market.civic_patience) * 0.0012;
        let demand_mobility = self.macro_state.demand_index.max(0.0) * 0.0010;
        (0.0035 + patience_pressure + demand_mobility + self.rng.range(-0.0010, 0.0012))
            .clamp(0.0024, 0.0065)
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
        let player_gain =
            self.allocate_customers_from(prospects, CustomerAllocationKind::NewConnections, None);
        if player_gain >= 8.0 {
            events.push(format!(
                "Your sales and service teams captured {:.0} {}.",
                player_gain,
                CustomerAllocationKind::NewConnections.label()
            ));
        }
    }

    fn allocate_customers_from(
        &mut self,
        prospects: f64,
        allocation_kind: CustomerAllocationKind,
        excluded_utility_index: Option<usize>,
    ) -> f64 {
        if prospects <= 0.0 {
            return 0.0;
        }

        let mut scores = self.competition_scores(allocation_kind);
        if let Some(index) = excluded_utility_index
            && index < scores.len()
        {
            scores[index] = 0.0;
        }
        let score_total: f64 = scores.iter().sum();
        if score_total <= 0.0 {
            return 0.0;
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

        player_gain
    }

    fn competition_scores(&self, allocation_kind: CustomerAllocationKind) -> Vec<f64> {
        let average_rate = self.average_rate();
        let mut scores = Vec::with_capacity(self.competitors.len() + 1);
        scores.push(
            self.player
                .competition_score(&self.market, average_rate, allocation_kind),
        );
        for competitor in &self.competitors {
            scores.push(competitor.competition_score(&self.market, average_rate, allocation_kind));
        }
        scores
    }

    #[cfg(test)]
    fn allocation_share(&self, allocation_kind: CustomerAllocationKind) -> f64 {
        let scores = self.competition_scores(allocation_kind);
        let score_total: f64 = scores.iter().sum();
        if score_total <= 0.0 {
            0.0
        } else {
            scores[0] / score_total
        }
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
        let book_equity = (self.player.asset_base + self.player.cash - self.player.debt)
            .max(shares * MIN_STOCK_PRICE);
        let book_value_per_share = book_equity / shares;

        let annualized_profit = finances.profit * 4.0;
        let multiple = self.earnings_multiple();
        let earnings_value_per_share = annualized_profit.max(0.0) * multiple / shares;

        let fundamental_price =
            (book_value_per_share * 0.5 + earnings_value_per_share * 0.5).max(MIN_STOCK_PRICE);
        let gap = fundamental_price - self.player.stock_price;
        let reverted = self.player.stock_price + gap * 0.22;

        let macro_signal = self.macro_state.valuation_signal();
        let shock_signal = self.active_shock_valuation_signal();
        let noise = self.rng.range(-0.025, 0.025);
        self.player.stock_price =
            (reverted * (1.0 + macro_signal + shock_signal + noise)).max(MIN_STOCK_PRICE);
    }

    fn earnings_multiple(&self) -> f64 {
        let base = 7.0;
        let growth_bonus = (self.player.customers / 250.0).clamp(0.0, 5.0);
        let reliability_bonus = (self.player.reliability - 0.78) * 4.0;
        let reputation_bonus = ((self.player.reputation - 55.0) / 20.0).clamp(-1.25, 2.0);
        let leverage_drag = (self.player.debt_to_assets() - 0.65).max(0.0) * 5.0;
        let macro_drag = (self.macro_state.benchmark_credit_rate() - 0.07) * 30.0;
        (base + growth_bonus + reliability_bonus + reputation_bonus - leverage_drag - macro_drag)
            .clamp(2.5, 18.0)
    }

    fn active_shock_valuation_signal(&self) -> f64 {
        let mut signal: f64 = 0.0;
        for shock in &self.active_shocks {
            signal += match shock.kind {
                ShockKind::RateFreeze => -0.018,
                ShockKind::DemandRecession => -0.025,
                ShockKind::DemandBoom => 0.012,
                ShockKind::InputCostShock => -0.020,
            };
        }
        signal.clamp(-0.065, 0.025)
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

        if !self.review_completed && self.quarter >= self.campaign_quarters {
            let share = self.market_share();
            let healthy_balance_sheet = self.player.debt_to_assets() <= 0.95;
            self.review_completed = true;
            if share >= 0.45 && self.player.reliability >= 0.72 && healthy_balance_sheet {
                self.outcome = Some(Outcome {
                    kind: OutcomeKind::Victory,
                    headline: "Market Lead Secured".to_string(),
                    details: format!(
                        "You finished with {:.0}% of connected accounts, solid reliability, and a balance sheet lenders can still underwrite.",
                        share * 100.0
                    ),
                    can_continue: true,
                });
            } else {
                self.outcome = Some(Outcome {
                    kind: OutcomeKind::Defeat,
                    headline: "Board Lost Confidence".to_string(),
                    details: format!(
                        "After Year 5 you held {:.0}% of connected accounts. The board wanted a clearer path to durable market leadership.",
                        share * 100.0
                    ),
                    can_continue: true,
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

fn acquisition_integration_cooldown(starting_customers: f64, acquired_customers: f64) -> u32 {
    let post_customers = (starting_customers + acquired_customers).max(1.0);
    let acquired_share = (acquired_customers / post_customers).clamp(0.0, 1.0);
    (2 + (acquired_share * 8.0).floor() as u32).clamp(2, 6)
}

fn acquisition_integration_strain(starting_customers: f64, acquired_customers: f64) -> f64 {
    let post_customers = (starting_customers + acquired_customers).max(1.0);
    let acquired_share = (acquired_customers / post_customers).clamp(0.0, 1.0);
    acquired_share * 1.35
}

fn acquisition_consolidation_premium(share: f64) -> f64 {
    let share = share.clamp(0.0, 1.0);
    1.0 + share * 0.35 + share * share * 2.2
}

fn apply_equipment_fire_damage(utility: &mut Utility, capacity_loss: f64) -> f64 {
    let lost = utility.generation_capacity_mwh * capacity_loss;
    utility.generation_capacity_mwh = (utility.generation_capacity_mwh - lost).max(20.0);
    utility.reliability = (utility.reliability - 0.07).clamp(0.35, 0.98);
    utility.reputation = (utility.reputation - 4.0).clamp(0.0, 100.0);
    lost
}

pub fn generation_reliability_after_new_capacity(
    current_reliability: f64,
    existing_generation_capacity_mwh: f64,
    added_capacity_mwh: f64,
) -> f64 {
    let effective_new_equipment_weight =
        added_capacity_mwh.max(0.0) * NEW_GENERATION_RELIABILITY_WEIGHT;
    if effective_new_equipment_weight <= 0.0 {
        return current_reliability.clamp(0.35, 0.98);
    }

    let blended = weighted_average(
        current_reliability,
        existing_generation_capacity_mwh.max(20.0),
        NEW_GENERATION_EQUIPMENT_RELIABILITY,
        effective_new_equipment_weight,
    );
    blended
        .max(current_reliability)
        .min(current_reliability + MAX_GENERATION_RELIABILITY_LIFT)
        .clamp(0.35, 0.98)
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

    fn competition_score(
        &self,
        market: &Market,
        average_rate: f64,
        allocation_kind: CustomerAllocationKind,
    ) -> f64 {
        let (
            rate_sensitivity,
            rate_cap,
            reliability_sensitivity,
            reputation_divisor,
            marketing_cap,
            utilization_bonus,
            utilization_penalty,
        ) = match allocation_kind {
            CustomerAllocationKind::NewConnections => (0.20, 0.65, 1.85, 85.0, 0.85, 0.18, 0.35),
            CustomerAllocationKind::SwitchedAccounts => (0.30, 0.85, 2.35, 110.0, 0.55, 0.24, 0.45),
        };

        let rate_component =
            ((average_rate - self.rate_cents) * rate_sensitivity).clamp(-rate_cap, rate_cap);
        let reliability_component = (self.reliability - 0.74) * reliability_sensitivity;
        let reputation_component = (self.reputation - 50.0) / reputation_divisor;
        let capacity_component = (self.capacity_headroom(market)
            / (market.addressable_customers * 0.08))
            .clamp(-0.35, 0.45);
        let marketing_component = self.marketing_momentum.clamp(0.0, marketing_cap);
        let utilization = self.utilization(market);
        let utilization_component = if utilization <= 0.82 {
            ((0.82 - utilization) / 0.82).clamp(0.0, 1.0) * utilization_bonus
        } else {
            -((utilization - 0.82) / 0.18).clamp(0.0, 1.0) * utilization_penalty
        };
        (1.0 + rate_component
            + reliability_component
            + reputation_component
            + capacity_component
            + utilization_component
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

    fn next_index(&mut self, len: usize) -> usize {
        (self.next_u64() as usize) % len.max(1)
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

#[allow(clippy::too_many_arguments)]
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
        stock_price: varied_pct(rng, stock_price, 0.12, initial_variance)
            .max(stock_price.min(MIN_STOCK_PRICE)),
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

fn shock_kind_label(kind: &ShockKind) -> &'static str {
    match kind {
        ShockKind::RateFreeze => "rate freeze",
        ShockKind::DemandRecession => "demand recession",
        ShockKind::DemandBoom => "demand boom",
        ShockKind::InputCostShock => "input cost shock",
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

#[cfg(test)]
mod tests;
