use std::fs::File;

use futures_util::TryStreamExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use serde::Deserialize;
use serde::Serialize;
use serde_json;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

#[derive(Serialize, Deserialize)]
struct NetworkInterface {
    ip: String,
    mac: String,
}

#[derive(Serialize, Deserialize)]
struct Machine {
    // TODO
    id: u32,
    hostname: String,
    interfaces: Vec<NetworkInterface>,
    ipmi: NetworkInterface,
    status: Status,
}

#[derive(Serialize, Deserialize)]
enum Status {
    Idle,
    InPXEBoot,
}

#[derive(Serialize, Deserialize)]
struct MyObj {
    name: String,
    id: u32,
}

#[derive(Serialize, Deserialize)]
struct State {
    machines: Vec<Machine>,
}

fn load_machines() -> Result<State, Box<dyn std::error::Error>> {
    let f = File::open("machines.yml")?;
    let machines: Vec<Machine> = serde_yaml::from_reader(f)?;
    Ok(State { machines: machines })
}

async fn list_machines(state: Arc<RwLock<State>>) -> Result<Response<Body>, hyper::Error> {
    let state = &(*state.read().unwrap());
    let json = serde_json::to_string_pretty(state).unwrap();
    Ok(Response::new(json.into()))
}

async fn echo(
    state: Arc<RwLock<State>>,
    req: Request<Body>,
) -> Result<Response<Body>, hyper::Error> {
    // TODO better router?
    // maybe https://github.com/kardeiz/reset-router
    // maybe https://github.com/lambdax-x/rouste

    // /v1/list
    // /v1/boot/{mac}
    // /v1/triggerBoot/mac/{mac}/payload/{payload}

    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Ok(Response::new(Body::from(
            "Try POSTing data to /echo such as: `curl localhost:3000/echo -XPOST -d 'hello world'`",
        ))),
        (&Method::GET, "/v1/list") => list_machines(state).await,

        (&Method::POST, "/echo") => Ok(Response::new(req.into_body())),

        (&Method::POST, "/echo/uppercase") => {
            let chunk_stream = req.into_body().map_ok(|chunk| {
                chunk
                    .iter()
                    .map(|byte| byte.to_ascii_uppercase())
                    .collect::<Vec<u8>>()
            });
            Ok(Response::new(Body::wrap_stream(chunk_stream)))
        }

        (&Method::POST, "/echo/reversed") => {
            let whole_body = hyper::body::to_bytes(req.into_body()).await?;
            let reversed_body = whole_body.iter().rev().cloned().collect::<Vec<u8>>();
            Ok(Response::new(Body::from(reversed_body)))
        }

        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // We'll bind to 127.0.0.1:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    // TODO load yaml?
    let state = Arc::new(RwLock::new(load_machines()?));

    let service = make_service_fn(move |_| {
        let state = state.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                let state = state.clone();
                echo(state, req)
            }))
        }
    });

    let server = Server::bind(&addr).serve(service);

    server.await?;
    Ok(())
}
