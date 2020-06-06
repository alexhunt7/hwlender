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
    hostname: String,
    // TODO handle multiple interfaces?
    interface: NetworkInterface,
    ipmi: NetworkInterface,
    status: Status,
}

#[derive(Serialize, Deserialize)]
enum Status {
    Idle,
    InPXEBoot(String),
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
    currently_booting: HashMap<String, String>,
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
    let currently_booting = machines
        .iter()
        .filter_map(|(_name, machine)| match &machine.status {
            Status::InPXEBoot(payload_name) => {
                Some((machine.interface.mac.clone(), payload_name.clone()))
            }
            _ => None,
        })
        .collect();
    Ok(State {
        machines,
        payloads,
        currently_booting,
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

async fn trigger_boot(
    state: Arc<RwLock<State>>,
    name: String,
    payload_name: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    let state = &mut (*state.write().unwrap());
    match state.payloads.get(&payload_name) {
        Some(_) => match state.machines.get(&name) {
            Some(machine) => {
                state
                    .currently_booting
                    .insert(machine.interface.mac.to_owned(), payload_name);
                // TODO save machines.yml?
                // TODO set nextboot to pxe
                // TODO trigger reboot
                Ok(http::StatusCode::OK)
            }
            None => Ok(http::StatusCode::NOT_FOUND),
        },
        // TODO error message?
        None => Ok(http::StatusCode::BAD_REQUEST),
    }
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

    let routes = get_list.or(get_pixiecore_boot).or(post_trigger_boot);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    Ok(())
}
