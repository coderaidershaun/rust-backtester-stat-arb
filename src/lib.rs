pub mod backtest;
pub mod evaluation;
pub mod models;
pub mod utils;

use csv::Reader;
use serde::Deserialize;

pub type BoxErr = Box<dyn std::error::Error>;

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
