use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::sync::{Arc, RwLock};

use clap::{App, Arg};
use pretty_env_logger;
use serde::Deserialize;
use serde::Serialize;
use tokio::process::Command;
use warp::{http, reply::Reply, Filter};

#[derive(Serialize, Deserialize)]
struct Machine {
    hostname: Option<String>,
    // TODO handle multiple interfaces?
    ip: Option<String>,
    mac: String,
    ipmi: Option<IPMI>,
}

#[derive(Serialize, Deserialize)]
struct IPMI {
    address: String,
    username: String,
    password: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct Payload {
    kernel: String,
    initrd: Vec<String>,
    cmdline: String,
    message: String,
}

#[derive(Serialize, Deserialize)]
struct State {
    machines: HashMap<String, Machine>,
    payloads: HashMap<String, Payload>,
    currently_booting: RwLock<HashMap<String, String>>,
}

fn load_machines(
    machines_file: &str,
) -> Result<HashMap<String, Machine>, Box<dyn std::error::Error>> {
    let f = File::open(machines_file)?;
    let machines: HashMap<String, Machine> = serde_yaml::from_reader(f)?;
    Ok(machines)
}

fn load_payloads(
    payload_file: &str,
) -> Result<HashMap<String, Payload>, Box<dyn std::error::Error>> {
    let f = File::open(payload_file)?;
    let payloads: HashMap<String, Payload> = serde_yaml::from_reader(f)?;
    Ok(payloads)
}

fn load_state(machine_file: &str, payload_file: &str) -> Result<State, Box<dyn std::error::Error>> {
    let machines = load_machines(machine_file)?;
    let payloads = load_payloads(payload_file)?;
    let currently_booting = RwLock::new(HashMap::new());
    Ok(State {
        machines,
        payloads,
        currently_booting,
    })
}

async fn list_machines(state: Arc<State>) -> Result<impl warp::Reply, warp::Rejection> {
    let json = serde_json::to_string_pretty(&state).unwrap();
    Ok(json)
}

async fn pixiecore_boot(
    state: Arc<State>,
    mac: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    //let state = &mut (*state.write().unwrap());
    let currently_booting = state.currently_booting.write().unwrap();
    match currently_booting.get(&mac) {
        Some(payload_name) => match state.payloads.get(payload_name) {
            Some(payload) => {
                // TODO figure out why pixiecore calls this three times
                //currently_booting.remove(&mac);
                Ok(serde_json::to_string_pretty(payload)
                    .unwrap()
                    .into_response())
            }
            None => Ok(warp::reply::with_status(
                "payload not found\n",
                http::StatusCode::NOT_FOUND,
            )
            .into_response()),
        },
        None => Ok(warp::reply::with_status(
            "MAC address not found in currently booting machines\n",
            http::StatusCode::NOT_FOUND,
        )
        .into_response()),
    }
}

async fn trigger_boot(
    state: Arc<State>,
    name: String,
    payload_name: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    //let state = &mut (*state.write().unwrap());
    match state.payloads.get(&payload_name) {
        Some(_) => match state.machines.get(&name) {
            Some(machine) => {
                {
                    // put this in its own scope to unlock immediately
                    state
                        .currently_booting
                        .write()
                        .unwrap()
                        .insert(machine.mac.to_owned(), payload_name);
                }
                if let Some(ipmi) = &machine.ipmi {
                    if let Err(_) = ipmi_boot(ipmi).await {
                        return Ok(http::StatusCode::INTERNAL_SERVER_ERROR);
                    }
                }
                Ok(http::StatusCode::OK)
            }
            None => Ok(http::StatusCode::NOT_FOUND),
        },
        // TODO error message?
        None => Ok(http::StatusCode::BAD_REQUEST),
    }
}

async fn ipmi_boot(ipmi: &IPMI) -> std::io::Result<()> {
    let base_args = &[
        "-I",
        "lanplus",
        "-L",
        "OPERATOR",
        "-H",
        &ipmi.address,
        "-U",
        &ipmi.username,
        "-P",
        &ipmi.password,
    ];

    let status = Command::new("ipmitool")
        .args(base_args)
        .args(&["chassis", "bootdev", "pxe"])
        .status()
        .await?;
    if !status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to set next boot device to PXE.",
        ));
    }

    Command::new("ipmitool")
        .args(base_args)
        .args(&["chassis", "power", "reset"])
        .status()
        .await?;
    if !status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to reset power.",
        ));
    }

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Config {
    cert_path: String,
    key_path: String,
    machines_path: String,
    payloads_path: String,
}

fn load_config(config_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let f = File::open(config_path)?;
    let config: Config = serde_yaml::from_reader(f)?;
    Ok(config)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::var_os("RUST_LOG").is_none() {
        // Set `RUST_LOG=todos=debug` to see debug logs,
        // this only shows access logs.
        env::set_var("RUST_LOG", "hwlender=info");
    }
    pretty_env_logger::init();

    let args = App::new("Hardware Lender")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Path to config file")
                .takes_value(true),
        )
        .get_matches();
    let config_path = args.value_of("config").unwrap_or("config.yml");
    let config = load_config(config_path)?;

    let state = Arc::new(load_state(&config.machines_path, &config.payloads_path)?);
    let state_filter = warp::any().map(move || state.clone());

    // /v1/list
    let get_list = warp::get()
        .and(state_filter.clone())
        .and(warp::path!("v1" / "list"))
        .and_then(list_machines);

    // /pixiecore/v1/boot/{mac}
    let get_pixiecore_boot = warp::get()
        .and(state_filter.clone())
        .and(warp::path!("pixiecore" / "v1" / "boot" / String))
        .and_then(pixiecore_boot);

    // /v1/triggerBoot/name/{name}/payload/{payload}
    let post_trigger_boot = warp::post()
        .and(state_filter.clone())
        .and(warp::path!(
            "v1" / "triggerBoot" / "name" / String / "payload" / String
        ))
        .and_then(trigger_boot);

    let routes = get_list
        .or(get_pixiecore_boot)
        .or(post_trigger_boot)
        .with(warp::log("hwlender"));

    warp::serve(routes)
        .tls()
        .cert_path(config.cert_path)
        .key_path(config.key_path)
        .run(([127, 0, 0, 1], 3030))
        .await;
    Ok(())
}
