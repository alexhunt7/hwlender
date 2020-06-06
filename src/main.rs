use std::collections::HashMap;
use std::fs::File;
//use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

//use futures_util::TryStreamExt;
//use hyper::service::{make_service_fn, service_fn};
//use hyper::{Body, Method, Request, Response, Server, StatusCode};
use serde::Deserialize;
use serde::Serialize;
use serde_json;
use warp::{http, Filter};

#[derive(Serialize, Deserialize)]
struct NetworkInterface {
    ip: String,
    mac: String,
}

#[derive(Serialize, Deserialize)]
struct Machine {
    // TODO
    hostname: String,
    interfaces: Vec<NetworkInterface>,
    ipmi: NetworkInterface,
    status: Status,
}

#[derive(Serialize, Deserialize)]
enum Status {
    Idle,
    InPXEBoot(Payload),
}

#[derive(Serialize, Deserialize)]
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
    Ok(State {
        machines: load_machines(machine_file)?,
        payloads: load_payloads(payload_file)?,
    })
}

async fn list_machines(state: Arc<RwLock<State>>) -> Result<impl warp::Reply, warp::Rejection> {
    let state = &(*state.read().unwrap());
    let json = serde_json::to_string_pretty(state).unwrap();
    Ok(json)
}

async fn pixiecore_boot(
    state: Arc<RwLock<State>>,
    mac: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    // TODO return https://github.com/danderson/netboot/blob/master/pixiecore/README.api.md
    unimplemented!("Pixiecore API not yet implemented");
    Ok("")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(RwLock::new(load_state("machines.yml", "payloads.yml")?));
    let state_filter = warp::any().map(move || state.clone());

    // /v1/list
    let get_list = warp::get()
        .and(state_filter.clone())
        .and(warp::path!("v1" / "list"))
        .and_then(list_machines);

    // /v1/boot/{mac}
    let get_pixiecore_boot = warp::get()
        .and(state_filter.clone())
        .and(warp::path!("pixiecore" / "v1" / "boot" / String))
        .and_then(pixiecore_boot);

    // /v1/triggerBoot/mac/{mac}/payload/{payload}

    let routes = get_list.or(get_pixiecore_boot);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    Ok(())
}
