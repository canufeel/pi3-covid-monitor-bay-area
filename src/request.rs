use tokio::runtime::{Runtime};
use tokio::time::delay_for;
use std::sync::mpsc::Sender;
use std::collections::HashMap;
use crate::stop::STOP;
use std::sync::atomic::{Ordering};
use std::time::{Duration, Instant};

static REQUEST_URL : &str = "https://files.sfchronicle.com/project-feeds/covid19_us_cases_ca_by_county_.json";
static REQUEST_TIMEOUT_SECS : u64 = 120;

pub async fn get_data () -> Result<Vec<HashMap<String, String>>, String> {
  reqwest::get(REQUEST_URL)
    .await
    .map_err(|e| e.to_string())?
    .json::<Vec<HashMap<String, String>>>()
    .await
    .map_err(|e| e.to_string())
}

pub fn get_data_loop (tx: Sender<Vec<HashMap<String, String>>>) -> Result<(), String> {
  let mut rt = Runtime::new()
    .map_err(|e| e.to_string())?;

  let result: Result<(), String> = rt.block_on(async {
    let max_wait = Duration::from_secs(REQUEST_TIMEOUT_SECS);
    'thread_loop: loop {

      match get_data().await {
        Ok(data) => {
          println!("{:#?}", data);
          tx
            .send(data)
            .map_err(|e| e.to_string())?;
        },
        _ => {}
      }

      let start = Instant::now();
      while start.elapsed() < max_wait {
        if STOP.load(Ordering::Acquire) {
          break 'thread_loop;
        }
        delay_for(Duration::from_millis(100)).await;
      }
    }
    Ok(())
  });

  result?;

  Ok(())
}
