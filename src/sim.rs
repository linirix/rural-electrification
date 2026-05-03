const DEFAULT_CAMPAIGN_QUARTERS: u32 = 20;
pub const GENERATION_PROJECT_COST: f64 = 18_500.0;
pub const GENERATION_PROJECT_CAPACITY_MWH: f64 = 185.0;
pub const DISTRIBUTION_PROJECT_COST: f64 = 9_500.0;
pub const DISTRIBUTION_PROJECT_CAPACITY: f64 = 230.0;
pub const MIN_GENERATION_PROJECT_MWH: f64 = 60.0;
pub const MAX_GENERATION_PROJECT_MWH: f64 = 650.0;
pub const MIN_DISTRIBUTION_PROJECT_CUSTOMERS: f64 = 80.0;
pub const MAX_DISTRIBUTION_PROJECT_CUSTOMERS: f64 = 900.0;
pub const ADJACENT_EXPANSION_BASE_COST: f64 = 185_000.0;
pub const ADJACENT_EXPANSION_DURATION_QUARTERS: u32 = 4;
pub const ADJACENT_EXPANSION_ADDRESSABLE_CUSTOMERS: f64 = 2_200.0;
pub const ADJACENT_EXPANSION_INITIAL_CUSTOMERS: f64 = 260.0;
pub const ADJACENT_EXPANSION_INCUMBENT_CUSTOMERS: f64 = 380.0;
pub const MAX_ADJACENT_EXPANSIONS: u32 = 3;
pub const REGIONAL_MANDATE_QUARTER: u32 = 40;
pub const REGIONAL_MANDATE_SHARE_TARGET: f64 = 0.58;
pub const REGIONAL_MANDATE_RELIABILITY_TARGET: f64 = 0.82;
pub const REGIONAL_MANDATE_LEVERAGE_LIMIT: f64 = 0.90;
pub const REGIONAL_MANDATE_EXPANSION_TARGET: u32 = 2;
const BASE_ANNUAL_RATE: f64 = 0.052;
const BASE_CREDIT_SPREAD: f64 = 0.018;
const MAINTENANCE_REFERENCE_ASSET_BASE: f64 = 78_000.0;
const ACQUISITION_CASH_ABSORPTION: f64 = 0.90;
const ACQUISITION_DEBT_ASSUMPTION: f64 = 0.70;
const ACQUISITION_CUSTOMER_BASE_VALUE: f64 = 66.0;
const ACQUISITION_REPUTATION_CUSTOMER_VALUE: f64 = 0.30;
const ACQUISITION_DISTRESSED_PREMIUM_DISCOUNT_MAX: f64 = 0.12;
const PUBLIC_INTEREST_CONCESSION_EXPONENT: f64 = 0.85;
const PUBLIC_INTEREST_CONCESSION_FULL_CONTROL_EXCESS: f64 = 0.50;
const PUBLIC_INTEREST_CONCESSION_EV_MULTIPLIER: f64 = 0.24;
const EQUITY_BASE_CASH_DISCOUNT: f64 = 0.60;
const EQUITY_BASE_ASSET_SUPPORT: f64 = 0.65;
const NEW_GENERATION_EQUIPMENT_RELIABILITY: f64 = 0.965;
const NEW_GENERATION_RELIABILITY_WEIGHT: f64 = 0.35;
const MAX_GENERATION_RELIABILITY_LIFT: f64 = 0.085;
const MIN_STOCK_PRICE: f64 = 0.25;
const VARIABLE_COST_REVERSION: f64 = 0.14;
const VARIABLE_COST_PRESSURE_SENSITIVITY: f64 = 2.35;
const VARIABLE_COST_FLOOR_MULTIPLE: f64 = 0.72;
const VARIABLE_COST_CEILING_MULTIPLE: f64 = 1.65;
const MAX_CREDIBLE_RATE_CENTS: f64 = 25.0;
// Opening scenario guardrails only. Live operating rates use dynamic public-tolerance and cost bounds.
const MIN_OPENING_RATE_CENTS: f64 = 7.5;
const MAX_OPENING_RATE_CENTS: f64 = 14.5;
/// Maximum rate premium above public tolerance that the command layer allows for new increases.
pub const MAX_PUBLIC_RATE_PREMIUM_CENTS: f64 = 5.0;
const RECEIVERSHIP_DEBT_TO_ASSETS: f64 = 1.05;
const DISTRESSED_COVERAGE_RECEIVERSHIP_DEBT_TO_ASSETS: f64 = 0.92;
const DISTRESSED_COVERAGE_RECEIVERSHIP: f64 = 0.65;
const HOSTILE_TAKEOVER_MIN_QUARTER: u32 = 12;
const HOSTILE_TAKEOVER_MARKET_CAP_TO_ASSETS: f64 = 0.40;
const HOSTILE_TAKEOVER_DEBT_TO_ASSETS: f64 = 0.85;
const HOSTILE_TAKEOVER_RIVAL_FINANCING_CAPACITY: f64 = 80_000.0;
const HOSTILE_TAKEOVER_RIVAL_DEBT_CAPACITY_WEIGHT: f64 = 0.55;
const MAX_RELIABILITY: f64 = 0.98;
const MIN_MAINTENANCE_RELIABILITY_GAIN: f64 = 0.0005;
const DILIGENCE_DURATION_QUARTERS: u32 = 3;
const DILIGENCE_ALERT_MAX_BACKING: f64 = 8_000.0;
const MERGER_RISK_MIN: f64 = 0.06;
const MERGER_RISK_MAX: f64 = 0.46;
pub const ACQUISITION_STRESS_NOTICE: f64 = 0.38;
pub const ACQUISITION_STRESS_STRAINED: f64 = 0.62;
pub const ACQUISITION_STRESS_DISTRESSED: f64 = 0.88;
pub const ACQUISITION_STRESS_RECEIVERSHIP: f64 = 0.96;
const BOARD_CHECKPOINT_QUARTER: u32 = 8;
const BOARD_CHECKPOINT_SHARE_TARGET: f64 = 0.32;
const BOARD_CHECKPOINT_REPUTATION_PENALTY: f64 = 3.0;
const BOARD_CHECKPOINT_EQUITY_FATIGUE: f64 = 0.30;
const MIN_RATE_TOLERANCE_ADJUSTMENT_CENTS: f64 = -1.25;
const MAX_RATE_TOLERANCE_ADJUSTMENT_CENTS: f64 = 1.35;
const ACQUISITION_RATE_ANCHOR_TOLERANCE_MULTIPLIER: f64 = 0.70;
const ACQUISITION_RATE_ANCHOR_MAX_LIFT_CENTS: f64 = 0.45;
const STARTING_PLAYER_OWNERSHIP: f64 = 0.32;
const MIN_PUBLIC_SHARES_AFTER_BUYBACK: f64 = 500.0;
pub const DEFAULT_MAINTENANCE_MANAGER_TARGET: f64 = 0.85;
pub const DEFAULT_MARKETING_MANAGER_TARGET: f64 = 90.0;
pub const MAINTENANCE_MANAGER_PROFIT_SHARE: f64 = 0.50;
pub const MARKETING_MANAGER_PROFIT_SHARE: f64 = 0.25;
const MIN_MANUAL_OPERATING_SPEND: f64 = 1_000.0;
const MIN_MANAGER_SPEND: f64 = 100.0;
pub const REVIEW_MIN_RATE_SUPPORT_RATIO: f64 = 0.72;
pub const REVIEW_MIN_INTEREST_COVERAGE: f64 = 1.0;
pub const REVIEW_MIN_RELIABILITY: f64 = 0.72;
const REVIEW_MIN_PROFIT: f64 = -500.0;
const BELOW_COST_PRESSURE_RATE_SUPPORT_RATIO: f64 = 0.68;
const PUBLIC_BALANCE_SHEET_DIVERGENCE_PROBABILITY: f64 = 0.25;
const PUBLIC_BALANCE_SHEET_MAX_DIVERGENCE: f64 = 0.20;
const DILIGENCE_ALERT_BASE_PROBABILITY: f64 = 0.48;
const DILIGENCE_ALERT_MIN_PROBABILITY: f64 = 0.18;
const RIVAL_REACTION_RATE_MOVE_CENTS: f64 = 0.60;
const RIVAL_REACTION_MARKETING_SPEND: f64 = 15_000.0;
const RIVAL_REACTION_FINANCING_AMOUNT: f64 = 45_000.0;
const DILIGENCE_ALERT_MAX_PROBABILITY: f64 = 0.88;

fn default_public_balance_sheet_multiplier() -> f64 {
    1.0
}

fn default_diligence_target_alerted() -> bool {
    true
}

mod attribution;
mod competitors;
mod economics;
mod outcome;

use serde::{Deserialize, Serialize};

use attribution::{AttributionContext, PointInTime, quarter_attributions};
use competitors::draw_competitor_name;
#[cfg(test)]
use competitors::{COMPETITOR_NAME_POOL, StartupEntryReason};
use economics::{
    bounded_rate_target, break_even_rate_cents, churn_for_market_with_floor, defensive_rate_floor,
    high_rate_excess, maintenance_asset_scale, maintenance_reliability_gain,
    maintenance_reputation_gain, maintenance_spend_for_reliability_target, positive_amount,
    settle_utility, weighted_average, weighted_rate,
};
#[cfg(test)]
use economics::{churn_rate_for, churn_rate_for_market};
pub use economics::{
    distribution_project_cost, distribution_project_duration, generation_project_cost,
    generation_project_duration, money, public_rate_tolerance,
};
use outcome::interest_coverage;

/// Complete simulation state for one campaign or sandbox run.
///
/// `Game` owns the market, player utility, rivals, pending projects, macro shocks, and current
/// outcome. Consumers normally mutate it through `apply_decision` and `advance_quarter`.
#[derive(Clone, Debug, Serialize, Deserialize)]
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
    #[serde(default)]
    pub acquisition_stress: f64,
    #[serde(default)]
    pub regional_integration: f64,
    pub active_shocks: Vec<ActiveShock>,
    pub startup_index: u32,
    pub adjacent_expansions: u32,
    #[serde(default)]
    pub regional_mandate_completed: bool,
    pub review_completed: bool,
    #[serde(default)]
    pub sandbox_mode: bool,
    pub equity_market_fatigue: f64,
    pub diligence_reports: Vec<DiligenceReport>,
    pub last_report: Option<QuarterReport>,
    pub outcome: Option<Outcome>,
    #[serde(default)]
    pub player_owned_shares: f64,
    #[serde(default)]
    pub player_dividends_received: f64,
    #[serde(default)]
    pub maintenance_manager_target: Option<f64>,
    #[serde(default)]
    pub marketing_manager_target: Option<f64>,
    #[serde(default)]
    rival_reaction: RivalReaction,
    rng: Rng,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
struct RivalReaction {
    quarters_remaining: u32,
    rate_move_cents: f64,
    marketing_spend: f64,
    financing_amount: f64,
}

