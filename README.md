### Example Usage

```rust
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
      let backtest_result: Result<String, String> = backtest.run_backtest(log_rets_1, Some(log_rets_2));
      dbg!(&backtest_result);
      assert!(backtest_result.unwrap().len() > 100);
  }
```
