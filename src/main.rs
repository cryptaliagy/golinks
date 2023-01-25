mod models;
mod utils;

#[macro_use]
extern crate rocket;

use std::env;
use std::fs;

use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::{Build, Request, Rocket, State};

use crate::models::{RequestTimer, Routes, StatusMessage};

#[catch(404)]
fn not_found(req: &Request) -> Json<StatusMessage> {
    Json(StatusMessage {
        status: "error".to_string(),
        message: format!("'{}' is not a known link.", req.uri()),
    })
}

#[get("/heartbeat")]
fn heartbeat() -> Json<StatusMessage> {
    Json(StatusMessage {
        status: "ok".to_string(),
        message: "The server is running".to_string(),
    })
}

#[get("/<link>")]
fn link(link: &str, routes: &State<Routes>) -> Option<Redirect> {
    routes
        .inner()
        .fetch(link)
        .map(|url| Redirect::temporary(url))
}

fn build_rocket(routes: Routes, enable_profiling: bool) -> Rocket<Build> {
    let ship = rocket::build();

    let ship = if enable_profiling {
        ship.attach(RequestTimer::new())
    } else {
        ship
    };

    ship.manage(routes)
        .mount("/", routes![heartbeat, link])
        .register("/", catchers![not_found])
}

#[launch]
fn rocket() -> _ {
    let enable_profiling = env::var("GOLINKS_PROFILING").unwrap_or("false".to_string()) != "false";
    let links_file = env::var("GOLINKS_ROUTES").unwrap_or("links.yaml".to_string());

    let config_file =
        fs::File::open(&links_file).expect(format!("Unable to open {}", &links_file).as_str());

    let routes: Routes = serde_yaml::from_reader(config_file)
        .expect(format!("Unable to parse {}", &links_file).as_str());

    build_rocket(routes, enable_profiling)
}
