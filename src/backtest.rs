use crate::evaluation::{Evaluation, Metrics};
use crate::models::WinRate;
use crate::utils::{cumulative_returns, normalise_returns, round_float};
use ndarray::arr1;


pub struct Backtest {
    signals: Vec<f64>,
    trading_costs: f64,
    weight_asset_1: f64, // Capital percentage on asset 1 between 0 and 1.0
    weight_asset_2: f64, // Capital percentage on asset 2 between 0 and 1.0
}

impl Backtest {
    pub fn new(signals: Vec<f64>, trading_costs: f64, weight_asset_1: f64, weight_asset_2: f64) -> Self {
        Self { weight_asset_1, trading_costs, weight_asset_2, signals }
    }

    /// Trade Costs
    /// Returns trading costs in correct sequence based on signals
    fn trade_costs(&self) -> Vec<f64> {
        let mut trading_costs: Vec<f64> = vec![0.0; self.signals.len()];
        for i in 1..self.signals.len() {
            let val: f64 = self.signals[i];
            let prev_val: f64 = self.signals[i - 1];

            // Trade Closed
            if val == 0.0 && prev_val != 0.0 {
                trading_costs[i - 1] = -self.trading_costs;
            
            // Trade Opened
            } else if val != 0.0 && prev_val == 0.0 {
                trading_costs[i] = -self.trading_costs;

            // Trade Closed and Opened (switched sides)
            } else if val != 0.0 && prev_val != 0.0 && val != prev_val {
                trading_costs[i - 1] = -self.trading_costs;
                trading_costs[i] = -self.trading_costs;
            }
        }
        trading_costs
    }

    /// Win Rate Stats
    /// Provide stats and win rates
    fn win_rate_stats(&self, log_rets: &Vec<f64>) -> WinRate {
        let mut opened: u32 = 0;
        let mut closed: u32 = 0;
        let mut closed_profit: u32 = 0;
        let mut curr_profit: f64 = 0.0;
        let mut is_open: bool = false;

        for i in 1..self.signals.len() {
            let val: f64 = self.signals[i];
            let prev_val: f64 = self.signals[i - 1];

            // Trade Closed
            if val == 0.0 && prev_val != 0.0 {
                is_open = false;
                closed += 1;
                if curr_profit > 0.0 {
                    closed_profit += 1;
                }
                curr_profit = 0.0;

            // Trade Opened
            } else if val != 0.0 && prev_val == 0.0 {
                is_open = true;
                opened += 1;
                curr_profit += log_rets[i];

            // Trade Closed and Opened (switched sides)
            } else if val != 0.0 && prev_val != 0.0 && val != prev_val {
                closed += 1;
                if curr_profit > 0.0 {
                    closed_profit += 1;
                }
                curr_profit += log_rets[i];
                is_open = true;
                opened += 1;
            
            // Accumulate profits
            } else if is_open {
                curr_profit += log_rets[i];
            }
        }
        
        let mut win_rate: f64 = 0.0;
        if closed_profit > 0 && closed > 0 {
            win_rate = closed_profit as f64 / closed as f64;
        }

        WinRate { win_rate: round_float(win_rate, 2), opened, closed, closed_profit }
    }

    /// Add Vectors
    /// Adds two vectors together
    fn add_vecs(&self, vec_1: &Vec<f64>, vec_2: &Vec<f64>) -> Vec<f64> {
        let arr_1 = arr1(&vec_1);
        let arr_2 = arr1(&vec_2);
        let net_arr = arr_1 + arr_2;
        net_arr.to_vec()
    }

