use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::{RwLock, Arc};
use chrono::prelude::*;
use chrono::Duration;

static CALIFORNIA_GEO : &str = "CALIFORNIA";
static BAY_AREA_GEO : &str = "BAY AREA";
static GEO_PROP_NAME : &str = "GEOGRAPHY";
static CASES_CAT : &str = "cases";
static DEATHS_CAT : &str = "deaths";
static CATEGORY_PROP_NAME : &str = "CATEGORY";
static TOTAL_PROP_NAME : &str = "TOTALS";

static API_DATE_FORMAT : &str = "%-m/%d/%y";

pub struct PayloadStorage {
  cases_ba: u32,
  deaths_ba: u32,
  cases_ca: u32,
  deaths_ca: u32,
  one_day_ba_cases: i32,
  seven_day_ba_cases: i32,
  one_day_ba_deaths: i32,
  seven_day_ba_deaths: i32,
  one_day_ca_cases: i32,
  seven_day_ca_cases: i32,
  one_day_ca_deaths: i32,
  seven_day_ca_deaths: i32,
}

impl PayloadStorage {
  pub fn new() -> Self {
    PayloadStorage {
      cases_ba: 0,
      deaths_ba: 0,
      cases_ca: 0,
      deaths_ca: 0,
      one_day_ba_cases: 0,
      seven_day_ba_cases: 0,
      one_day_ba_deaths: 0,
      seven_day_ba_deaths: 0,
      one_day_ca_cases: 0,
      seven_day_ca_cases: 0,
      one_day_ca_deaths: 0,
      seven_day_ca_deaths: 0
    }
  }

  fn update_data(
    &mut self,
    cases_ba: u32,
    deaths_ba: u32,
    cases_ca: u32,
    deaths_ca: u32,
    one_day_ba_cases: i32,
    seven_day_ba_cases: i32,
    one_day_ba_deaths: i32,
    seven_day_ba_deaths: i32,
    one_day_ca_cases: i32,
    seven_day_ca_cases: i32,
    one_day_ca_deaths: i32,
    seven_day_ca_deaths: i32
  ) {
    self.cases_ba = cases_ba;
    self.deaths_ba = deaths_ba;
    self.cases_ca = cases_ca;
    self.deaths_ca = deaths_ca;
    self.one_day_ba_cases = one_day_ba_cases;
    self.seven_day_ba_cases = seven_day_ba_cases;
    self.one_day_ba_deaths = one_day_ba_deaths;
    self.seven_day_ba_deaths = seven_day_ba_deaths;
    self.one_day_ca_cases = one_day_ca_cases;
    self.seven_day_ca_cases = seven_day_ca_cases;
    self.one_day_ca_deaths = one_day_ca_deaths;
    self.seven_day_ca_deaths = seven_day_ca_deaths;
  }

