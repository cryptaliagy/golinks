use std::collections::HashMap;
use std::time::SystemTime;

use chrono::Utc;

use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::serde::{Deserialize, Serialize};
use rocket::{Data, Request, Response};

use crate::config::AppConfig;
use crate::utils;

/// Fairing for timing requests.
pub struct RequestTimer {
    date_format: String,
}

/// Value stored in request-local state.
#[derive(Copy, Clone)]
pub struct TimerStart(Option<SystemTime>);

/// Struct for serializing a status message.
#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct StatusMessage {
    pub status: String,
    pub message: String,
}

/// Struct for deserializing routes
#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Routes {
    routes: HashMap<String, String>,
}

impl Routes {
    /// Fetches the URL for a given link.
    pub fn fetch(&self, link: &str) -> Option<String> {
        self.routes.get(link).cloned()
    }

    /// Creates a new `Routes` from an existing `HashMap`.
    #[allow(dead_code)]
    pub fn with_routes(routes: HashMap<String, String>) -> Self {
        Self { routes }
    }
}

impl RequestTimer {
    /// Creates a new `RequestTimer`.
    pub fn new(configs: &AppConfig) -> Self {
        Self {
            date_format: configs.time_format().to_string(),
        }
    }

    /// Gets the current time as a string in the configured format.
    pub fn now(&self) -> String {
        Utc::now().format(&self.date_format).to_string()
    }
}

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

    /// Print the elapsed time of the request.
    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut Response<'r>) {
        let start_time = req.local_cache(|| TimerStart(None));
        if let Some(Ok(duration)) = start_time.0.map(|st| st.elapsed()) {
            let formatted = utils::format_duration(duration);
            println!(
                "{time} | {method:^7} | {duration:>12} | {status} | \"{uri}\"",
                time = self.now(),
                method = req.method(),
                uri = req.uri(),
                duration = formatted,
                status = res.status().code,
            );

            res.set_header(Header::new("X-Request-Duration", formatted));
        }
    }
}
