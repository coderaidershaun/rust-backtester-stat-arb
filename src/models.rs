use serde::{Deserialize, Serialize};
use ts_rs::TS;

/*
  Triggers Model
  Supporting generating opening and closing of positions
*/

#[derive(Debug, Deserialize, Serialize)]
pub enum SignalType {
  Long,
  Short
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Signals {
          // open       // close
  pub eq: (Option<f64>, Option<f64>),
  pub neq: (Option<f64>, Option<f64>),
  pub gt: (Option<f64>, Option<f64>),
  pub lt: (Option<f64>, Option<f64>),
  pub signal_type: SignalType
}

impl Signals {
  pub fn new(
    eq: (Option<f64>, Option<f64>),
    neq: (Option<f64>, Option<f64>),
    gt: (Option<f64>, Option<f64>),
    lt: (Option<f64>, Option<f64>),
    signal_type: SignalType
  ) -> Self {
    Self {
      eq, neq, gt, lt, signal_type
    }
  }

  /// Generate Triggers
  /// Takes in a vector of values and matches 1 for open position and -1 for close position
  /// This is regardless of long or short, 1 means open, -1 means close
  fn generate_triggers(&self, series: &Vec<f64>) -> Vec<f64> {
    let transformed: Vec<f64> = series.iter()
    .map(|&value| {
      match value {
        x if self.eq.0.is_some() && x == self.eq.0.unwrap() => 1.0,
        x if self.neq.0.is_some() && x != self.neq.0.unwrap() => 1.0,
        x if self.gt.0.is_some() && x >= self.gt.0.unwrap() => 1.0,
        x if self.lt.0.is_some() && x <= self.lt.0.unwrap() => 1.0,

        x if self.eq.1.is_some() && x == self.eq.1.unwrap() => -1.0,
        x if self.neq.1.is_some() && x != self.neq.1.unwrap() => -1.0,
        x if self.gt.1.is_some() && x >= self.gt.1.unwrap() => -1.0,
        x if self.lt.1.is_some() && x <= self.lt.1.unwrap() => -1.0,

        _ => 0.0,
      }
    })
    .collect();

    transformed
  }

  /// Generate Signals
  /// Calculates whether in position or not based on Triggers
  pub fn generate_signals(&self, series: &Vec<f64>) -> Vec<f64> {

    // Extract triggers (shows any occurance when to trigger a trade to open or close)
    let triggers: Vec<f64> = self.generate_triggers(series);

    // Determine sign direction
    let factor: f64 = match self.signal_type {
      SignalType::Long => 1.0,
      SignalType::Short => -1.0
    };

    // Determine sign direction
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
  pub fn consolidate_signals(&self, signals_arr: Vec<Vec<f64>>) -> Vec<f64> {
    let inner_len: usize = signals_arr[0].len();
    let mut sigs: Vec<f64> = vec![];
    for inner_i in 0..inner_len {
        for i in 0..signals_arr.len() {
            let val: f64 = signals_arr[i][inner_i];
            
            if val == 1.0 {
              sigs.push(1.0);
              break;
            } else if val == -1.0 {
              sigs.push(-1.0);
              break;
            }

            if i + 1 == signals_arr.len() { sigs.push(0.0) }
        }
    }
    sigs
  }
}

/*
  Win Rate Stats
  Figures for number of trades placed
*/

#[derive(Debug, Deserialize, Serialize, Clone, TS)]
pub struct WinRate {
  pub win_rate: f64,
  pub opened: u32,
  pub closed: u32,
  pub closed_profit: u32
}
