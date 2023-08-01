use crate::models::WinRate;
use crate::utils::{normalise_returns, round_float};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Metrics {
  pub arr: f64,
  pub drawdowns: Vec<f64>,
  pub equity_curve: Vec<f64>,
  pub max_drawdown: f64,
  pub mean_return: f64,
  pub sharpe_ratio: f64,
  pub sortino_ratio: f64,
  pub total_return: f64,
  pub win_rate_stats: WinRate
}

#[derive(Debug)]
pub struct Evaluation {
  pub log_returns: Vec<f64>,
  pub cum_norm_returns: Vec<f64>,
  pub win_rate_stats: WinRate,
}

impl Evaluation {
  pub fn new(log_returns: Vec<f64>, cum_norm_returns: Vec<f64>, win_rate_stats: WinRate) -> Self {
    Self {
      log_returns,
      cum_norm_returns,
      win_rate_stats,
    }
  }

  // Annual Rate of Return
  fn annual_rate_of_return(&self) -> f64 {
    let mean_return: f64 = self.mean_return();
    let periods_per_year: f64 = 252.0; // for daily returns
    (1.0 + mean_return).powf(periods_per_year) - 1.0
  }

  /// Drawdowns
  fn drawdowns(&self) -> Vec<f64> {
    let norm_returns: Vec<f64> = normalise_returns(&self.log_returns);
    let mut drawdowns: Vec<f64> = Vec::new();
    let mut max_return_so_far: f64 = norm_returns[0];
    for r in norm_returns {
      if r > max_return_so_far {
        max_return_so_far = r;
      }
      let drawdown: f64 = max_return_so_far - r;
      drawdowns.push(-drawdown);
    }
    drawdowns
  }

  /// Mean Return
  /// Takes in log returns and provides a linear mean return value
  fn mean_return(&self) -> f64 {
    let filtered_vec: Vec<&f64> = self.log_returns.iter().filter(|&&x| x != 0.0).collect();
    let sum: f64 = filtered_vec.iter().fold(0.0, |a, b| a + **b);
    let count: usize = filtered_vec.len();
    
    let log_ret = match count {
      0 => 0.0,
      _ => sum / (count as f64),
    };

    f64::exp(log_ret) - 1.0
  }

  /// Sharpe Ratio
  fn sharpe_ratio(&self) -> f64 {
    let n: f64 = self.log_returns.len() as f64;
    if n == 0.0 { return 0.0; };

    let mean: f64 = self.log_returns.iter().sum::<f64>() / n;
    if mean == 0.0 { return 0.0; };

    let variance: f64 = self.log_returns.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n;
    if variance == 0.0 { return 0.0; };

    mean / variance.sqrt()
  }

  /// Sortino Ratio without risk-free rate
  fn sortino_ratio(&self) -> f64 {
    let n: f64 = self.log_returns.len() as f64;
    if n == 0.0 { return 0.0; };

    let mean: f64 = self.log_returns.iter().sum::<f64>() / n;
    if mean == 0.0 { return 0.0; };

    // Filter only negative returns
    let negative_returns: Vec<f64> = self.log_returns.iter().filter(|&&x| x < 0.0).map(|&x| x.powi(2)).collect();
    let n_neg: f64 = negative_returns.len() as f64;

    if n_neg == 0.0 { return 0.0; };

    let downside_variance: f64 = negative_returns.iter().sum::<f64>() / n_neg;
    if downside_variance == 0.0 { return 0.0; };

    mean / downside_variance.sqrt()
  }

  /// Total Return
  fn total_return(&self) -> f64 {
    self.cum_norm_returns[self.cum_norm_returns.len() - 1]
  }

  /// Run Evaluation Metrics
  /// Calculates metrics and returns net evaluation serialized
  pub fn run_evaluation_metrics(&self) -> Metrics {

    let arr: f64 = round_float(self.annual_rate_of_return(), 2);
    let drawdowns: Vec<f64> = self.drawdowns().iter().map(|f| round_float(*f, 3)).collect();
    let equity_curve: Vec<f64> = self.cum_norm_returns.iter().map(|f| round_float(*f, 3)).collect();
    let max_drawdown: f64 = round_float(drawdowns.iter().cloned().fold(f64::NAN, f64::min), 2);
    let mean_return: f64 = round_float(self.mean_return(), 3);
    let sharpe_ratio: f64 = round_float(self.sharpe_ratio(), 2);
    let sortino_ratio: f64 = round_float(self.sortino_ratio(), 2);
    let total_return: f64 = round_float(self.total_return(), 2);
    let win_rate_stats: WinRate = self.win_rate_stats.to_owned();

    Metrics { arr, drawdowns, equity_curve, max_drawdown, mean_return, 
      sharpe_ratio, sortino_ratio, total_return, win_rate_stats }
  }

}
