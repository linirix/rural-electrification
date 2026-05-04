#[derive(Clone, Debug, Default)]
pub struct StrategySummary {
    pub runs: u32,
    pub victories: u32,
    pub board_defeats: u32,
    pub receivership_defeats: u32,
    pub market_access_defeats: u32,
    pub board_defeat_quarters: f64,
    pub receivership_defeat_quarters: f64,
    pub market_access_defeat_quarters: f64,
    pub defeat_quarters: f64,
    pub defeat_quarter_range: Range,
    pub start_share: f64,
    pub start_cash: f64,
    pub start_debt: f64,
    pub start_customers: f64,
    pub start_reliability: f64,
    pub start_rate: f64,
    pub start_headroom: f64,
    pub start_share_range: Range,
    pub start_cash_range: Range,
    pub start_debt_range: Range,
    pub start_customers_range: Range,
    pub start_reliability_range: Range,
    pub start_rate_range: Range,
    pub start_headroom_range: Range,
    pub finish_share: f64,
    pub finish_share_range: Range,
    pub finish_share_histogram: ShareHistogram,
    pub finish_cash: f64,
    pub finish_debt: f64,
    pub finish_customers: f64,
    pub finish_reliability: f64,
    pub finish_wealth: f64,
    pub finish_dividends: f64,
    pub finish_ownership: f64,
    pub peak_acquisition_stress: f64,
    pub peak_acquisition_stress_range: Range,
    pub acquisition_stress_defeats: u32,
    pub acquisition_stress_receiverships: u32,
}

impl StrategySummary {
    pub fn win_rate(&self) -> f64 {
        if self.runs == 0 {
            0.0
        } else {
            self.victories as f64 / self.runs as f64
        }
    }

    pub fn defeats(&self) -> u32 {
        self.board_defeats + self.receivership_defeats + self.market_access_defeats
    }

    pub fn average_finish_share(&self) -> f64 {
        if self.runs == 0 {
            0.0
        } else {
            self.finish_share / self.runs as f64
        }
    }

    pub fn average_peak_acquisition_stress(&self) -> f64 {
        if self.runs == 0 {
            0.0
        } else {
            self.peak_acquisition_stress / self.runs as f64
        }
    }

    pub fn average_finish_wealth(&self) -> f64 {
        if self.runs == 0 {
            0.0
        } else {
            self.finish_wealth / self.runs as f64
        }
    }

    pub fn average_finish_ownership(&self) -> f64 {
        if self.runs == 0 {
            0.0
        } else {
            self.finish_ownership / self.runs as f64
        }
    }

    pub fn average_finish_dividends(&self) -> f64 {
        if self.runs == 0 {
            0.0
        } else {
            self.finish_dividends / self.runs as f64
        }
    }

    pub(super) fn record_defeat(&mut self, headline: &str, quarter: u32) {
        let quarter = quarter as f64;
        self.defeat_quarters += quarter;
        self.defeat_quarter_range.observe(quarter);
        if headline.contains("Market Access") {
            self.market_access_defeats += 1;
            self.market_access_defeat_quarters += quarter;
        } else if headline.contains("Receivership") {
            self.receivership_defeats += 1;
            self.receivership_defeat_quarters += quarter;
        } else {
            self.board_defeats += 1;
            self.board_defeat_quarters += quarter;
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ShareHistogram {
    bins: [u32; 7],
}

impl ShareHistogram {
    pub(super) fn observe(&mut self, share: f64) {
        let pct = share * 100.0;
        let index = if pct < 35.0 {
            0
        } else if pct < 40.0 {
            1
        } else if pct < 45.0 {
            2
        } else if pct < 50.0 {
            3
        } else if pct < 55.0 {
            4
        } else if pct < 60.0 {
            5
        } else {
            6
        };
        self.bins[index] += 1;
    }

    pub fn render(&self) -> String {
        let labels = [
            "<35%", "35-40%", "40-45%", "45-50%", "50-55%", "55-60%", "60%+",
        ];
        let mut parts = Vec::with_capacity(labels.len());
        for (label, count) in labels.iter().zip(self.bins.iter()) {
            parts.push(format!("{label}: {count}"));
        }
        parts.join(" | ")
    }
}

#[derive(Clone, Debug)]
pub struct Range {
    pub min: f64,
    pub max: f64,
}

impl Range {
    pub(super) fn observe(&mut self, value: f64) {
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }
}

impl Default for Range {
    fn default() -> Self {
        Self {
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
        }
    }
}
