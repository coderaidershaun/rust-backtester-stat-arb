use std::f64;

// Normalise Returns
// Converts log returns to normal for an array
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
