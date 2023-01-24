#[macro_use]
extern crate rocket;

use std::collections::HashMap;
use std::env;
use std::fs;
use std::time::{Duration, SystemTime};

use chrono::Utc;

use rocket::fairing::{Fairing, Info, Kind};
use rocket::response::Redirect;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{Data, Request, Response, State};

/// Fairing for timing requests.
struct RequestTimer;

/// Value stored in request-local state.
#[derive(Copy, Clone)]
struct TimerStart(Option<SystemTime>);

#[rocket::async_trait]
impl Fairing for RequestTimer {
    fn info(&self) -> Info {
        Info {
            name: "Request Timer",
            kind: Kind::Request | Kind::Response,
        }
    }

    /// Stores the start time of the request in request-local state.
    async fn on_request(&self, request: &mut Request<'_>, _: &mut Data<'_>) {
        // Store a `TimerStart` instead of directly storing a `SystemTime`
        // to ensure that this usage doesn't conflict with anything else
        // that might store a `SystemTime` in request-local cache.
        request.local_cache(|| TimerStart(Some(SystemTime::now())));
    }

    /// Adds a header to the response indicating how long the server took to
    /// process the request.
    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut Response<'r>) {
        let start_time = req.local_cache(|| TimerStart(None));
        if let Some(Ok(duration)) = start_time.0.map(|st| st.elapsed()) {
            println!(
                "{time} | {method:^7} | {duration:>12} | {status} | \"{uri}\"",
                time = Utc::now().format("%Y-%m-%d - %H:%M:%S"),
                method = req.method(),
                uri = req.uri(),
                duration = format_duration(duration),
                status = res.status().code,
            );
        }
    }
}

/// Struct for serializing a status message.
#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct StatusMessage {
    status: String,
    message: String,
}

/// Struct for deserializing routes
#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Routes {
    routes: HashMap<String, String>,
}

/// Formats a `Duration` as a string.
fn format_duration(duration: Duration) -> String {
    let millis = duration.as_millis();
    let micros = duration.subsec_micros() as u128 - millis * 1000;
    let nanos = duration.subsec_nanos() as u128 - micros * 1000;
    if millis > 0 {
        format!("{}.{} ms", millis, micros)
    } else {
        format!("{}.{} μs", micros, nanos)
    }
}

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
        .routes
        .get(link)
        .map(|url| Redirect::temporary(url.clone()))
}

#[launch]
fn rocket() -> _ {
    let enable_profiling = env::var("GOLINKS_PROFILING").unwrap_or("false".to_string()) != "false";
    let links_file = env::var("GOLINKS_ROUTES").unwrap_or("links.yaml".to_string());

    let config_file =
        fs::File::open(&links_file).expect(format!("Unable to open {}", &links_file).as_str());

    let routes: Routes = serde_yaml::from_reader(config_file)
        .expect(format!("Unable to parse {}", &links_file).as_str());

    let ship = if enable_profiling {
        rocket::build().attach(RequestTimer {})
    } else {
        rocket::build()
    };

    ship.manage(routes)
        .mount("/", routes![heartbeat, link])
        .register("/", catchers![not_found])
}