/// Controls how much seeded starts vary around the baseline scenario.
///
/// `amplitude` is clamped to `0.0..=2.0` when applied: `0.0` gives the fixed baseline, `1.0` is
/// the normal game variance, and `2.0` is a deliberately wide opening spread.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
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

/// A temporary market-wide shock that modifies demand, rates, costs, or valuation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActiveShock {
    pub kind: ShockKind,
    pub quarters_remaining: u32,
}

/// Time-limited due-diligence record for a competitor, including frozen acquisition terms.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiligenceReport {
    pub competitor_name: String,
    pub quarters_remaining: u32,
    pub quoted_terms: AcquisitionTerms,
    #[serde(default = "default_diligence_target_alerted")]
    pub target_alerted: bool,
}

/// Types of macro shocks that can be active during a quarter.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ShockKind {
    RateFreeze,
    DemandRecession,
    DemandBoom,
    InputCostShock,
}

/// Core market conditions shared by the player and competitors.
#[derive(Clone, Debug, Serialize, Deserialize)]
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
    pub rate_tolerance_adjustment_cents: f64,
}

/// Current macro-financing environment that affects debt rates, demand, costs, and valuation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MacroEnvironment {
    pub annual_base_rate: f64,
    pub credit_spread: f64,
    pub demand_index: f64,
    pub cost_pressure: f64,
}

/// Operating and balance-sheet state for the player utility or a competitor.
#[derive(Clone, Debug, Serialize, Deserialize)]
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
    #[serde(default = "default_public_balance_sheet_multiplier")]
    pub public_cash_multiplier: f64,
    #[serde(default = "default_public_balance_sheet_multiplier")]
    pub public_debt_multiplier: f64,
}

/// Capital project under construction and waiting to complete in a future quarter.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub kind: ProjectKind,
    pub quarters_remaining: u32,
}

/// Type and scale of a pending capital project.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProjectKind {
    Generation {
        capacity_mwh: f64,
    },
    Distribution {
        customer_capacity: f64,
    },
    AdjacentTerritory {
        addressable_customers: f64,
        initial_customers: f64,
        incumbent_customers: f64,
    },
}

/// Player command accepted by the simulation core.
///
/// The terminal layer parses text into these variants; the simulation validates cash,
/// financing, market-access, and regulatory constraints when applying them.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Decision {
    BuildGeneration { capacity_mwh: f64 },
    BuildDistribution { customer_capacity: f64 },
    Marketing { spend: f64 },
    IssueStock { amount: f64 },
    BuyBackStock { amount: f64 },
    DeclareDividend { amount: f64 },
    Borrow { amount: f64 },
    RepayDebt { amount: f64 },
    Diligence { competitor_index: usize },
    Acquire { competitor_index: usize },
    EnterAdjacentMarket,
    AdjustRate { delta_cents: f64 },
    Maintenance { spend: f64 },
    HireMaintenanceManager { target_reliability: f64 },
    FireMaintenanceManager,
    HireMarketingManager { target_reputation: f64 },
    FireMarketingManager,
}

/// Result of one processed quarter, suitable for terminal display or automated playtests.
#[derive(Clone, Debug, Serialize, Deserialize)]
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

/// Income-statement and service-output summary for a settled utility in one quarter.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FirmFinances {
    pub revenue: f64,
    pub operating_cost: f64,
    pub interest: f64,
    pub profit: f64,
    pub served_mwh: f64,
    pub unmet_demand_ratio: f64,
}

/// Full acquisition economics for a target, including the post-close player balance sheet.
#[derive(Clone, Debug, Serialize, Deserialize)]
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

/// High-level category for a formal review or terminal operating outcome.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutcomeKind {
    Victory,
    Defeat,
}

