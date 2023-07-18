use serde::{Deserialize, Serialize};

/*
  Triggers Model
  Supporting generating opening and closing of positions
*/

#[derive(Debug)]
pub struct Triggers {
          // open       // close
  pub eq: (Option<f64>, Option<f64>),
  pub neq: (Option<f64>, Option<f64>),
  pub gt: (Option<f64>, Option<f64>),
  pub lt: (Option<f64>, Option<f64>)
}

impl Triggers {
  pub fn new(
    eq: (Option<f64>, Option<f64>),
    neq: (Option<f64>, Option<f64>),
    gt: (Option<f64>, Option<f64>),
    lt: (Option<f64>, Option<f64>)
  ) -> Self {
    Self {
      eq, neq, gt, lt
    }
  }

  /// Generate Triggers
  /// Takes in a vector of values and matches 1 for open position and -1 for close position
  /// This is regardless of long or short, 1 means open, -1 means close
  pub fn generate_triggers(&self, series: &Vec<f64>) -> Vec<f64> {
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
}

/*
  TradeStats
  Figures for number of trades placed
*/

#[derive(Debug)]
pub struct TradeStats {
  longs_opened: u32,
  shorts_opened: u32,
  longs_closed: u32,
  shorts_closed: u32,
  net_opened: u32,
  net_closed: u32,
  longs_profitable: u32,
  shorts_profitable: u32,
  net_win_rate: f64,
}

/*
  Metrics
  Metrics for trading performance evaluation
*/

#[derive(Debug)]
pub struct Metrics {
  pub trade_stats: Option<TradeStats>,
  pub arr: f64,
  pub roi: f64,
  pub sharpe_ratio: f64,
  pub sortino_ratio: f64,
  pub max_drawdown: f64,
  pub drawdowns: Vec<f64>,
}

impl Metrics {
  pub fn new() -> Self {
    Self {
      trade_stats: None, arr: 0.0, roi: 0.0, sharpe_ratio: 0.0,
      sortino_ratio: 0.0, max_drawdown: 0.0, drawdowns: vec![]
    }
  }


}