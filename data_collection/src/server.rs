use std::collections::HashMap;
use std::fs::File;
use std::ops::Sub;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use log::info;
use serde::{Deserialize, Serialize};

use crate::get_as_json;

#[derive(Serialize, Deserialize, Debug)]
struct StationData {
    trips: HashMap<String, Trip>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Trip {
    delay: u64,
    product_type: String,
    cancelled: Option<bool>,
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
                delay: 0,
                product_type: product_type.clone(),
                cancelled: Some(true),
            });
        }

        let delay = match entry.get("delay") {
            None => {
                0u64
            }
            Some(delay_val) => {
                if (delay_val.is_null()) {
                    0_u64
                } else {
                    delay_val.as_u64().unwrap()
                }
            }
        };
        info!("{}:{}:{}", &trip_id, &product_type, &delay);
        station_data.trips.insert(trip_id, Trip {
            delay,
            product_type,
            cancelled: None,
        });
    }
    let content = serde_json::to_string(&station_data)?;
    std::fs::write(path, content)?;

    Ok(())
}