/// Current end-state of the game, if one has been reached.
///
/// `can_continue` is true for formal review outcomes that can be cleared with
/// `Game::continue_after_review`; it is false for terminal operating failures such as
/// receivership or lost market access.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outcome {
    pub kind: OutcomeKind,
    pub headline: String,
    pub details: String,
    pub can_continue: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct Rng {
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
            rate_tolerance_adjustment_cents: 0.0,
        };

        let macro_state = MacroEnvironment {
            annual_base_rate: varied_abs(&mut rng, BASE_ANNUAL_RATE, 0.006, initial_variance)
                .clamp(0.038, 0.070),
            credit_spread: varied_abs(&mut rng, BASE_CREDIT_SPREAD, 0.005, initial_variance)
                .clamp(0.008, 0.032),
            demand_index: varied_abs(&mut rng, 0.0, 0.12, initial_variance).clamp(-0.22, 0.22),
            cost_pressure: varied_abs(&mut rng, 0.0, 0.008, initial_variance).clamp(-0.014, 0.018),
        };

        let player = starting_utility(
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
        );
        let player_owned_shares = player.shares * STARTING_PLAYER_OWNERSHIP;

        Self {
            quarter: 0,
            campaign_quarters: DEFAULT_CAMPAIGN_QUARTERS,
            market,
            macro_state,
            player,
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
            acquisition_stress: 0.0,
            regional_integration: 0.0,
            active_shocks: Vec::new(),
            startup_index: 0,
            adjacent_expansions: 0,
            regional_mandate_completed: false,
            review_completed: false,
            sandbox_mode: false,
            equity_market_fatigue: 0.0,
            diligence_reports: Vec::new(),
            last_report: None,
            outcome: None,
            player_owned_shares,
            player_dividends_received: 0.0,
            maintenance_manager_target: None,
            marketing_manager_target: None,
            rival_reaction: RivalReaction::default(),
            rng,
        }
    }

    pub fn date_label(&self) -> String {
        let year = self.quarter / 4 + 1;
        let quarter = self.quarter % 4 + 1;
        format!("Year {year} Q{quarter}")
    }

    pub fn player_ownership(&self) -> f64 {
        if self.player.shares <= 0.0 {
            0.0
        } else {
            (self.player_owned_shares / self.player.shares).clamp(0.0, 1.0)
        }
    }

    pub fn player_equity_value(&self) -> f64 {
        self.player_owned_shares.max(0.0) * self.player.stock_price.max(0.0)
    }

    pub fn player_wealth(&self) -> f64 {
        self.player_equity_value() + self.player_dividends_received.max(0.0)
    }

    pub fn normalize_after_load(&mut self) {
        if self.player_owned_shares <= 0.0 && self.player.shares > 0.0 {
            self.player_owned_shares = self.player.shares * STARTING_PLAYER_OWNERSHIP;
        }
        if !self.player_owned_shares.is_finite() {
            self.player_owned_shares = 0.0;
        }
        if !self.player_dividends_received.is_finite() || self.player_dividends_received < 0.0 {
            self.player_dividends_received = 0.0;
        }
        self.maintenance_manager_target = self
            .maintenance_manager_target
            .and_then(|target| validate_maintenance_manager_target(target).ok());
        self.marketing_manager_target = self
            .marketing_manager_target
            .and_then(|target| validate_marketing_manager_target(target).ok());
        if !self.rival_reaction.rate_move_cents.is_finite()
            || !self.rival_reaction.marketing_spend.is_finite()
            || !self.rival_reaction.financing_amount.is_finite()
        {
            self.rival_reaction = RivalReaction::default();
        }
    }

    pub fn validate_loaded_state(&self) -> Result<(), String> {
        validate_loaded_utility("player", &self.player, true)?;
        for (index, competitor) in self.competitors.iter().enumerate() {
            validate_loaded_utility(&format!("competitor {}", index + 1), competitor, false)?;
        }
        if !self.market.addressable_customers.is_finite()
            || self.market.addressable_customers <= 0.0
        {
            return Err("market addressable customers must be positive and finite".to_string());
        }
        if !self.market.avg_mwh_per_customer.is_finite() || self.market.avg_mwh_per_customer <= 0.0
        {
            return Err("market average usage must be positive and finite".to_string());
        }
        if !self.market.variable_cost_per_mwh.is_finite() || self.market.variable_cost_per_mwh < 0.0
        {
            return Err("market variable cost must be finite and nonnegative".to_string());
        }
        let connected = self.total_connected_customers();
        let plausible_limit = self.market.addressable_customers * 1.25 + 250.0;
        if connected > plausible_limit {
            return Err(format!(
                "connected customers {:.0} exceed plausible market size {:.0}",
                connected, plausible_limit
            ));
        }
        Ok(())
    }

    pub fn player_maintenance_response_scale(&self) -> f64 {
        maintenance_asset_scale(&self.player)
    }

    fn record_rival_reaction(
        &mut self,
        rate_move_cents: f64,
        marketing_spend: f64,
        financing_amount: f64,
    ) {
        self.rival_reaction.quarters_remaining = self.rival_reaction.quarters_remaining.max(1);
        if rate_move_cents.abs() > self.rival_reaction.rate_move_cents.abs() {
            self.rival_reaction.rate_move_cents = rate_move_cents;
        }
        self.rival_reaction.marketing_spend = self
            .rival_reaction
            .marketing_spend
            .max(marketing_spend.max(0.0));
        self.rival_reaction.financing_amount = self
            .rival_reaction
            .financing_amount
            .max(financing_amount.max(0.0));
    }

    pub(super) fn decay_rival_reaction(&mut self) {
        if self.rival_reaction.quarters_remaining == 0 {
            self.rival_reaction = RivalReaction::default();
            return;
        }

        self.rival_reaction.quarters_remaining -= 1;
        if self.rival_reaction.quarters_remaining == 0 {
            self.rival_reaction = RivalReaction::default();
        }
    }

    pub fn apply_decision(&mut self, decision: Decision) -> Result<String, String> {
        if self.outcome.is_some() {
            return Err(
                "The market review is paused on a final result. Use 'continue' to keep playing if the review allows it."
                    .to_string(),
            );
        }

        match decision {
            Decision::BuildGeneration { capacity_mwh } => self.apply_build_generation(capacity_mwh),
            Decision::BuildDistribution { customer_capacity } => {
                self.apply_build_distribution(customer_capacity)
            }
            Decision::EnterAdjacentMarket => self.apply_enter_adjacent_market(),
            Decision::Marketing { spend } => self.apply_marketing(spend),
            Decision::IssueStock { amount } => self.apply_issue_stock(amount),
            Decision::BuyBackStock { amount } => self.apply_buyback_stock(amount),
            Decision::DeclareDividend { amount } => self.apply_declare_dividend(amount),
            Decision::Borrow { amount } => self.apply_borrow(amount),
            Decision::RepayDebt { amount } => self.apply_repay_debt(amount),
            Decision::Diligence { competitor_index } => self.apply_diligence(competitor_index),
            Decision::Acquire { competitor_index } => self.apply_acquire(competitor_index),
            Decision::AdjustRate { delta_cents } => self.apply_adjust_rate(delta_cents),
            Decision::HireMaintenanceManager { target_reliability } => {
                self.apply_hire_maintenance_manager(target_reliability)
            }
            Decision::FireMaintenanceManager => self.apply_fire_maintenance_manager(),
            Decision::HireMarketingManager { target_reputation } => {
                self.apply_hire_marketing_manager(target_reputation)
            }
            Decision::FireMarketingManager => self.apply_fire_marketing_manager(),
            Decision::Maintenance { spend } => self.apply_maintenance(spend),
        }
    }

    fn apply_build_generation(&mut self, capacity_mwh: f64) -> Result<String, String> {
        let requested_capacity_mwh = positive_amount(capacity_mwh, "generation project size")?;
        let capacity_mwh =
            requested_capacity_mwh.clamp(MIN_GENERATION_PROJECT_MWH, MAX_GENERATION_PROJECT_MWH);
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
        ) + &project_clamp_note(
            requested_capacity_mwh,
            capacity_mwh,
            MIN_GENERATION_PROJECT_MWH,
            MAX_GENERATION_PROJECT_MWH,
            "MWh/quarter",
        ))
    }

    fn apply_build_distribution(&mut self, customer_capacity: f64) -> Result<String, String> {
        let requested_customer_capacity =
            positive_amount(customer_capacity, "distribution project size")?;
        let customer_capacity = requested_customer_capacity.clamp(
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
        ) + &project_clamp_note(
            requested_customer_capacity,
            customer_capacity,
            MIN_DISTRIBUTION_PROJECT_CUSTOMERS,
            MAX_DISTRIBUTION_PROJECT_CUSTOMERS,
            "customers",
        ))
    }

    fn apply_enter_adjacent_market(&mut self) -> Result<String, String> {
        self.require_adjacent_expansion_ready()?;
        let cost = self.adjacent_expansion_cost();
        self.require_cash(cost)?;
        self.player.cash -= cost;
        self.player.asset_base += cost;
        self.pending_projects.push(Project {
            name: "adjacent territory entry".to_string(),
            kind: ProjectKind::AdjacentTerritory {
                addressable_customers: ADJACENT_EXPANSION_ADDRESSABLE_CUSTOMERS,
                initial_customers: ADJACENT_EXPANSION_INITIAL_CUSTOMERS,
                incumbent_customers: ADJACENT_EXPANSION_INCUMBENT_CUSTOMERS,
            },
            quarters_remaining: ADJACENT_EXPANSION_DURATION_QUARTERS,
        });
        Ok(format!(
            "Started adjacent territory entry for {}. It will open about {:.0} addressable customers in {} quarter(s), with an initial foothold and a local incumbent rival.",
            money(cost),
            ADJACENT_EXPANSION_ADDRESSABLE_CUSTOMERS,
            ADJACENT_EXPANSION_DURATION_QUARTERS
        ))
    }

    fn apply_marketing(&mut self, spend: f64) -> Result<String, String> {
        let spend = positive_amount(spend, "marketing spend")?;
        let spend = spend.max(MIN_MANUAL_OPERATING_SPEND);
        self.require_cash(spend)?;
        self.player.cash -= spend;
        self.player.marketing_momentum += (spend / 5_000.0) * 0.13;
        self.player.reputation = (self.player.reputation + spend / 4_000.0).clamp(0.0, 100.0);
        if spend >= RIVAL_REACTION_MARKETING_SPEND {
            self.record_rival_reaction(0.0, spend, 0.0);
        }
        let regional_relief = self.reduce_regional_integration(spend / 160_000.0, "marketing");
        Ok(format!(
            "Spent {} on customer acquisition, financing offers, and sales coverage.",
            money(spend)
        ) + &regional_relief)
    }

    fn apply_issue_stock(&mut self, amount: f64) -> Result<String, String> {
        let amount = positive_amount(amount, "stock issuance")?;
        let old_shares = self.player.shares.max(1.0);
        let financing_base = self.equity_issuance_base();
        let pressure = amount / financing_base;
        // Fatigue is accumulated on the same unitless scale as issue pressure, so repeat
        // offerings price like a larger single offering even across separate commands.
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
        let issuance_fee = amount * fee_rate;
        let net_proceeds = amount - issuance_fee;
        let transaction_pre_money = issue_price * old_shares;
        let post_money_market_cap =
            (transaction_pre_money + net_proceeds).max((old_shares + new_shares) * MIN_STOCK_PRICE);
        self.player.cash += net_proceeds;
        self.player.shares += new_shares;
        self.player.stock_price = (post_money_market_cap / self.player.shares).max(MIN_STOCK_PRICE);
        self.player.reputation =
            (self.player.reputation - effective_pressure * 1.5 - effective_pressure_squared * 1.5)
                .clamp(0.0, 100.0);
        self.equity_market_fatigue = (self.equity_market_fatigue + pressure * 0.80).clamp(0.0, 6.0);
        if amount >= RIVAL_REACTION_FINANCING_AMOUNT {
            self.record_rival_reaction(0.0, 0.0, amount);
        }
        Ok(format!(
            "Issued {:.0} shares at ${:.2}. Gross {}, net cash {} after fees; dilution reset stock to ${:.2}.",
            new_shares,
            issue_price,
            money(amount),
            money(net_proceeds),
            self.player.stock_price
        ))
    }

    fn apply_buyback_stock(&mut self, amount: f64) -> Result<String, String> {
        let buyback_share_floor = self
            .player_owned_shares
            .max(MIN_PUBLIC_SHARES_AFTER_BUYBACK);
        if self.player.shares <= buyback_share_floor {
            return Err("The company has too few public shares to buy back more.".to_string());
        }
        let amount = positive_amount(amount, "stock buyback")?;
        let old_market_cap = self.player.market_cap().max(1.0);
        let old_shares = self.player.shares.max(1.0);
        let pressure = amount / old_market_cap;
        let remaining_cash_if_filled = self.player.cash - amount.min(self.player.cash);
        let liquidity_drag =
            ((8_000.0 - remaining_cash_if_filled).max(0.0) / 8_000.0 * 0.09).clamp(0.0, 0.09);
        let overextension_drag = ((amount / self.player.cash.max(1.0)) - 0.60).max(0.0) * 0.12;
        // The repurchase premium is the price paid to coax sellers into the tender.
        // It should not re-anchor the whole pre-buyback equity value. The smaller
        // capital return signal below models the confidence/scarcity effect on the
        // remaining float after cash has actually left the balance sheet.
        let repurchase_premium = (pressure.min(1.0) * 0.55 + pressure.powf(1.20) * 0.04
            - liquidity_drag
            - overextension_drag
            - self.equity_market_fatigue * 0.45)
            .clamp(-0.18, 0.30);
        let repurchase_price =
            (self.player.stock_price * (1.0 + repurchase_premium)).max(MIN_STOCK_PRICE);
        let max_shares = (self.player.shares - buyback_share_floor).max(0.0);
        let shares_bought = (amount / repurchase_price).min(max_shares);
        if shares_bought < 1.0 {
            return Err("The buyback is too small to retire a meaningful share count.".to_string());
        }
        let actual_spend = shares_bought * repurchase_price;
        self.require_cash(actual_spend)?;
        let remaining_shares = self.player.shares - shares_bought;
        let float_retired = shares_bought / old_shares;
        let realistic_post_market_cap =
            (old_market_cap - actual_spend).max(remaining_shares * MIN_STOCK_PRICE);
        let realistic_price = realistic_post_market_cap / remaining_shares.max(1.0);
        let capital_return_signal =
            (float_retired * 0.30 + pressure.min(1.0) * 0.04).clamp(0.0, 0.10);
        let signal_lift = (capital_return_signal
            - liquidity_drag
            - overextension_drag
            - self.equity_market_fatigue * 0.45)
            .clamp(-0.08, 0.06);
        self.player.cash -= actual_spend;
        self.player.shares = remaining_shares;
        self.player.stock_price = (realistic_price * (1.0 + signal_lift)).max(MIN_STOCK_PRICE);
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

    fn apply_declare_dividend(&mut self, amount: f64) -> Result<String, String> {
        let amount = positive_amount(amount, "dividend")?;
        self.require_cash(amount)?;
        let shares = self.player.shares.max(1.0);
        let dividend_per_share = amount / shares;
        let founder_payment = dividend_per_share * self.player_owned_shares.max(0.0);
        self.player.cash -= amount;
        self.player.stock_price =
            (self.player.stock_price - dividend_per_share).max(MIN_STOCK_PRICE);
        self.player_dividends_received += founder_payment;
        Ok(format!(
            "Declared a {} dividend (${:.2}/share). Founder received {}; company cash fell by {}.",
            money(amount),
            dividend_per_share,
            money(founder_payment),
            money(amount)
        ))
    }

    fn apply_borrow(&mut self, amount: f64) -> Result<String, String> {
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
        if amount >= RIVAL_REACTION_FINANCING_AMOUNT {
            self.record_rival_reaction(0.0, 0.0, amount);
        }
        let leverage = self.player.debt_to_assets();
        let annual_rate = self.macro_state.annual_interest_rate_for(&self.player);
        if leverage > 0.82 {
            self.player.reputation = (self.player.reputation - 3.2).clamp(0.0, 100.0);
        } else if leverage > 0.68 {
            self.player.reputation = (self.player.reputation - 1.4).clamp(0.0, 100.0);
        }
        Ok(format!(
            "Borrowed {} at a current floating rate of {:.1}% annual. Debt/assets now {:.0}%.",
            money(amount),
            annual_rate * 100.0,
            leverage * 100.0
        ))
    }

    fn apply_repay_debt(&mut self, amount: f64) -> Result<String, String> {
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

    fn apply_diligence(&mut self, competitor_index: usize) -> Result<String, String> {
        if competitor_index >= self.competitors.len() {
            return Err("No competitor has that number.".to_string());
        }
        let cost = self
            .diligence_cost(competitor_index)
            .expect("competitor index checked before diligence");
        self.require_cash(cost)?;
        self.player.cash -= cost;
        let name = self.competitors[competitor_index].name.clone();
        let mut alert_message = None;
        if let Some(report) = self
            .diligence_reports
            .iter_mut()
            .find(|report| report.competitor_name == name && report.quarters_remaining > 0)
        {
            report.quarters_remaining = DILIGENCE_DURATION_QUARTERS;
        } else {
            let target_alerted = self.diligence_alert_leaks(competitor_index);
            alert_message = Some(if target_alerted {
                self.apply_diligence_alert_response(competitor_index)
            } else {
                format!(
                    "The review appears to have stayed quiet; {} made no visible defensive move.",
                    name
                )
            });
            let quoted_terms = self
                .current_acquisition_terms(competitor_index)
                .expect("competitor index checked before diligence terms quote");
            self.diligence_reports
                .retain(|report| report.competitor_name != name);
            self.diligence_reports.push(DiligenceReport {
                competitor_name: name.clone(),
                quarters_remaining: DILIGENCE_DURATION_QUARTERS,
                quoted_terms,
                target_alerted,
            });
        }
        let alert_suffix = alert_message
            .map(|message| format!(" {message}"))
            .unwrap_or_else(|| {
                " Existing diligence was extended without a new market signal.".to_string()
            });
        Ok(format!(
            "Completed diligence on {name} for {}. Exact acquisition terms are available for {} quarter(s).{}",
            money(cost),
            DILIGENCE_DURATION_QUARTERS,
            alert_suffix
        ))
    }

    fn apply_acquire(&mut self, competitor_index: usize) -> Result<String, String> {
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

        let had_diligence = self.has_diligence(competitor_index);
        let public_estimate = (!had_diligence)
            .then(|| self.public_acquisition_estimate(competitor_index))
            .flatten();
        let terms = self
            .acquisition_terms(competitor_index)
            .expect("competitor index checked before acquisition");
        if !had_diligence && self.player.cash + 0.01 < terms.price {
            return Err(format!(
                "The undiligenced close priced at {}, but cash on hand is only {}. Diligence would reveal and freeze exact terms before financing.",
                money(terms.price),
                money(self.player.cash)
            ));
        }
        self.require_cash(terms.price)?;
        let rate_anchor_suffix = self.apply_acquisition_rate_anchor_shift(competitor_index);
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
        let pre_close_integration_strain = self.integration_strain;
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
        let acquisition_stress_add = acquisition_covenant_stress_score(
            &acquired,
            &terms,
            starting_customers,
            pre_close_integration_strain,
            !had_diligence,
        );
        self.acquisition_stress =
            (self.acquisition_stress + acquisition_stress_add).clamp(0.0, 1.8);
        self.diligence_reports
            .retain(|report| report.competitor_name != acquired_name);
        let execution_suffix = self.apply_merger_execution_outcome(
            &acquired,
            &terms,
            starting_customers,
            pre_close_integration_strain,
        );
        let diligence_suffix = if had_diligence {
            String::new()
        } else if let Some(public_estimate) = public_estimate {
            format!(
                " Closed without diligence; the public estimate centered on {} before hidden balance-sheet details resolved at close.",
                money(public_estimate.price)
            )
        } else {
            " Closed without diligence; hidden balance-sheet details resolved at close.".to_string()
        };
        let financing_suffix = acquisition_financing_suffix(acquisition_stress_add);
        Ok(format!(
            "Acquired {acquired_name} for {} (net of {} absorbed cash, plus {} assumed debt). Integration will take {} quarter(s).",
            money(terms.price),
            money(terms.absorbed_cash),
            money(terms.assumed_debt),
            self.acquisition_cooldown
        ) + &diligence_suffix
            + &financing_suffix
            + &execution_suffix
            + &rate_anchor_suffix)
    }

    fn apply_adjust_rate(&mut self, delta_cents: f64) -> Result<String, String> {
        if !delta_cents.is_finite() {
            return Err("The rate change must be a finite number.".to_string());
        }
        if delta_cents > 0.0 && self.rate_frozen() {
            return Err(
                "The market-wide rate freeze blocks any rate increase right now.".to_string(),
            );
        }
        let old = self.player.rate_cents;
        let requested_rate = self.player.rate_cents + delta_cents;
        if requested_rate <= 0.0 {
            return Err("Rates must stay above 0.0c/kWh; use a positive tariff.".to_string());
        }
        let public_ceiling = public_rate_tolerance(&self.market) + MAX_PUBLIC_RATE_PREMIUM_CENTS;
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
        if actual_delta.abs() >= RIVAL_REACTION_RATE_MOVE_CENTS {
            self.record_rival_reaction(actual_delta, 0.0, 0.0);
        }
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

    fn apply_hire_maintenance_manager(
        &mut self,
        target_reliability: f64,
    ) -> Result<String, String> {
        let target = validate_maintenance_manager_target(target_reliability)?;
        self.maintenance_manager_target = Some(target);
        Ok(format!(
            "Hired a maintenance manager to maintain reliability near {:.0}%, spending up to {:.0}% of last quarter's profit.",
            target * 100.0,
            MAINTENANCE_MANAGER_PROFIT_SHARE * 100.0
        ))
    }

    fn apply_fire_maintenance_manager(&mut self) -> Result<String, String> {
        if self.maintenance_manager_target.take().is_some() {
            Ok("Dismissed the maintenance manager.".to_string())
        } else {
            Ok("No maintenance manager was on staff.".to_string())
        }
    }

    fn apply_hire_marketing_manager(&mut self, target_reputation: f64) -> Result<String, String> {
        let target = validate_marketing_manager_target(target_reputation)?;
        self.marketing_manager_target = Some(target);
        Ok(format!(
            "Hired a marketing manager to build reputation toward {:.0}, spending up to {:.0}% of last quarter's profit.",
            target,
            MARKETING_MANAGER_PROFIT_SHARE * 100.0
        ))
    }

    fn apply_fire_marketing_manager(&mut self) -> Result<String, String> {
        if self.marketing_manager_target.take().is_some() {
            Ok("Dismissed the marketing manager.".to_string())
        } else {
            Ok("No marketing manager was on staff.".to_string())
        }
    }

    fn apply_maintenance(&mut self, spend: f64) -> Result<String, String> {
        let spend = positive_amount(spend, "maintenance spend")?;
        let spend = spend.max(MIN_MANUAL_OPERATING_SPEND);
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
        let regional_relief = self.reduce_regional_integration(spend / 140_000.0, "service work");
        Ok(format!(
            "Spent {} on reliability work; gained {:.1} reliability points.",
            money(spend),
            actual_gain * 100.0
        ) + &regional_relief)
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
        self.decay_rival_reaction();
        self.apply_integration_strain(&mut events);
        self.apply_regional_integration_strain(&mut events);
        let manager_spend = self.apply_operating_managers(&mut events);

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
        let mut player_finances = settle_utility(
            &mut self.player,
            &self.market,
            &self.macro_state,
            cost_multiplier,
            true,
            &mut events,
        );
        if manager_spend > 0.0 {
            player_finances.operating_cost += manager_spend;
            player_finances.profit -= manager_spend;
        }
        self.apply_below_cost_pricing_pressure(&player_finances, &mut events);
        let mut system_demanded_mwh = demanded_mwh_from_finances(&player_finances);
        let mut system_unmet_mwh = unmet_mwh_from_finances(&player_finances);
        let competitor_count = self.competitors.len();
        for index in 0..competitor_count {
            let competitor = &mut self.competitors[index];
            let competitor_finances = settle_utility(
                competitor,
                &self.market,
                &self.macro_state,
                cost_multiplier,
                false,
                &mut events,
            );
            system_demanded_mwh += demanded_mwh_from_finances(&competitor_finances);
            system_unmet_mwh += unmet_mwh_from_finances(&competitor_finances);
        }
        let system_unmet_demand_ratio = if system_demanded_mwh <= 0.0 {
            0.0
        } else {
            system_unmet_mwh / system_demanded_mwh
        };
        self.update_public_rate_tolerance(system_unmet_demand_ratio, &mut events);

        self.update_stock_price(&player_finances);
        self.apply_acquisition_stress_pressure(&player_finances, &mut events);
        self.quarter += 1;
        if self.acquisition_cooldown > 0 {
            self.acquisition_cooldown -= 1;
        }
        self.advance_diligence_reports();
        self.apply_board_checkpoint(&mut events);
        self.check_outcome(&player_finances);
        self.decay_acquisition_stress(&player_finances);

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
            starting: PointInTime {
                customers: starting_customers,
                market_share: starting_market_share,
                total_connected: starting_total_connected,
                reliability: starting_reliability,
                headroom: starting_headroom,
            },
            ending: PointInTime {
                customers: ending_customers,
                market_share: ending_market_share,
                total_connected: ending_total_connected,
                reliability: ending_reliability,
                headroom: ending_headroom,
            },
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
        let next = if self.review_completed && !self.regional_mandate_completed {
            format!(
                " The next board mandate is Year 10 regional leadership: {:.0}% share, {} adjacent territories, {:.0}% reliability, and debt/assets below {:.0}%.",
                REGIONAL_MANDATE_SHARE_TARGET * 100.0,
                REGIONAL_MANDATE_EXPANSION_TARGET,
                REGIONAL_MANDATE_RELIABILITY_TARGET * 100.0,
                REGIONAL_MANDATE_LEVERAGE_LIMIT * 100.0
            )
        } else {
            " Operations now continue without a fixed end date.".to_string()
        };
        Ok(format!("Continuing after {headline}.{next}"))
    }

    pub fn enable_sandbox_mode(&mut self) -> Result<String, String> {
        if let Some(outcome) = &self.outcome
            && !outcome.can_continue
        {
            return Err(
                "This outcome is terminal; the company cannot continue operating.".to_string(),
            );
        }

        let prior_outcome = self.outcome.take().map(|outcome| outcome.headline);
        let already_active = self.sandbox_mode;
        self.sandbox_mode = true;

        let suffix = " Board reviews, mandates, and checkpoint penalties are disabled; operating failures such as receivership, market-access loss, and hostile takeover still apply.";
        if already_active {
            Ok(format!("Sandbox mode is already active.{suffix}"))
        } else if let Some(headline) = prior_outcome {
            Ok(format!("Sandbox mode enabled after {headline}.{suffix}"))
        } else {
            Ok(format!("Sandbox mode enabled.{suffix}"))
        }
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
        (self.player.asset_base * self.borrowing_limit_ratio() - self.player.debt).max(0.0)
    }

    pub fn borrowing_limit_ratio(&self) -> f64 {
        let macro_limit = self.macro_state.borrowing_limit_ratio();
        let service_drag = (0.78 - self.player.reliability).clamp(0.0, 0.20) * 0.42;
        let reputation_drag = ((50.0 - self.player.reputation) / 100.0).clamp(0.0, 0.18) * 0.48;
        let stress_drag = (self.acquisition_stress - ACQUISITION_STRESS_NOTICE).clamp(0.0, 0.80)
            * 0.10
            + (self.acquisition_stress - ACQUISITION_STRESS_DISTRESSED).clamp(0.0, 0.60) * 0.06;
        let break_even_rate = self.player_break_even_rate_cents();
        let rate_support_drag = if break_even_rate <= 0.0 {
            0.0
        } else {
            let support_ratio = self.player.rate_cents / break_even_rate;
            (REVIEW_MIN_RATE_SUPPORT_RATIO - support_ratio).clamp(0.0, 0.35) * 0.15
        };
        let mut coverage_drag: f64 = 0.0;
        if let Some(report) = &self.last_report {
            if report.profit < 0.0 {
                coverage_drag += ((-report.profit) / report.revenue.max(1.0)).clamp(0.0, 0.12);
            }
            let coverage = interest_coverage(report.profit, report.interest);
            if coverage < 1.0 {
                coverage_drag += ((1.0 - coverage) * 0.18).clamp(0.0, 0.18);
            } else if coverage < 1.35 {
                coverage_drag += ((1.35 - coverage) * 0.06).clamp(0.0, 0.04);
            }
        }

        (macro_limit
            - service_drag
            - reputation_drag
            - stress_drag
            - rate_support_drag
            - coverage_drag)
            .clamp(0.40, macro_limit)
    }

    pub fn adjacent_expansion_cost(&self) -> f64 {
        ADJACENT_EXPANSION_BASE_COST * (1.0 + f64::from(self.adjacent_expansions) * 0.50)
    }

    pub fn adjacent_expansion_duration(&self) -> u32 {
        ADJACENT_EXPANSION_DURATION_QUARTERS
    }

    pub fn adjacent_expansion_pending(&self) -> bool {
        self.pending_projects
            .iter()
            .any(|project| matches!(project.kind, ProjectKind::AdjacentTerritory { .. }))
    }

    pub fn regional_mandate_due_in(&self) -> u32 {
        REGIONAL_MANDATE_QUARTER.saturating_sub(self.quarter)
    }

    pub fn regional_mandate_active(&self) -> bool {
        if self.sandbox_mode {
            return false;
        }
        self.review_completed || self.adjacent_expansion_pending() || self.adjacent_expansions > 0
    }

    pub fn regional_mandate_met(&self) -> bool {
        self.adjacent_expansions >= REGIONAL_MANDATE_EXPANSION_TARGET
            && self.market_share() >= REGIONAL_MANDATE_SHARE_TARGET
            && self.player.reliability >= REGIONAL_MANDATE_RELIABILITY_TARGET
            && self.player.debt_to_assets() <= REGIONAL_MANDATE_LEVERAGE_LIMIT
            && self.regional_integration <= 0.12
    }

    pub fn adjacent_expansion_blocker(&self) -> Option<String> {
        if self.quarter < 8 && !self.review_completed && !self.sandbox_mode {
            return Some("Adjacent expansion unlocks in Year 3.".to_string());
        }
        if self.adjacent_expansion_pending() {
            return Some("Adjacent territory entry is already in the pipeline.".to_string());
        }
        if self.adjacent_expansions >= MAX_ADJACENT_EXPANSIONS {
            return Some(format!(
                "Metro has already entered {MAX_ADJACENT_EXPANSIONS} adjacent territories; broader regional expansion is not modeled yet."
            ));
        }
        if self.player.reliability < 0.82 && !self.sandbox_mode {
            return Some(
                "The board will not underwrite expansion until reliability is at least 82%."
                    .to_string(),
            );
        }
        if self.market_share() < 0.34 && !self.sandbox_mode {
            return Some(
                "The board wants at least 34% core-market share before funding adjacent expansion."
                    .to_string(),
            );
        }
        if self.player.debt_to_assets() > 0.95 {
            return Some(
                "Lenders will not finance adjacent expansion while debt/assets is above 95%."
                    .to_string(),
            );
        }
        None
    }

    fn require_adjacent_expansion_ready(&self) -> Result<(), String> {
        if let Some(blocker) = self.adjacent_expansion_blocker() {
            Err(blocker)
        } else {
            Ok(())
        }
    }

    fn update_public_rate_tolerance(
        &mut self,
        system_unmet_demand_ratio: f64,
        events: &mut Vec<String>,
    ) {
        let old_adjustment = self.market.rate_tolerance_adjustment_cents;
        let reliability = self.weighted_market_reliability();
        let demand_signal = self.macro_state.demand_index.clamp(-0.60, 0.70) * 0.55;
        let service_signal = if reliability >= 0.84 && system_unmet_demand_ratio <= 0.012 {
            ((reliability - 0.84) / 0.10).clamp(0.0, 1.0) * 0.55
        } else if reliability < 0.78 {
            -((0.78 - reliability) / 0.18).clamp(0.0, 1.0) * 0.70
        } else {
            0.0
        };
        let unmet_service_drag = -(system_unmet_demand_ratio * 5.0).clamp(0.0, 0.90);
        let credit_stress =
            ((self.macro_state.benchmark_credit_rate() - 0.075).max(0.0) * 4.5).clamp(0.0, 0.55);
        let cost_stress = (self.macro_state.cost_pressure.max(0.0) * 7.0).clamp(0.0, 0.45);
        let easy_credit_bonus = if self.macro_state.benchmark_credit_rate() < 0.058 {
            0.12
        } else {
            0.0
        };
        let shock_signal = self
            .active_shocks
            .iter()
            .map(|shock| match shock.kind {
                ShockKind::DemandBoom => 0.25,
                ShockKind::DemandRecession => -0.35,
                ShockKind::InputCostShock => -0.20,
                ShockKind::RateFreeze => -0.25,
            })
            .sum::<f64>();
        let target_adjustment =
            (demand_signal + service_signal + unmet_service_drag - credit_stress - cost_stress
                + easy_credit_bonus
                + shock_signal)
                .clamp(
                    MIN_RATE_TOLERANCE_ADJUSTMENT_CENTS,
                    MAX_RATE_TOLERANCE_ADJUSTMENT_CENTS,
                );
        let new_adjustment = (old_adjustment * 0.68 + target_adjustment * 0.32).clamp(
            MIN_RATE_TOLERANCE_ADJUSTMENT_CENTS,
            MAX_RATE_TOLERANCE_ADJUSTMENT_CENTS,
        );
        self.market.rate_tolerance_adjustment_cents = new_adjustment;

        let delta = new_adjustment - old_adjustment;
        if delta >= 0.14 {
            events.push(format!(
                "Public rate tolerance improved by {:.1}c as demand and service conditions supported higher tariffs.",
                delta
            ));
        } else if delta <= -0.14 {
            events.push(format!(
                "Public rate tolerance tightened by {:.1}c under financial stress or weaker service quality.",
                delta.abs()
            ));
        }
    }

    fn weighted_market_reliability(&self) -> f64 {
        let mut weighted = self.player.reliability * self.player.customers.max(1.0);
        let mut total = self.player.customers.max(1.0);
        for competitor in &self.competitors {
            let weight = competitor.customers.max(1.0);
            weighted += competitor.reliability * weight;
            total += weight;
        }
        weighted / total.max(1.0)
    }

    pub fn acquisition_rate_anchor_lift_cents(&self, competitor_index: usize) -> Option<f64> {
        let competitor = self.competitors.get(competitor_index)?;
        let target_rate = competitor.rate_cents;
        let target_customers = competitor.customers.max(0.0);
        let alternative_rate = self.alternative_rate_for_competitor(competitor_index);
        let low_rate_gap = (alternative_rate - target_rate).max(0.0);
        if low_rate_gap < 0.20 || target_customers <= 0.0 {
            return Some(0.0);
        }

        let target_share =
            (target_customers / self.total_connected_customers().max(1.0)).clamp(0.0, 1.0);
        let requested_lift =
            (low_rate_gap * target_share * ACQUISITION_RATE_ANCHOR_TOLERANCE_MULTIPLIER)
                .clamp(0.0, ACQUISITION_RATE_ANCHOR_MAX_LIFT_CENTS);
        let old_adjustment = self.market.rate_tolerance_adjustment_cents;
        let new_adjustment = (old_adjustment + requested_lift).clamp(
            MIN_RATE_TOLERANCE_ADJUSTMENT_CENTS,
            MAX_RATE_TOLERANCE_ADJUSTMENT_CENTS,
        );
        Some((new_adjustment - old_adjustment).max(0.0))
    }

    fn apply_acquisition_rate_anchor_shift(&mut self, competitor_index: usize) -> String {
        let actual_lift = self
            .acquisition_rate_anchor_lift_cents(competitor_index)
            .unwrap_or(0.0);
        if actual_lift < 0.025 {
            return String::new();
        }

        let old_adjustment = self.market.rate_tolerance_adjustment_cents;
        self.market.rate_tolerance_adjustment_cents = (old_adjustment + actual_lift).clamp(
            MIN_RATE_TOLERANCE_ADJUSTMENT_CENTS,
            MAX_RATE_TOLERANCE_ADJUSTMENT_CENTS,
        );
        let actual_lift = (self.market.rate_tolerance_adjustment_cents - old_adjustment).max(0.0);
        if actual_lift >= 0.05 {
            format!(
                " Removing a low-rate alternative lifted public rate tolerance by {:.1}c.",
                actual_lift
            )
        } else {
            String::new()
        }
    }

    fn equity_issuance_base(&self) -> f64 {
        let market_cap = self.player.market_cap().max(1.0);
        let cash_adjusted_cap = market_cap - self.player.cash.max(0.0) * EQUITY_BASE_CASH_DISCOUNT;
        let asset_support = self.player.asset_base.max(1.0) * EQUITY_BASE_ASSET_SUPPORT;
        cash_adjusted_cap
            .max(asset_support)
            .max(self.player.shares * MIN_STOCK_PRICE)
            .max(1.0)
    }

    pub fn player_annual_interest_rate(&self) -> f64 {
        self.macro_state.annual_interest_rate_for(&self.player)
    }

    pub fn player_break_even_rate_cents(&self) -> f64 {
        break_even_rate_cents(
            &self.player,
            &self.market,
            &self.macro_state,
            self.cost_shock_multiplier(),
        )
    }

    pub fn player_rate_support_ratio(&self) -> f64 {
        let break_even = self.player_break_even_rate_cents();
        if break_even <= 0.0 {
            f64::INFINITY
        } else {
            self.player.rate_cents / break_even
        }
    }

    pub fn player_last_interest_coverage(&self) -> Option<f64> {
        self.last_report
            .as_ref()
            .map(|report| interest_coverage(report.profit, report.interest))
    }

    pub fn acquisition_price(&self, competitor_index: usize) -> f64 {
        self.acquisition_terms(competitor_index)
            .expect("competitor index out of range")
            .price
    }

    pub fn acquisition_stress_score(&self, competitor_index: usize) -> Option<f64> {
        let competitor = self.competitors.get(competitor_index)?;
        let had_diligence = self.has_diligence(competitor_index);
        let terms = if had_diligence {
            self.acquisition_terms(competitor_index)
        } else {
            self.public_acquisition_estimate(competitor_index)
        }?;
        Some(acquisition_covenant_stress_score(
            competitor,
            &terms,
            self.player.customers,
            self.integration_strain,
            !had_diligence,
        ))
    }

    pub fn acquisition_stress_label(score: f64) -> &'static str {
        if score >= ACQUISITION_STRESS_DISTRESSED {
            "distressed"
        } else if score >= ACQUISITION_STRESS_STRAINED {
            "strained"
        } else if score >= ACQUISITION_STRESS_NOTICE {
            "guarded"
        } else {
            "ordinary"
        }
    }

    pub fn public_acquisition_estimate(&self, competitor_index: usize) -> Option<AcquisitionTerms> {
        let competitor = self.competitors.get(competitor_index)?;
        let mut public_competitor = competitor.clone();
        public_competitor.cash = competitor.public_cash_estimate();
        public_competitor.debt = competitor.public_debt_estimate();
        self.acquisition_terms_for_competitor(&public_competitor)
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

    fn competitor_under_active_diligence(&self, competitor_index: usize) -> bool {
        let Some(competitor) = self.competitors.get(competitor_index) else {
            return false;
        };

        self.diligence_reports.iter().any(|report| {
            report.quarters_remaining > 0
                && report.target_alerted
                && report.competitor_name == competitor.name
        })
    }

    fn diligence_alert_leaks(&mut self, competitor_index: usize) -> bool {
        let probability = self.diligence_alert_probability(competitor_index);
        self.rng.chance(probability)
    }

    fn diligence_alert_probability(&self, competitor_index: usize) -> f64 {
        let Some(competitor) = self.competitors.get(competitor_index) else {
            return 0.0;
        };
        let total_connected = self.total_connected_customers().max(1.0);
        let target_share = (competitor.customers / total_connected).clamp(0.0, 1.0);
        let concentration_pressure = (self.market_share() - 0.40).max(0.0) * 0.30;
        let strategic_target_pressure = target_share * 0.30;
        let serial_deal_pressure = self.acquisition_stress.clamp(0.0, 1.0) * 0.30;
        let rate_pressure =
            ((competitor.rate_cents - self.player.rate_cents) / 4.0).clamp(0.0, 0.10);
        (DILIGENCE_ALERT_BASE_PROBABILITY
            + strategic_target_pressure
            + concentration_pressure
            + serial_deal_pressure
            + rate_pressure)
            .clamp(
                DILIGENCE_ALERT_MIN_PROBABILITY,
                DILIGENCE_ALERT_MAX_PROBABILITY,
            )
    }

    fn apply_diligence_alert_response(&mut self, competitor_index: usize) -> String {
        let total_connected = self.total_connected_customers().max(1.0);
        let player_rate = self.player.rate_cents;
        let market = self.market.clone();
        let macro_state = self.macro_state.clone();
        let cost_multiplier = self.cost_shock_multiplier();
        let rate_frozen = self.rate_frozen();

        let competitor = &mut self.competitors[competitor_index];
        let share = (competitor.customers / total_connected).clamp(0.0, 1.0);
        let response_scale =
            (0.35 + share * 0.65 + competitor.reputation / 850.0).clamp(0.35, 0.90);

        let starting_rate = competitor.rate_cents;
        let starting_asset_base = competitor.asset_base;
        let starting_debt = competitor.debt;
        let starting_reliability = competitor.reliability;
        let mut retention_spend = 0.0;
        let mut capacity_spend = 0.0;
        let mut maintenance_spend = 0.0;

        if competitor.debt_to_assets() < 0.78 {
            let backing = ((1_600.0 + competitor.asset_base * 0.009 + competitor.customers * 1.2)
                * response_scale)
                .clamp(1_800.0, DILIGENCE_ALERT_MAX_BACKING);
            competitor.cash += backing;
            competitor.debt += backing;
        }

        let marketing_budget = ((750.0 + competitor.customers * 1.25) * response_scale)
            .clamp(550.0, 2_200.0)
            .min(competitor.cash * 0.08);
        if marketing_budget > 450.0 {
            competitor.cash -= marketing_budget;
            retention_spend = marketing_budget;
            competitor.marketing_momentum =
                (competitor.marketing_momentum + marketing_budget / 10_500.0).clamp(0.0, 1.75);
            competitor.reputation =
                (competitor.reputation + marketing_budget / 6_500.0).clamp(0.0, 100.0);
        }

        let capex_budget = ((900.0 + competitor.asset_base * 0.004) * response_scale)
            .clamp(850.0, 3_200.0)
            .min(competitor.cash * 0.09);
        if capex_budget > 650.0 {
            competitor.cash -= capex_budget;
            capacity_spend = capex_budget;
            competitor.asset_base += capex_budget;
            competitor.distribution_capacity += capex_budget / 85.0;
            competitor.generation_capacity_mwh += capex_budget / 420.0;
        }

        let maintenance_budget = ((550.0 + competitor.asset_base * 0.003) * response_scale)
            .clamp(450.0, 1_500.0)
            .min(competitor.cash * 0.06);
        if competitor.reliability < 0.88 && maintenance_budget > 400.0 {
            competitor.cash -= maintenance_budget;
            maintenance_spend = maintenance_budget;
            let gain = maintenance_reliability_gain(competitor, maintenance_budget);
            competitor.reliability = (competitor.reliability + gain).clamp(0.35, 0.98);
            competitor.reputation = (competitor.reputation
                + maintenance_reputation_gain(competitor, maintenance_budget) * 0.35)
                .clamp(0.0, 100.0);
        }

        if !rate_frozen && competitor.rate_cents > player_rate + 0.15 {
            let defensive_floor =
                defensive_rate_floor(competitor, &market, &macro_state, cost_multiplier);
            let target = (player_rate + 0.20).max(defensive_floor);
            let cut = 0.15_f64.min((competitor.rate_cents - target).max(0.0));
            competitor.rate_cents -= cut;
        }

        let raised_backing = competitor.debt - starting_debt;
        let capex = competitor.asset_base - starting_asset_base;
        let rate_cut = starting_rate - competitor.rate_cents;
        let reliability_gain = competitor.reliability - starting_reliability;

        let mut actions = Vec::new();
        if raised_backing > 100.0 {
            actions.push(format!(
                "raised {} in defensive financing",
                money(raised_backing)
            ));
        }
        if retention_spend > 100.0 {
            actions.push(format!(
                "spent {} on customer retention",
                money(retention_spend)
            ));
        }
        if capacity_spend > 100.0 {
            actions.push(format!(
                "invested {} in service capacity",
                money(capacity_spend)
            ));
        }
        if maintenance_spend > 100.0 {
            actions.push(format!("spent {} on maintenance", money(maintenance_spend)));
        }
        if rate_cut > 0.01 {
            actions.push(format!("cut rates {:.1}c", rate_cut));
        }
        if reliability_gain >= MIN_MAINTENANCE_RELIABILITY_GAIN {
            actions.push(format!(
                "improved service quality {:.1} pts",
                reliability_gain * 100.0
            ));
        }
        if actions.is_empty() && capex > 100.0 {
            actions.push(format!(
                "put {} into its operating base",
                money(capex.max(0.0))
            ));
        }

        if actions.is_empty() {
            format!(
                "{} picked up signs of the review, but made no visible operating move.",
                competitor.name
            )
        } else {
            format!(
                "{} picked up signs of the review and responded: {}.",
                competitor.name,
                actions.join("; ")
            )
        }
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
        self.acquisition_terms_for_competitor(competitor)
    }

    fn acquisition_terms_for_competitor(&self, competitor: &Utility) -> Option<AcquisitionTerms> {
        let acquired_customers = competitor.customers * 0.96;
        let acquired_generation_capacity_mwh = competitor.generation_capacity_mwh * 0.90;
        let acquired_distribution_capacity = competitor.distribution_capacity * 0.94;
        let acquired_asset_base = competitor.asset_base * 0.76;
        let customer_value = competitor.customers
            * (ACQUISITION_CUSTOMER_BASE_VALUE
                + competitor.reputation * ACQUISITION_REPUTATION_CUSTOMER_VALUE);
        let generation_value = competitor.generation_capacity_mwh * 36.0;
        let line_value = competitor.distribution_capacity * 11.0;
        let premium = 8_000.0 + competitor.reliability * 4_000.0;
        let share = self.market_share();
        let debt_load = competitor.debt_to_assets();
        let distress_discount = ((debt_load - 0.55).max(0.0) * 0.20)
            .clamp(0.0, ACQUISITION_DISTRESSED_PREMIUM_DISCOUNT_MAX);
        let consolidation_premium =
            acquisition_consolidation_premium(share) * (1.0 - distress_discount);
        let enterprise_value =
            (customer_value + generation_value + line_value + premium) * consolidation_premium;
        let total_connected = self.total_connected_customers().max(1.0);
        let post_player_customers = self.player.customers + acquired_customers;
        let post_total_connected = (total_connected - competitor.customers + acquired_customers)
            .max(post_player_customers)
            .max(1.0);
        let post_market_share = post_player_customers / post_total_connected;
        let public_interest_concession = if post_market_share > 0.50 {
            let excess_share = post_market_share - 0.50;
            let dominance_intensity = (excess_share
                / PUBLIC_INTEREST_CONCESSION_FULL_CONTROL_EXCESS)
                .clamp(0.0, 1.0)
                .powf(PUBLIC_INTEREST_CONCESSION_EXPONENT);
            enterprise_value * dominance_intensity * PUBLIC_INTEREST_CONCESSION_EV_MULTIPLIER
                + acquired_customers * 18.0 * dominance_intensity
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
        if self.sandbox_mode
            || self.quarter != BOARD_CHECKPOINT_QUARTER
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
        let cost_multiplier = self.cost_shock_multiplier();
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
                    ProjectKind::AdjacentTerritory {
                        addressable_customers,
                        initial_customers,
                        incumbent_customers,
                    } => {
                        let old_serviceable = self.market.serviceable_customers();
                        self.market.addressable_customers += addressable_customers;
                        let added_serviceable = addressable_customers * 0.34;
                        self.market.electrification = ((old_serviceable + added_serviceable)
                            / self.market.addressable_customers.max(1.0))
                        .clamp(0.05, 0.68);

                        let added_generation =
                            initial_customers * self.market.avg_mwh_per_customer * 1.45;
                        let old_reliability = self.player.reliability;
                        self.player.generation_capacity_mwh += added_generation;
                        self.player.distribution_capacity += initial_customers * 1.55;
                        self.player.customers += initial_customers;
                        self.player.last_quarter_customers = self.player.customers;
                        self.player.reliability = generation_reliability_after_new_capacity(
                            old_reliability,
                            self.player.generation_capacity_mwh - added_generation,
                            added_generation,
                        );
                        self.player.reputation = (self.player.reputation + 2.0).clamp(0.0, 100.0);

                        self.startup_index += 1;
                        let mut used_names = vec![self.player.name.clone()];
                        used_names.extend(
                            self.competitors
                                .iter()
                                .map(|competitor| competitor.name.clone()),
                        );
                        let incumbent_name =
                            draw_competitor_name(&mut self.rng, &used_names, self.startup_index);
                        let incumbent_rate_target =
                            self.market.standard_rate_cents - 0.1 + self.rng.range(-0.35, 0.45);
                        let incumbent_generation = incumbent_customers
                            * self.market.avg_mwh_per_customer
                            * self.rng.range(1.18, 1.42);
                        let (public_cash_multiplier, public_debt_multiplier) =
                            public_balance_sheet_multipliers(&mut self.rng);
                        let mut incumbent = Utility {
                            name: incumbent_name.clone(),
                            cash: 24_000.0 + self.rng.range(0.0, 8_000.0),
                            debt: 18_000.0 + self.rng.range(0.0, 8_000.0),
                            shares: 0.0,
                            stock_price: 0.0,
                            customers: incumbent_customers,
                            generation_capacity_mwh: incumbent_generation,
                            distribution_capacity: incumbent_customers * self.rng.range(1.20, 1.40),
                            rate_cents: incumbent_rate_target.max(0.25),
                            reputation: 55.0 + self.rng.range(-4.0, 6.0),
                            reliability: 0.80 + self.rng.range(-0.035, 0.055),
                            marketing_momentum: 0.22,
                            asset_base: 95_000.0 + self.rng.range(0.0, 28_000.0),
                            last_quarter_customers: incumbent_customers,
                            public_cash_multiplier,
                            public_debt_multiplier,
                        };
                        let defensive_floor = defensive_rate_floor(
                            &incumbent,
                            &self.market,
                            &self.macro_state,
                            cost_multiplier,
                        );
                        let public_ceiling = public_rate_tolerance(&self.market)
                            .min(self.market.standard_rate_cents + 2.0);
                        incumbent.rate_cents = bounded_rate_target(
                            incumbent_rate_target,
                            defensive_floor,
                            public_ceiling,
                        );
                        self.competitors.push(incumbent);
                        self.adjacent_expansions += 1;
                        self.regional_integration = (self.regional_integration
                            + adjacent_expansion_integration_burden(self.adjacent_expansions))
                        .clamp(0.0, 1.20);
                        events.push(format!(
                            "The {} opened: Metro connected {:.0} launch customers, the addressable market grew by {:.0}, and {incumbent_name} emerged as the local incumbent.",
                            project.name, initial_customers, addressable_customers
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

    fn reduce_regional_integration(&mut self, relief: f64, source: &str) -> String {
        if self.regional_integration <= 0.01 || relief <= 0.0 {
            return String::new();
        }

        let before = self.regional_integration;
        self.regional_integration = (self.regional_integration - relief).max(0.0);
        let actual = before - self.regional_integration;
        if actual < 0.01 {
            String::new()
        } else {
            format!(
                " Regional integration burden eased by {:.0} points through {source}.",
                actual * 100.0
            )
        }
    }

    fn apply_regional_integration_strain(&mut self, events: &mut Vec<String>) {
        if self.regional_integration <= 0.01 {
            self.regional_integration = 0.0;
            return;
        }

        let reliability_drag = (0.002 + self.regional_integration * 0.006).min(0.014);
        let reputation_drag = (0.25 + self.regional_integration * 1.10).min(2.0);
        let marketing_drag = (1.0 - self.regional_integration * 0.035).clamp(0.88, 1.0);
        self.player.reliability = (self.player.reliability - reliability_drag).clamp(0.35, 0.98);
        self.player.reputation = (self.player.reputation - reputation_drag).clamp(0.0, 100.0);
        self.player.marketing_momentum *= marketing_drag;

        if self.regional_integration >= 0.18 {
            events.push(format!(
                "Regional expansion is absorbing management attention; service and reputation slipped by {:.1} points.",
                reputation_drag
            ));
        }

        self.regional_integration *= 0.82;
        if self.regional_integration < 0.04 {
            self.regional_integration = 0.0;
        }
    }

    fn apply_operating_managers(&mut self, events: &mut Vec<String>) -> f64 {
        self.apply_maintenance_manager(events) + self.apply_marketing_manager(events)
    }

    fn manager_budget(&self, profit_share: f64) -> f64 {
        let Some(report) = &self.last_report else {
            return 0.0;
        };
        (report.profit.max(0.0) * profit_share).min(self.player.cash.max(0.0))
    }

    fn apply_maintenance_manager(&mut self, events: &mut Vec<String>) -> f64 {
        let Some(target) = self.maintenance_manager_target else {
            return 0.0;
        };
        if self.player.reliability >= target - MIN_MAINTENANCE_RELIABILITY_GAIN {
            return 0.0;
        }

        let budget = self.manager_budget(MAINTENANCE_MANAGER_PROFIT_SHARE);
        if budget < MIN_MANAGER_SPEND {
            return 0.0;
        }
        let required = maintenance_spend_for_reliability_target(&self.player, target);
        let spend = required.min(budget).min(self.player.cash.max(0.0));
        if spend < MIN_MANAGER_SPEND {
            return 0.0;
        }

        let gain = maintenance_reliability_gain(&self.player, spend);
        let new_reliability = (self.player.reliability + gain).clamp(0.35, MAX_RELIABILITY);
        let actual_gain = new_reliability - self.player.reliability;
        if actual_gain < MIN_MAINTENANCE_RELIABILITY_GAIN {
            return 0.0;
        }

        self.player.cash -= spend;
        self.player.reliability = new_reliability;
        self.player.reputation = (self.player.reputation
            + maintenance_reputation_gain(&self.player, spend))
        .clamp(0.0, 100.0);
        let regional_relief = self.reduce_regional_integration(spend / 140_000.0, "service work");
        events.push(format!(
            "Maintenance manager spent {} toward the {:.0}% reliability target; reliability rose by {:.1} points to {:.0}%.",
            money(spend),
            target * 100.0,
            actual_gain * 100.0,
            self.player.reliability * 100.0
        ) + &regional_relief);
        spend
    }

    fn apply_marketing_manager(&mut self, events: &mut Vec<String>) -> f64 {
        let Some(target) = self.marketing_manager_target else {
            return 0.0;
        };
        if self.player.reputation >= target - 0.05 {
            return 0.0;
        }

        let budget = self.manager_budget(MARKETING_MANAGER_PROFIT_SHARE);
        if budget < MIN_MANAGER_SPEND {
            return 0.0;
        }
        let needed = ((target - self.player.reputation).max(0.0) * 4_000.0).max(0.0);
        let spend = needed.min(budget).min(self.player.cash.max(0.0));
        if spend < MIN_MANAGER_SPEND {
            return 0.0;
        }

        self.player.cash -= spend;
        self.player.marketing_momentum += (spend / 5_000.0) * 0.13;
        self.player.reputation = (self.player.reputation + spend / 4_000.0).clamp(0.0, 100.0);
        let regional_relief = self.reduce_regional_integration(spend / 160_000.0, "marketing");
        events.push(format!(
            "Marketing manager spent {} toward the {:.0} reputation target; reputation is now {:.0}.",
            money(spend),
            target,
            self.player.reputation
        ) + &regional_relief);
        spend
    }

    fn apply_acquisition_stress_pressure(
        &mut self,
        finances: &FirmFinances,
        events: &mut Vec<String>,
    ) {
        if self.acquisition_stress < ACQUISITION_STRESS_NOTICE {
            return;
        }

        let leverage = self.player.debt_to_assets();
        let weak_financing = leverage > 0.86 || finances.profit < 0.0 || self.player.cash < 8_000.0;
        let fatigue = (self.acquisition_stress * 0.055).min(0.11);
        self.equity_market_fatigue = (self.equity_market_fatigue + fatigue).clamp(0.0, 3.0);
        self.player.reputation =
            (self.player.reputation - self.acquisition_stress * 0.42).clamp(0.0, 100.0);

        if self.acquisition_stress >= ACQUISITION_STRESS_DISTRESSED {
            let oversight_cost =
                (self.player.asset_base.max(1.0) * self.acquisition_stress * 0.0025)
                    .clamp(1_200.0, 9_000.0);
            self.player.cash -= oversight_cost;
            events.push(format!(
                "Acquisition stress forced lender oversight and integration controls; {} was diverted from cash and borrowing room narrowed.",
                money(oversight_cost)
            ));
        } else if self.acquisition_stress >= ACQUISITION_STRESS_STRAINED && weak_financing {
            let amendment_cost = (self.player.debt.max(1.0) * self.acquisition_stress * 0.0045)
                .clamp(850.0, 14_000.0);
            self.player.cash -= amendment_cost;
            events.push(format!(
                "Lenders tightened acquisition covenants after reviewing integration progress; {} of fees hit cash and financing flexibility narrowed.",
                money(amendment_cost)
            ));
        } else if self.acquisition_stress >= ACQUISITION_STRESS_STRAINED {
            events.push(
                "Lenders are watching the acquisition integration closely; future financing is less forgiving."
                    .to_string(),
            );
        }
    }

    fn decay_acquisition_stress(&mut self, finances: &FirmFinances) {
        if self.acquisition_stress <= 0.01 {
            self.acquisition_stress = 0.0;
            return;
        }

        let leverage = self.player.debt_to_assets();
        let decay = if finances.profit < 0.0 || leverage > 0.90 || self.player.cash < 0.0 {
            0.90
        } else if self.acquisition_stress >= ACQUISITION_STRESS_STRAINED {
            0.78
        } else {
            0.64
        };
        self.acquisition_stress *= decay;
        if self.acquisition_stress < 0.04 {
            self.acquisition_stress = 0.0;
        }
    }

    fn apply_merger_execution_outcome(
        &mut self,
        acquired: &Utility,
        terms: &AcquisitionTerms,
        starting_customers: f64,
        pre_close_integration_strain: f64,
    ) -> String {
        let risk = merger_execution_risk(
            self,
            acquired,
            terms,
            starting_customers,
            pre_close_integration_strain,
        );
        if !self.rng.chance(risk) {
            return String::new();
        }

        let deal_share =
            terms.acquired_customers / (starting_customers + terms.acquired_customers).max(1.0);
        let severity = self.rng.range(0.55, 1.15) * (0.78 + deal_share);
        let customer_loss_rate = (self.rng.range(0.045, 0.13) * severity).clamp(0.025, 0.22);
        let customer_loss = (terms.acquired_customers * customer_loss_rate)
            .min((self.player.customers - 1.0).max(0.0));
        let capacity_loss_rate = (self.rng.range(0.020, 0.060) * severity).clamp(0.010, 0.12);
        let generation_loss = terms.acquired_generation_capacity_mwh * capacity_loss_rate;
        let distribution_loss = terms.acquired_distribution_capacity * capacity_loss_rate * 1.15;
        let overrun =
            (terms.price * self.rng.range(0.018, 0.055) * severity).clamp(1_250.0, 22_000.0);

        self.player.customers = (self.player.customers - customer_loss).max(1.0);
        self.player.generation_capacity_mwh =
            (self.player.generation_capacity_mwh - generation_loss).max(20.0);
        self.player.distribution_capacity =
            (self.player.distribution_capacity - distribution_loss).max(20.0);
        self.player.cash -= overrun;
        self.player.reliability =
            (self.player.reliability - (0.014 + severity * 0.018).min(0.055)).clamp(0.35, 0.98);
        self.player.reputation =
            (self.player.reputation - (2.0 + severity * 4.6).min(9.0)).clamp(0.0, 100.0);
        self.integration_strain = (self.integration_strain + severity * 0.18).clamp(0.0, 1.8);

        format!(
            " Integration broke badly: {:.0} accounts defected, {} of overruns hit cash, and service quality suffered.",
            customer_loss,
            money(overrun)
        )
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

    fn apply_below_cost_pricing_pressure(
        &mut self,
        finances: &FirmFinances,
        events: &mut Vec<String>,
    ) {
        let break_even = self.player_break_even_rate_cents();
        if break_even <= 0.0 {
            return;
        }

        let support_ratio = self.player.rate_cents / break_even;
        if support_ratio >= BELOW_COST_PRESSURE_RATE_SUPPORT_RATIO {
            return;
        }

        let severity = (BELOW_COST_PRESSURE_RATE_SUPPORT_RATIO - support_ratio).clamp(0.0, 0.75);
        self.equity_market_fatigue = (self.equity_market_fatigue + severity * 0.36).clamp(0.0, 3.0);
        self.player.reputation = (self.player.reputation - severity * 3.8).clamp(0.0, 100.0);

        if finances.profit < 0.0 && support_ratio < 0.72 {
            let stress_debt = (-finances.profit * severity * 0.18).clamp(0.0, 18_000.0);
            self.player.debt += stress_debt;
        }

        if support_ratio < 0.82 || finances.profit < -1_000.0 {
            events.push(format!(
                "Below-cost pricing strained financing: rate {:.1}c versus {:.1}c break-even.",
                self.player.rate_cents, break_even
            ));
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

fn project_clamp_note(requested: f64, actual: f64, min: f64, max: f64, unit: &str) -> String {
    if (requested - actual).abs() < 0.01 {
        return String::new();
    }

    format!(
        " Requested {:.0} {} was clamped to the buildable range of {:.0}-{:.0} {}.",
        requested, unit, min, max, unit
    )
}

fn validate_loaded_utility(
    label: &str,
    utility: &Utility,
    require_public_shares: bool,
) -> Result<(), String> {
    let checks = [
        ("cash", utility.cash, -1_000_000.0),
        ("debt", utility.debt, 0.0),
        ("customers", utility.customers, 0.0),
        ("generation capacity", utility.generation_capacity_mwh, 0.0),
        ("distribution capacity", utility.distribution_capacity, 0.0),
        ("asset base", utility.asset_base, 0.0),
        (
            "last quarter customers",
            utility.last_quarter_customers,
            0.0,
        ),
    ];
    for (field, value, minimum) in checks {
        if !value.is_finite() || value < minimum {
            return Err(format!(
                "{label} {field} must be finite and at least {minimum:.0}"
            ));
        }
    }
    if !(0.0..=100.0).contains(&utility.reputation) {
        return Err(format!("{label} reputation must be between 0 and 100"));
    }
    if !(0.0..=1.0).contains(&utility.reliability) {
        return Err(format!("{label} reliability must be between 0 and 1"));
    }
    if utility.rate_cents <= 0.0 || !utility.rate_cents.is_finite() {
        return Err(format!("{label} rate must be positive and finite"));
    }
    if require_public_shares && (!utility.shares.is_finite() || utility.shares <= 0.0) {
        return Err(format!("{label} shares must be positive and finite"));
    }
    if !utility.stock_price.is_finite() || utility.stock_price < 0.0 {
        return Err(format!(
            "{label} stock price must be finite and nonnegative"
        ));
    }
    Ok(())
}

fn validate_maintenance_manager_target(target: f64) -> Result<f64, String> {
    if !target.is_finite() {
        return Err("Maintenance manager target must be finite.".to_string());
    }
    if !(0.35..=MAX_RELIABILITY).contains(&target) {
        return Err(format!(
            "Maintenance manager target must be between 35% and {:.0}%.",
            MAX_RELIABILITY * 100.0
        ));
    }
    Ok(target)
}

fn validate_marketing_manager_target(target: f64) -> Result<f64, String> {
    if !target.is_finite() {
        return Err("Marketing manager target must be finite.".to_string());
    }
    if !(0.0..=100.0).contains(&target) {
        return Err("Marketing manager target must be between 0 and 100.".to_string());
    }
    Ok(target)
}

fn merger_execution_risk(
    game: &Game,
    acquired: &Utility,
    terms: &AcquisitionTerms,
    starting_customers: f64,
    pre_close_integration_strain: f64,
) -> f64 {
    let deal_share =
        terms.acquired_customers / (starting_customers + terms.acquired_customers).max(1.0);
    let target_service_risk = (0.84 - acquired.reliability).max(0.0) * 0.70;
    let target_reputation_risk = (58.0 - acquired.reputation).max(0.0) / 320.0;
    let leverage_risk = (game.player.debt_to_assets() - 0.68).max(0.0) * 0.34;
    let serial_rollup_risk = pre_close_integration_strain.max(0.0) * 0.14;
    let dominance_risk = (game.market_share() - 0.42).max(0.0) * 0.12;

    (0.035
        + deal_share * 0.29
        + target_service_risk
        + target_reputation_risk
        + leverage_risk
        + serial_rollup_risk
        + dominance_risk)
        .clamp(MERGER_RISK_MIN, MERGER_RISK_MAX)
}

fn acquisition_covenant_stress_score(
    acquired: &Utility,
    terms: &AcquisitionTerms,
    starting_customers: f64,
    pre_close_integration_strain: f64,
    no_diligence: bool,
) -> f64 {
    let deal_share =
        terms.acquired_customers / (starting_customers + terms.acquired_customers).max(1.0);
    let service_risk = (0.82 - acquired.reliability).max(0.0) * 1.18;
    let reputation_risk = (55.0 - acquired.reputation).max(0.0) / 150.0;
    let leverage_risk = (terms.post_debt_to_assets - 0.78).max(0.0) * 1.55;
    let cash_buffer_risk = ((10_000.0 - terms.post_cash) / 16_000.0).clamp(0.0, 1.0) * 0.18;
    let strain_risk = pre_close_integration_strain.max(0.0) * 0.20;
    let dominance_risk = (terms.post_market_share - 0.52).max(0.0) * 0.20;
    let hidden_close_risk = if no_diligence { 0.05 } else { 0.0 };

    (deal_share * 0.36
        + service_risk
        + reputation_risk
        + leverage_risk
        + cash_buffer_risk
        + strain_risk
        + dominance_risk
        + hidden_close_risk)
        .clamp(0.0, 1.20)
}

fn acquisition_financing_suffix(score: f64) -> String {
    if score >= ACQUISITION_STRESS_DISTRESSED {
        " Lenders view the close as distressed; losses could force a financing reckoning."
            .to_string()
    } else if score >= ACQUISITION_STRESS_STRAINED {
        " Lenders view the close as strained; patience is thinner until integration proves out."
            .to_string()
    } else if score >= ACQUISITION_STRESS_NOTICE {
        " Financing signals are guarded until the integration plan proves out.".to_string()
    } else {
        String::new()
    }
}

fn adjacent_expansion_integration_burden(completed_expansions: u32) -> f64 {
    (0.20 + f64::from(completed_expansions.saturating_sub(1)) * 0.08).clamp(0.20, 0.42)
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

fn demanded_mwh_from_finances(finances: &FirmFinances) -> f64 {
    if finances.unmet_demand_ratio >= 0.999 {
        finances.served_mwh
    } else {
        finances.served_mwh / (1.0 - finances.unmet_demand_ratio).max(0.001)
    }
}

fn unmet_mwh_from_finances(finances: &FirmFinances) -> f64 {
    demanded_mwh_from_finances(finances) * finances.unmet_demand_ratio
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
        let leverage_premium = (leverage - 0.35).max(0.0).powf(1.25) * 0.24;
        let distress_premium = (leverage - 0.75).max(0.0) * 0.60;
        (self.benchmark_credit_rate() + leverage_premium + distress_premium).clamp(0.04, 0.36)
    }

    pub fn borrowing_limit_ratio(&self) -> f64 {
        let rate_stress = (self.annual_base_rate - BASE_ANNUAL_RATE).max(0.0) * 2.0;
        let spread_stress = (self.credit_spread - BASE_CREDIT_SPREAD).max(0.0) * 3.0;
        (0.88 - rate_stress - spread_stress).clamp(0.54, 0.90)
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

    pub fn public_cash_estimate(&self) -> f64 {
        self.cash * self.public_cash_multiplier.max(0.0)
    }

    pub fn public_debt_estimate(&self) -> f64 {
        self.debt * self.public_debt_multiplier.max(0.0)
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
            CustomerAllocationKind::NewConnections => (0.22, 0.70, 2.05, 82.0, 0.88, 0.20, 0.39),
            CustomerAllocationKind::SwitchedAccounts => (0.33, 0.92, 2.65, 102.0, 0.60, 0.26, 0.50),
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
    let (public_cash_multiplier, public_debt_multiplier) = public_balance_sheet_multipliers(rng);

    Utility {
        name: name.to_string(),
        cash: varied_pct(rng, cash, 0.18, initial_variance).max(1_500.0),
        debt: varied_debt,
        shares,
        stock_price: if stock_price == 0.0 {
            0.0
        } else {
            varied_pct(rng, stock_price, 0.12, initial_variance)
                .max(stock_price.min(MIN_STOCK_PRICE))
        },
        customers: varied_customers,
        generation_capacity_mwh: varied_pct(rng, generation_capacity_mwh, 0.10, initial_variance)
            .max(20.0),
        distribution_capacity: varied_pct(rng, distribution_capacity, 0.12, initial_variance)
            .max(50.0),
        rate_cents: varied_abs(rng, rate_cents, 0.35, initial_variance)
            .clamp(MIN_OPENING_RATE_CENTS, MAX_OPENING_RATE_CENTS),
        reputation: varied_abs(rng, reputation, 6.0, initial_variance).clamp(20.0, 85.0),
        reliability: varied_abs(rng, reliability, 0.045, initial_variance).clamp(0.55, 0.94),
        marketing_momentum: varied_abs(rng, marketing_momentum, 0.025, initial_variance)
            .clamp(0.0, 0.18),
        asset_base: varied_asset_base,
        last_quarter_customers: varied_customers,
        public_cash_multiplier,
        public_debt_multiplier,
    }
}

pub(super) fn public_balance_sheet_multipliers(rng: &mut Rng) -> (f64, f64) {
    if !rng.chance(PUBLIC_BALANCE_SHEET_DIVERGENCE_PROBABILITY) {
        return (1.0, 1.0);
    }

    (
        1.0 + rng.range(
            -PUBLIC_BALANCE_SHEET_MAX_DIVERGENCE,
            PUBLIC_BALANCE_SHEET_MAX_DIVERGENCE,
        ),
        1.0 + rng.range(
            -PUBLIC_BALANCE_SHEET_MAX_DIVERGENCE,
            PUBLIC_BALANCE_SHEET_MAX_DIVERGENCE,
        ),
    )
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
