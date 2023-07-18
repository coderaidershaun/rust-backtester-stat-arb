use crate::utils::{cumulative_returns, normalise_returns};
use ndarray::arr1;

enum SigType {
    Long,
    Short
}

pub struct Backtest {
    signals: Option<Vec<f64>>
}

impl Backtest {
    pub fn new() -> Self {
        Self { signals: None }
    }

    /// Generate Signals
    /// Calculates whether in position or not based on Triggers
    fn generate_signals(&self, triggers: Vec<f64>, signal_type: SigType) -> Vec<f64> {

        let factor: f64 = match signal_type {
            SigType::Long => 1.0,
            SigType::Short => -1.0
        };

        let mut open_signals: Vec<f64> = vec![0.0];
        let mut is_open: bool = false;
        for i in 1..triggers.len() {
            let prev_val: f64 = triggers[i - 1];
            
            if !is_open && prev_val == 1.0 {
                is_open = true;
                open_signals.push(factor);
            } else if is_open && prev_val != -1.0 {
                open_signals.push(factor);
            } else {
                is_open = false;
                open_signals.push(0.0);
            }
        }
        open_signals
    }

    /// Consolidate Signals
    /// Takes in an array of signals and creates a consolidated Signal for whether should be long, short or neutral
    fn consolidate_signals(&mut self, signals_arr: Vec<Vec<f64>>) {
        let inner_len: usize = signals_arr[0].len();
        let mut sigs: Vec<f64> = vec![];
        for inner_i in 0..inner_len {
            for i in 0..signals_arr.len() {
                let val: f64 = signals_arr[i][inner_i];
                match val {
                    1.0 => {
                        sigs.push(1.0);
                        break;
                    },
                    -1.0 => {
                        sigs.push(-1.0);
                        break;
                    },
                    _ => {}
                };
                if i + 1 == signals_arr.len() { sigs.push(0.0) }
            }
        }
        self.signals = Some(sigs);
    }

    /// Construct Portfolio Returns
    /// Takes in log returns and computes portfolio returns as such:
    /// Asset_1: log_returns * signal (long, short neutral) * (sign as +1.0) * capital_weighting
    /// Asset_2: log_returns * signal (long, short neutral) * inverse (sign as -1.0) * capital_weighting
    /// The inverse is used for asset_2 as the original signal was constructed for asset 1. Asset 2 is just the other side
    fn construct_portfolio_returns(log_rets: Vec<f64>, sign: f64,  weight: f64) {

        // Get strategy returns
        let rets_arr = arr1(&log_rets);
        let sig_arr = arr1(&signals);
        let strat_log_rets = rets_arr * sig_arr * sign * weight;

        // Get Cumulative returns
        let strat_cum_log_rets: Vec<f64> = cumulative_returns(&strat_log_rets.to_vec());

        // Normalise returns
        let strat_cum_norm_rets: Vec<f64> = normalise_returns(&strat_cum_log_rets);
    }

    /// Run Backtest
    pub fn run_backtest(
        &self, log_rets_1: Vec<f64>, log_rets_2: Vec<f64>, asset_1_w: f64, asset_2_w: f64
    ) -> Result<(), String> {

        construct_portfolio_returns(log_rets_1, 1.0, asset_1_w);

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
    use crate::models::Triggers;
    use tradestats::utils::log_returns;
    use tradestats::metrics::{spread_standard, rolling_zscore};

    fn generate_zscore_triggers_long(series: &Vec<f64>) -> Vec<f64> {
        let eq = (Some(-1.5), Some(0.0));
        let neq = (None, None);
        let gt = (None, Some(0.0));
        let lt = (Some(-1.5), None);

        let triggers: Triggers = Triggers::new(eq, neq, gt, lt);
        triggers.generate_triggers(series)
    }

    fn generate_zscore_triggers_short(series: &Vec<f64>) -> Vec<f64> {
        let eq = (Some(1.5), Some(0.0));
        let neq = (None, None);
        let gt = (Some(1.5), None);
        let lt = (None, Some(0.0));

        let triggers: Triggers = Triggers::new(eq, neq, gt, lt);
        triggers.generate_triggers(series)
    }

    #[test]
    fn tests_equity_curve() {
        let (series_1, series_2) = get_test_data();
        let log_rets_1: Vec<f64> = log_returns(&series_1, true);
        let log_rets_2: Vec<f64> = log_returns(&series_2, true);
  
        let spread: Vec<f64> = spread_standard(&series_1, &series_2).unwrap();
        let roll_zscore: Vec<f64> = rolling_zscore(&spread, 21).unwrap();

        let weighting_asset_1: f64 = 1.0; // Amount of capital to assign to asset 1
        let weighting_asset_2: f64 = 1.0; // Amount of capital to assign to asset 2

        // Long means long series 1, short series 2
        let long_triggers_zscore: Vec<f64> = generate_zscore_triggers_long(&roll_zscore);
        let long_open_signals: Vec<f64> = signal_generation(long_triggers_zscore, 1.0);

        // Short means short series 1, long series 2
        let short_triggers_zscore: Vec<f64> = generate_zscore_triggers_short(&roll_zscore);
        let short_open_signals: Vec<f64> = signal_generation(short_triggers_zscore, -1.0);
        
        let signals_main: Vec<f64> = consolidate_signals(vec![long_open_signals, short_open_signals]);
        
        run_backtest(log_rets_1, log_rets_2, signals_main, weighting_asset_1, weighting_asset_2);
    }
}
