use std::collections::HashMap;
use std::fs::File;

use log::info;
use serde::{Deserialize, Serialize};

use crate::{EvaluationMode, ProductType};
use crate::filter_stations::Station;
use crate::ProductType::{All, AllNational, AllRegional, National, NationalExpress, Regional, RegionalExpress, Suburban};
use crate::server::{StationData, Trip};

#[derive(Serialize, Deserialize, Debug)]
pub struct EvalResult {
    id: String,
    name: String,
    lat: f64,
    long: f64,
    result: f64,
    weight: f64,
}

pub fn evaluate(product_type: ProductType, evaluation_mode: EvaluationMode) -> eyre::Result<()> {
    let stations_file = File::open("stations.json")?;
    let stations: Vec<Station> = serde_json::from_reader(stations_file)?;
    let mut station_map: HashMap<String, Vec<Trip>> = HashMap::with_capacity(stations.len());
    let mut trips_map: HashMap<Station, i32> = HashMap::with_capacity(200);
    info!("Starting to parse {} stations", stations.len());

    for station in &stations {
        let file = File::open(format!("data/{}.json", station.id))?;
        let station_data: StationData = serde_json::from_reader(file)?;
        let trips: Vec<Trip> =
            station_data.trips.iter().filter(|trip| {
                if product_type == All {
                    true
                } else {
                    let parsed_product = trip.1.product_type.as_str();
                    match parsed_product {
                        "nationalExpress" => product_type == NationalExpress || product_type == AllNational,
                        "national" => product_type == National || product_type == AllNational,
                        "regionalExpress" => product_type == RegionalExpress || product_type == AllRegional,
                        "regional" => product_type == Regional || product_type == AllRegional,
                        "suburban" => product_type == Suburban,
                        _ => false,
                    }
                }
            })
                .map(|trip| {
                    trip.1.clone()
                }).collect();
        station_map.insert(station.id.clone(), trips);
    }
    info!("Done, sorting...");
    let mut sorted_map: HashMap<String, f64> = HashMap::with_capacity(station_map.len());
    for entry in station_map {
        match evaluation_mode {
            EvaluationMode::DelayPercentage => {
                //https://de.wikipedia.org/wiki/P%C3%BCnktlichkeit_(Bahn)#Terminologie
                let delay_threshold = 60 * 6;
                let delayed_trips = entry.1.iter().filter(|trip| trip.delay >= delay_threshold).count() as f64;
                if entry.1.is_empty() {
                    continue;
                }
                let delay_percentage = delayed_trips / entry.1.len() as f64;
                sorted_map.insert(entry.0, delay_percentage);
            }
        }
    }

    let mut keys: Vec<&String> = sorted_map.keys().collect();
    keys.sort_by(|key1, key2| {
        let key1_result = sorted_map[*key1];
        let key2_result = sorted_map[*key2];

        key1_result.total_cmp(&key2_result)
    });
    keys.reverse();

    let result = keys.iter().map(|key| {
        let station = &stations.iter().find(|s| &&s.id == key).unwrap();


        EvalResult {
            id: key.clone().clone(),
            name: station.name.clone(),
            lat: station.lat,
            long: station.long,
            result: sorted_map[*key],
            weight: station.weight,
        }
    }).collect::<Vec<EvalResult>>();

    let path = "result.json";
    let content = serde_json::to_string_pretty(&result)?;
    std::fs::write(path, content)?;
    info!("Result written to {path}");


    Ok(())
}