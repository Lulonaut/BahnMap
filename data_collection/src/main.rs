use clap::{Parser, Subcommand};
use log::LevelFilter::Info;
use serde_json::Value;
use simple_logger::SimpleLogger;

use crate::filter_stations::filter_stations;
use crate::server::update_station;

mod filter_stations;
mod server;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    ///Update and filter the stations from the API
    FilterStations {
        ///The station weight cutoff
        #[arg()]
        minimum_weight: u64
    },

    ///Start collecting data for all stations in stations.json
    RunServer {},

    ///Update a single stations departures
    UpdateStation {
        #[arg()]
        station: u64
    },
}

pub fn get_as_json(url: &str) -> serde_json::Map<String, Value> {
    let client = reqwest::blocking::Client::builder().user_agent("BahnMap").build().unwrap();
    let resp = client.get(url).send().unwrap().json::<Value>().unwrap();
    let obj = resp.as_object().unwrap();

    obj.clone()
}

fn main() -> eyre::Result<()> {
    SimpleLogger::new()
        .without_timestamps()
        .with_colors(true)
        .with_level(Info)
        .env()
        .init()?;
    let cli = Cli::parse();

    match &cli.command {
        Commands::FilterStations { minimum_weight } => {
            filter_stations(*minimum_weight)?;
        }
        Commands::RunServer { .. } => {}
        Commands::UpdateStation { station } => {
            update_station(*station)?;
        }
    }

    Ok(())
}
