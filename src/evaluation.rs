use crate::utils::normalise_returns;

struct Evaluation {
  log_returns: Vec<f64>,
  cum_norm_returns: Vec<f64>,
}

impl Evaluation {
  pub fn new(log_returns: Vec<f64>, cum_norm_returns: Vec<f64>) -> Self {
    Self {
      log_returns,
      cum_norm_returns
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
    let mut max_return_so_far = norm_returns[0];
    for r in norm_returns {
      if r > max_return_so_far {
        max_return_so_far = r;
      }
      let drawdown: f64 = max_return_so_far - r;
      drawdowns.push(drawdown);
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

}
