//use std::fs::File;
//use std::io;
//use std::sync::Mutex;

use futures_util::TryStreamExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use serde::Deserialize;
use serde::Serialize;
use std::convert::Infallible;
use std::net::SocketAddr;

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
struct AppState {
    machines: Vec<Machine>,
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

//#[get("/{id}/{name}/index.html")]
//async fn index(info: web::Path<(u32, String)>) -> HttpResponse {
//    HttpResponse::Ok().json(MyObj {
//        name: info.1.clone(),
//        id: info.0,
//    })
//}
//
//#[get("/v1/list")]
//async fn list_machines(
//    data: web::Data<Mutex<AppState>>,
//) -> Result<HttpResponse, std::sync::PoisonError<AppState>> {
//    Ok(HttpResponse::Ok().json(&data.lock()?.machines))
//}

//#[derive(Debug)]
//enum Error {
//    YamlError(serde_yaml::Error),
//    IoError(io::Error),
//}
//
//impl fmt::Display for Error {
//    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//        match *self {
//            YamlError => f.write_str("Error parsing yaml"),
//            IoError => f.write_str("Error with IO"),
//        }
//    }
//}
//
//impl std::error::Error for Error {
//    fn description(&self) -> &str {
//        match *self {
//            YamlError => "Error parsing yaml",
//            IoError => "Error with IO",
//        }
//    }
//    //fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
//
//    //}
//}
//
//impl From<serde_yaml::Error> for Error {
//    fn from(e: serde_yaml::Error) -> Self {
//        Error::YamlError(e)
//    }
//}
//
//impl From<io::Error> for Error {
//    fn from(e: io::Error) -> Self {
//        Error::IoError(e)
//    }
//}

//#[actix_rt::main]
//async fn main() -> std::io::Result<()> {
//    HttpServer::new(move || {
//        //let machines = Vec::<Machine>::new();
//        let f = File::open("machines.yml").unwrap();
//        let machines: Vec<Machine> = serde_yaml::from_reader(f).unwrap();
//        let app_state = web::Data::new(Mutex::new(AppState { machines: machines }));
//        App::new()
//            .wrap(middleware::Logger::default())
//            .data(web::JsonConfig::default().limit(4096))
//            .app_data(app_state)
//            .service(list_machines)
//            .service(index)
//    })
//    .bind("[::1]:8080")?
//    .run()
//    .await
//}

async fn hello_world(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new("Hello, World!".into()))
}

async fn echo(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Ok(Response::new(Body::from(
            "Try POSTing data to /echo such as: `curl localhost:3000/echo -XPOST -d 'hello world'`",
        ))),

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
async fn main() {
    // We'll bind to 127.0.0.1:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(echo)) });

    let server = Server::bind(&addr).serve(service);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
