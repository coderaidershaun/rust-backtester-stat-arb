use std::f64;

/// Number Round
/// Round number to n decimal places
pub fn round_float(num: f64, decimals: i32) -> f64 {
  let multiplier: f64 = 10f64.powi(decimals);
  (num * multiplier).round() / multiplier
}

/// Normalise Returns
/// Converts log returns to normal for an array
pub fn normalise_returns(log_returns: &Vec<f64>) -> Vec<f64> {
  log_returns.iter().map(|&x| f64::exp(x) - 1.0).collect()
}

/// Cumulative Returns
/// Cumulatively adds and log returns
pub fn cumulative_returns(log_returns: &Vec<f64>) -> Vec<f64> {
  log_returns.iter().scan(0.0, |state, &x| {
    *state = *state + x;
    Some(*state)
  }).collect()
}