  fn parse_json_data (&mut self, parsed: Vec<HashMap<String, String>>) {
    let mut cases_ba = 0;
    let mut deaths_ba = 0;
    let mut one_day_ba_cases = 0;
    let mut seven_day_ba_cases = 0;
    let mut one_day_ba_deaths = 0;
    let mut seven_day_ba_deaths = 0;
    let mut cases_ca = 0;
    let mut deaths_ca = 0;
    let mut one_day_ca_cases = 0;
    let mut seven_day_ca_cases = 0;
    let mut one_day_ca_deaths = 0;
    let mut seven_day_ca_deaths = 0;

    for obj in parsed {
      // we want to take only full days into account that's why we just start with previous day always
      let mut local: DateTime<Local> = Local::now() - Duration::days(1);
      let to_modify_opt = match (obj.get(GEO_PROP_NAME), obj.get(CATEGORY_PROP_NAME)) {
        (Some(g), Some(c)) if g == CALIFORNIA_GEO && c == CASES_CAT => {
          Some((& mut cases_ca, & mut one_day_ca_cases, & mut seven_day_ca_cases))
        },
        (Some(g), Some(c)) if g == CALIFORNIA_GEO && c == DEATHS_CAT => {
          Some((&mut deaths_ca, & mut one_day_ca_deaths, & mut seven_day_ca_deaths))
        },
        (Some(g), Some(c)) if g == BAY_AREA_GEO && c == CASES_CAT => {
          Some((& mut cases_ba, & mut one_day_ba_cases, & mut seven_day_ba_cases))
        }
        (Some(g), Some(c)) if g == BAY_AREA_GEO && c == DEATHS_CAT => {
          Some((& mut deaths_ba, & mut one_day_ba_deaths, & mut seven_day_ba_deaths))
        },
        (_, _) => None
      };
      if let Some((to_modify_total, to_modify_one_day, to_modify_seven_day)) = to_modify_opt {
        *to_modify_total = obj
          .get(TOTAL_PROP_NAME)
          .expect("total prop not found")
          .parse::<u32>()
          .expect("total prop is not a valid number");
        let mut iterations = 0;
        while obj.get(&local.format(API_DATE_FORMAT).to_string()).is_none() && iterations < 10 {
          local = local - Duration::days(1);
          iterations += 1;
        };
        // Otherwise assume that the date is not found
        if iterations < 10 {
          let prev_day_time = local - Duration::days(1);
          let prev_week_time = local - Duration::days(7);
          let last_day_val = obj
            .get(&local.format(API_DATE_FORMAT).to_string())
            .expect("total prop not found")
            .parse::<i32>()
            .expect("total prop is not a valid number");
          let day_before_last_day_val = obj
            .get(&prev_day_time.format(API_DATE_FORMAT).to_string())
            .expect("prev day prop not found")
            .parse::<i32>()
            .expect("prev day prop is not a valid number");
          *to_modify_one_day = last_day_val * 100 / day_before_last_day_val - 100;
          let mut seven_days_current_total = 0;
          let mut seven_days_prev_total = 0;
          for i in 0..7 {
            seven_days_current_total += obj
              .get(&(local - Duration::days(1 * i)).format(API_DATE_FORMAT).to_string())
              .expect(&format!("seven days current prop not found for idx: {}", i))
              .parse::<i32>()
              .expect(&format!("seven days current prop for idx: {} is not a number", i));
            seven_days_prev_total += obj
              .get(&(prev_week_time - Duration::days(1 * i)).format(API_DATE_FORMAT).to_string())
              .expect(&format!("seven days prev week prop not found for idx: {}", i))
              .parse::<i32>()
              .expect(&format!("seven days prev week prop for idx: {} is not a number", i));
          }
          *to_modify_seven_day = seven_days_current_total * 100 / seven_days_prev_total - 100;
        } else {
          println!("Closest current day does not exist in api response within range of 10 days from now");
        }
      }
    }
    self.update_data(
      cases_ba,
      deaths_ba,
      cases_ca,
      deaths_ca,
      one_day_ba_cases,
      seven_day_ba_cases,
      one_day_ba_deaths,
      seven_day_ba_deaths,
      one_day_ca_cases,
      seven_day_ca_cases,
      one_day_ca_deaths,
      seven_day_ca_deaths
    );
  }

  pub fn peek(&self) -> Vec<String> {
    vec![
      format!("{}", Local::now().format("%A %d %B %Y %H:%M")),
      format!(
        "Cases BA: {} ({}% 1d) ({}% 7d)",
        self.cases_ba,
        self.one_day_ba_cases,
        self.seven_day_ba_cases
      ),
      format!(
        "Deaths BA: {} ({}% 1d) ({}% 7d)",
        self.deaths_ba,
        self.one_day_ba_deaths,
        self.seven_day_ba_deaths
      ),
      format!(
        "Cases CA: {} ({}% 1d) ({}% 7d)",
        self.cases_ca,
        self.one_day_ca_cases,
        self.seven_day_ca_cases
      ),
      format!(
        "Deaths CA: {} ({}% 1d) ({}% 7d)",
        self.deaths_ca,
        self.one_day_ca_deaths,
        self.seven_day_ca_deaths
      ),
    ]
  }
}

pub fn handle_data_updates (
  receiver: Receiver<Vec<HashMap<String, String>>>,
  storage: Arc<RwLock<PayloadStorage>>
) {
  for message in receiver {
    let mut writable_storage = storage.write().expect("Can not read storage");
    writable_storage.parse_json_data(message);
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::request::get_data;
  use tokio::runtime::{Runtime};

  #[test]
  fn can_get_and_parse_data() -> Result<(), String> {
    let mut rt = Runtime::new()
      .map_err(|e| e.to_string())?;
    let result: Result<Vec<HashMap<String, String>>, String> = rt.block_on(async {
      let resp = get_data().await;
      resp
    });

    let mut storage = PayloadStorage::new();
    storage.parse_json_data(result?);

    Ok(())
  }
}