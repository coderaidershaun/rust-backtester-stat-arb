use crate::utils::{cumulative_returns, normalise_returns};
use ndarray::arr1;


pub struct Backtest {
    weight_asset_1: f64, // Capital percentage on asset 1 between 0 and 1.0
    weight_asset_2: f64, // Capital percentage on asset 2 between 0 and 1.0
    signals: Option<Vec<f64>>
}

impl Backtest {
    pub fn new(weight_asset_1: f64, weight_asset_2: f64) -> Self {
        Self { weight_asset_1, weight_asset_2, signals: None }
    }

    /// Construct Portfolio Returns
    /// Takes in log returns and computes portfolio returns as such:
    /// Asset_1: log_returns * signal (long, short neutral) * (sign as +1.0) * capital_weighting
    /// Asset_2: log_returns * signal (long, short neutral) * inverse (sign as -1.0) * capital_weighting
    /// The inverse is used for asset_2 as the original signal was constructed for asset 1. Asset 2 is just the other side
    fn construct_portfolio_returns(&self, log_rets: Vec<f64>, sign: f64,  weight: f64) -> Result<(), String> {

        if let None = self.signals {
            return Err("Please generate and consolidate signals before running this function".to_string());
        }

        // Get strategy returns
        let rets_arr = arr1(&log_rets);
        let sig_arr = arr1(self.signals.as_ref().unwrap());
        let strat_log_rets = rets_arr * sig_arr * sign * weight;

        // Get Cumulative returns
        let strat_cum_log_rets: Vec<f64> = cumulative_returns(&strat_log_rets.to_vec());

        // Normalise returns
        let strat_cum_norm_rets: Vec<f64> = normalise_returns(&strat_cum_log_rets);

        Ok(())
    }

    /// Run Backtest
    pub fn run_backtest(&self, log_rets_1: Vec<f64>, log_rets_2: Vec<f64>) -> Result<(), String> {

        if let None = self.signals {
            return Err("Please generate and consolidate signals before running a backtest".to_string());
        }

        self.construct_portfolio_returns(log_rets_1, 1.0, self.weight_asset_1);
        self.construct_portfolio_returns(log_rets_2, -1.0, self.weight_asset_2);

        Ok(())
    }

}

/// Trade Counts
/// Provides number of open and closed trades based upon signals
fn trade_counts(signals: &Vec<f64>) {
    let mut long_opens: u32 = 0;
    let mut long_closes: u32 = 0;
    let mut short_opens: u32 = 0;
    let mut short_closes: u32 = 0;

    // All positions are positive and used for cost placing
    let mut positions: Vec<f64> = vec![0.0]; 
    for i in 1..signals.len() - 1 {
        let val: f64 = signals[i];
        let prev_val: f64 = signals[i - 1];

        // Position was opened
        if val == 1.0 && prev_val == 0.0 {
            positions.push(1.0);
            long_opens += 1;
        } else if val == -1.0 && prev_val == 0.0 {
            positions.push(1.0);
            short_opens += 1;
        } else if val == 0.0 && prev_val == 1.0 { 
            positions.push(1.0);
            long_closes += 1; 
        } else if val == 0.0 && prev_val == -1.0 {
            positions.push(1.0);
            short_closes += 1;
        } else if val == 1.0 && prev_val == -1.0 {
            positions.push(1.0);
            short_closes += 1;
            long_opens += 1;
        } else if val == -1.0 && prev_val == 1.0 { 
            positions.push(1.0);
            long_closes += 1;
            short_opens += 1;
        } else {
            positions.push(0.0);
        }
    }
}






#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_data;
    use crate::models::{Signals, SignalType};
    use tradestats::metrics::{spread_standard, rolling_zscore};

    #[test]
    fn tests_equity_curve() {
        let (series_1, series_2) = get_test_data();
        let log_rets_1: Vec<f64> = tradestats::utils::log_returns(&series_1, true);
        let log_rets_2: Vec<f64> = tradestats::utils::log_returns(&series_2, true);
  
        let spread: Vec<f64> = spread_standard(&series_1, &series_2).unwrap();
        let roll_zscore: Vec<f64> = rolling_zscore(&spread, 21).unwrap();

        let weighting_asset_1: f64 = 1.0; // Amount of capital to assign to asset 1
        let weighting_asset_2: f64 = 1.0; // Amount of capital to assign to asset 2

        // Extract Long Signals (long asset 1, short asset 2)
        let json_long_str: &str = r#"{
            "eq": [-1.5, 0.0],
            "neq": [null, null],
            "gt": [null, 0.0],
            "lt": [-1.5, null],
            "signal_type": "Long"
        }"#;

        let params: Signals = serde_json::from_str(&json_long_str).unwrap();
        let signals_obj: Signals = Signals::new(params.eq, params.neq, params.gt, params.lt, params.signal_type);
        let long_signals: Vec<f64> = signals_obj.generate_signals(&roll_zscore);

        // Extract Short Signals (short asset 1, long asset 2)
        let json_short_str: &str = r#"{
            "eq": [1.5, 0.0],
            "neq": [null, null],
            "gt": [1.5, null],
            "lt": [null, 0.0],
            "signal_type": "Short"
        }"#;
        
        let params: Signals = serde_json::from_str(&json_short_str).unwrap();
        let signals_obj: Signals = Signals::new(params.eq, params.neq, params.gt, params.lt, params.signal_type);
        let short_signals: Vec<f64> = signals_obj.generate_signals(&roll_zscore);

        // Consolidate signals
        let net_signals: Vec<f64> = signals_obj.consolidate_signals(vec![long_signals, short_signals]);
        
        // run_backtest(log_rets_1, log_rets_2, signals_main, weighting_asset_1, weighting_asset_2);
    }
}
