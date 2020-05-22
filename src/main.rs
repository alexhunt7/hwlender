use std::fs::File;
use std::io;
use std::sync::Mutex;

use actix_web::{get, middleware, web, App, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Interface {
    ip: String,
    mac: String,
}

#[derive(Serialize, Deserialize)]
struct Machine {
    // TODO
    id: u32,
    hostname: String,
    interfaces: Vec<Interface>,
    ipmi: Interface,
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

#[get("/{id}/{name}/index.html")]
async fn index(info: web::Path<(u32, String)>) -> HttpResponse {
    HttpResponse::Ok().json(MyObj {
        name: info.1.clone(),
        id: info.0,
    })
}

#[get("/v1/list")]
async fn list_machines(
    data: web::Data<Mutex<AppState>>,
) -> Result<HttpResponse, std::sync::PoisonError<AppState>> {
    Ok(HttpResponse::Ok().json(&data.lock()?.machines))
}

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

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        //let machines = Vec::<Machine>::new();
        let f = File::open("machines.yml").unwrap();
        let machines: Vec<Machine> = serde_yaml::from_reader(f).unwrap();
        let app_state = web::Data::new(Mutex::new(AppState { machines: machines }));
        App::new()
            .wrap(middleware::Logger::default())
            .data(web::JsonConfig::default().limit(4096))
            .app_data(app_state)
            .service(list_machines)
            .service(index)
    })
    .bind("[::1]:8080")?
    .run()
    .await
}
