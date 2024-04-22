use clap::{Parser, Subcommand};
use log::LevelFilter;
use serde_json::Value;
use simple_logger::SimpleLogger;

use crate::filter_stations::filter_stations;
use crate::server::{run_server, update_station};

mod filter_stations;
mod server;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[arg(short, long, default_value = "false")]
    log_to_file: bool,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    ///Update and filter the stations from the API
    FilterStations {
        ///The station weight cutoff
        #[arg(default_value = "450")]
        minimum_weight: u64
    },

    ///Start collecting data for all stations in stations.json
    RunServer,

    ///Update a single stations departures
    UpdateStation {
        #[arg()]
        station: u64
    },
}

pub fn get_as_json(url: &str) -> serde_json::Map<String, Value> {
    let client = reqwest::blocking::Client::builder().user_agent("BahnMap (https://github.com/Lulonaut/BahnMap)").build().unwrap();
    let resp = client.get(url).send().unwrap().json::<Value>().unwrap_or(Value::Object(serde_json::Map::new()));
    let obj = resp.as_object().unwrap();

    obj.clone()
}

fn main() -> eyre::Result<()> {

    let cli = Cli::parse();

    if cli.log_to_file {
        simple_logging::log_to_file("test.log", LevelFilter::Info)?;
    } else {
        SimpleLogger::new()
            .without_timestamps()
            .with_colors(true)
            .with_level(LevelFilter::Info)
            .env()
            .init()?;
    }

    match &cli.command {
        Commands::FilterStations { minimum_weight } => {
            filter_stations(*minimum_weight)?;
        }
        Commands::RunServer => {
            run_server()
        }
        Commands::UpdateStation { station } => {
            update_station(*station)?;
        }
    }

    Ok(())
}
