use std::collections::HashSet;

use log::info;
use serde::{Deserialize, Serialize};

use crate::get_as_json;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Station {
    pub name: String,
    pub id: String,
    pub weight: f64,
    pub lat: f64,
    pub long: f64,
}

pub fn filter_stations(minimum_weight: u64) -> eyre::Result<()> {
    let stations_url = "https://v6.db.transport.rest/stations/";

    info!("Retrieving station list from {stations_url}...");
    let response = get_as_json(stations_url);

    let valid_stations: Vec<Station> = response.keys().filter(|key| {
        let entry = response[*key].clone();
        entry.get("weight").is_some()
    }).map(|key| {
        let entry = response[key].clone();
        if entry.get("weight").is_none() {
            println!("{}", entry);
            println!("{}", key);
        }
        let weight = entry.get("weight").unwrap().as_f64().unwrap();
        let name = entry.get("name").unwrap().as_str().unwrap().to_string();
        let id = entry.get("id").unwrap().as_str().unwrap().to_string();

        let location = entry.get("location").unwrap();
        let lat = location.get("latitude").unwrap().as_f64().unwrap();
        let long = location.get("longitude").unwrap().as_f64().unwrap();

        Station {
            name,
            id,
            weight,
            lat,
            long,
        }
    }).collect();

    let mut unique_ids = HashSet::new();
    let mut unique_stations = Vec::new();
    for station in valid_stations {
        if unique_ids.insert(station.id.clone()) {
            unique_stations.push(station);
        }
    }
    
    
    info!("Total number of stations: {}", unique_stations.len());

    let filtered_stations: Vec<&Station> = unique_stations.iter().filter(|station| station.weight > minimum_weight as f64).collect();
    // println!("{:?}", filtered_stations);
    info!("Stations with weight >={minimum_weight}: {}", filtered_stations.len());
    let stations_file = "stations.json";
    info!("Saving stations to {stations_file}");

    let content = serde_json::to_string(&filtered_stations)?;
    std::fs::write(stations_file, content)?;
    info!("Done.");
    Ok(())
}