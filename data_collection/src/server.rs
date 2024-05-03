use std::collections::HashMap;
use std::fs::File;
use std::ops::Sub;
use std::path::Path;
use std::thread;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use log::{info, warn};
use serde::{Deserialize, Serialize};

use crate::filter_stations::Station;
use crate::get_as_json;

#[derive(Serialize, Deserialize, Debug)]
pub struct StationData {
    pub trips: HashMap<String, Trip>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Trip {
    pub delay: i32,
    pub product_type: String,
}

pub fn update_station(station: u64) -> eyre::Result<()> {
    let path_str = format!("data/{station}.json");
    let path = Path::new(path_str.as_str());
    if !path.exists() {
        let data = StationData {
            trips: HashMap::new()
        };
        let content = serde_json::to_string(&data)?;
        std::fs::write(path, content)?;
    }

    let file = File::open(path)?;
    let mut station_data: StationData = serde_json::from_reader(file)?;
    let target = SystemTime::now().sub(Duration::from_secs(15 * 60));
    let unix_timestamp = target.duration_since(UNIX_EPOCH)?.as_secs();
    let url = format!("https://v6.db.transport.rest/stops/{station}/departures?when={unix_timestamp}&duration=15&taxi=false&tram=false&subway=false&ferry=false&bus=false");


    let resp = get_as_json(url.as_str());
    let departures = resp.get("departures").unwrap().as_array().unwrap();

    for entry in departures {
        let trip_id = entry.get("tripId").unwrap().as_str().unwrap().to_string();
        let product_type = entry.get("line").unwrap().as_object().unwrap().get("product").unwrap().as_str().unwrap().to_string();
        if entry.get("cancelled").is_some() {
            station_data.trips.insert(trip_id.clone(), Trip {
                delay: -1,
                product_type: product_type.clone(),
            });
            continue
        }

        let delay = match entry.get("delay") {
            None => {
                0
            }
            Some(delay_val) => {
                if delay_val.is_null() {
                    0
                } else {
                    delay_val.as_u64().unwrap() as i32
                }
            }
        };
        // info!("{}:{}:{}", &trip_id, &product_type, &delay);
        station_data.trips.insert(trip_id, Trip {
            delay,
            product_type,
        });
    }
    let content = serde_json::to_string(&station_data)?;
    std::fs::write(path, content)?;

    Ok(())
}

fn update_all_stations() -> eyre::Result<()> {
    let requests_per_minute = 50;
    let time = SystemTime::now();
    
    let stations_file = File::open("stations.json")?;
    let stations: Vec<Station> = serde_json::from_reader(stations_file)?;
    for station in stations {
        let id = station.id.parse::<u64>()?;
        update_station(id)?;
        sleep(Duration::from_millis(60_000 / requests_per_minute));
    }

    let time_taken = SystemTime::now().duration_since(time).unwrap().as_secs();
    info!("Iteration done! took {time_taken}s");

    Ok(())
}

pub fn run_server() {
    let delay = 15;
    info!("Starting server with a delay of {delay} minutes");
    loop {
        let handle = thread::spawn(update_all_stations);
        sleep(Duration::from_secs(60 * delay));
        if !handle.is_finished() {
            warn!("Thread is not done after {delay} minutes");
            let _ = handle.join();
        }
    }
}