    /// Construct Portfolio Returns
    /// Takes in log returns and computes portfolio returns as such:
    /// Asset_1: log_returns * signal (long, short neutral) * (sign as +1.0) * capital_weighting
    /// Asset_2: log_returns * signal (long, short neutral) * inverse (sign as -1.0) * capital_weighting
    /// The inverse is used for asset_2 as the original signal was constructed for asset 1. Asset 2 is just the other side
    fn construct_portfolio_returns(&self, log_rets: Vec<f64>, trading_costs: &Vec<f64>, sign: f64,  weight: f64) -> Vec<f64> {

        // Get strategy returns
        let rets_arr = arr1(&log_rets);
        let sig_arr = arr1(&self.signals);
        let strat_log_rets_arr = rets_arr * sig_arr * sign * weight;
        let strat_log_rets = strat_log_rets_arr.to_vec();
        
        // Add trading costs
        let strat_log_rets_with_costs: Vec<f64> = self.add_vecs(&strat_log_rets, trading_costs);

        // Returns
        strat_log_rets_with_costs
    }

    /// Run Pairs Backtest
    /// Performs all steps needed to execute a full backtest for a pairs trade
    pub fn run_backtest(&self, log_rets_1: Vec<f64>, log_rets_2_opt: Option<Vec<f64>>) -> Result<Metrics, String> {

        // Trading costs
        let trading_costs: Vec<f64> = self.trade_costs();

        // Asset 1 Returns
        let strat_log_rets_1: Vec<f64> = self.construct_portfolio_returns(log_rets_1, &trading_costs, 1.0, self.weight_asset_1);
        
        // Log Returns (including asset 2 returns assumed as pairs trade if provided)
        let log_returns: Vec<f64> = match log_rets_2_opt {
            Some(log_rets_2) => {
                let strat_log_rets_2: Vec<f64> = self.construct_portfolio_returns(log_rets_2, &trading_costs, -1.0, self.weight_asset_2);
                self.add_vecs(&strat_log_rets_1, &strat_log_rets_2)
            },
            None => strat_log_rets_1
        };

        // Get Cumulative returns
        let strat_cum_log_rets: Vec<f64> = cumulative_returns(&log_returns);

        // Normalise returns
        let cum_norm_returns: Vec<f64> = normalise_returns(&strat_cum_log_rets);

        // Win Rate Stats
        let win_rate_stats: WinRate = self.win_rate_stats(&log_returns);

        // Evaluation Metrics
        let evaluation: Evaluation = Evaluation::new(log_returns, cum_norm_returns, win_rate_stats);
        let eval_metrics: Metrics = evaluation.run_evaluation_metrics();
    
        // Return JSON string result
        Ok(eval_metrics)
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Signals;
    use tradestats::metrics::{spread_standard, rolling_zscore};
    use csv::Reader;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct Record {
      series_1: f64,
      series_2: f64,
    }
    
    pub fn get_test_data() -> (Vec<f64>, Vec<f64>) {
      let mut rdr: Reader<std::fs::File> = Reader::from_path("data/data.csv").unwrap();
      let mut series_1: Vec<f64> = vec![];
      let mut series_2: Vec<f64> = vec![];
      for result in rdr.deserialize() {
        let record: Record = result.unwrap();
        series_1.push(record.series_1);
        series_2.push(record.series_2);
      }
      (series_1, series_2)
    }
    

    #[test]
    fn tests_backtest() {
        let (series_1, series_2) = get_test_data();
        let log_rets_1: Vec<f64> = tradestats::utils::log_returns(&series_1, true);
        let log_rets_2: Vec<f64> = tradestats::utils::log_returns(&series_2, true);
  
        let spread: Vec<f64> = spread_standard(&series_1, &series_2).unwrap();
        let roll_zscore: Vec<f64> = rolling_zscore(&spread, 21).unwrap();

        let trading_costs: f64 = 0.001;
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
        
        // Run Backtest
        let backtest: Backtest = Backtest::new(net_signals, trading_costs, weighting_asset_1, weighting_asset_2);
        let backtest_result: Result<Metrics, String> = backtest.run_backtest(log_rets_1, Some(log_rets_2));
        match backtest_result {
            Ok(_) => assert!(true),
            Err(_) => assert!(false)
        }
    }
}
