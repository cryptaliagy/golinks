mod models;
mod utils;

#[macro_use]
extern crate rocket;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

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

#[get("/<path..>")]
fn path(path: PathBuf, routes_map: &State<Routes>) -> Option<Redirect> {
    let mut current = Some(path.as_path());

    while current.is_some() {
        let forward = routes_map.fetch(current?.to_str().unwrap());

        if forward.is_some() {
            let afterimage = path.strip_prefix(current.unwrap()).unwrap();

            let afterimage = if afterimage == Path::new("") {
                afterimage.to_str().unwrap().to_string()
            } else {
                format!("/{}", afterimage.to_str().unwrap())
            };

            return forward.map(|x| x + &afterimage).map(Redirect::temporary);
        }
        current = current?.parent();
    }

    None
}

fn build_rocket(routes: Routes, enable_profiling: bool) -> Rocket<Build> {
    let ship = rocket::build();

    let ship = if enable_profiling {
        ship.attach(RequestTimer::new())
    } else {
        ship
    };

    ship.manage(routes)
        .mount("/", routes![heartbeat, path])
        .register("/", catchers![not_found])
}

#[launch]
fn rocket() -> _ {
    println!("Application startup...");
    let enable_profiling =
        env::var("GOLINKS_PROFILING").unwrap_or_else(|_| "false".to_string()) != "false";
    let links_file = env::var("GOLINKS_ROUTES").unwrap_or_else(|_| "links.yaml".to_string());

    let config_file =
        fs::File::open(&links_file).unwrap_or_else(|_| panic!("Unable to open {}", &links_file));

    println!(
        "Finished reading routes from {}. Parsing into routes object...",
        &links_file
    );

    let routes: Routes = serde_yaml::from_reader(config_file)
        .unwrap_or_else(|_| panic!("Unable to parse {}", &links_file));

    println!("Finished parsing routes. Starting server...");
    build_rocket(routes, enable_profiling)
}

#[cfg(test)]
mod tests {
    use crate::models::Routes;

    use std::collections::HashMap;
    use std::time::Duration;

    use rocket::http::{ContentType, Status};
    use rocket::local::blocking::{Client, LocalResponse};
    use rocket::{Build, Rocket};

    fn scaffold_rocket(enable_profiling: bool) -> Rocket<Build> {
        let route_map = HashMap::from([
            ("test".to_string(), "https://example.com".to_string()),
            ("e/x".to_string(), "https://example.com".to_string()),
            ("e".to_string(), "https://differentexample.com".to_string()),
        ]);
        let routes = Routes::with_routes(route_map);

        crate::build_rocket(routes, enable_profiling)
    }

    fn scaffold_client(enable_profiling: bool) -> Client {
        Client::tracked(scaffold_rocket(enable_profiling)).expect("valid rocket instance")
    }

    fn duration_from_response(response: &LocalResponse<'_>) -> Duration {
        // Parse the header in form of "X-Request-Duration: 12.34 <unit>"
        // where <unit> is either "s", "ms" or "µs" into a Duration.
        // If the unit is ms or µs, the numeric value is less than 1000.
        let duration_header = response.headers().get_one("X-Request-Duration").unwrap();
        let duration = duration_header
            .split_whitespace()
            .next()
            .unwrap()
            .split_terminator('.') // Ignore the decimal part
            .next()
            .unwrap()
            .parse::<u64>()
            .unwrap();

        let unit = duration_header.split_whitespace().last().unwrap();

        match unit {
            "s" => std::time::Duration::from_secs(duration),
            "ms" => std::time::Duration::from_millis(duration),
            "μs" => std::time::Duration::from_micros(duration),
            _ => panic!("Unknown unit {}", unit),
        }
    }

    /// Test that the heartbeat endpoint returns a 200 status code and a JSON
    /// response.
    #[test]
    fn test_heartbeat() {
        let client = scaffold_client(false);
        let response = client.get("/heartbeat").dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.content_type(),
            Some(ContentType::new("application", "json"))
        );
    }

    /// Test that some registered path with a single path element
    /// returns a 307 status code and a Location
    /// header with the correct value.
    #[test]
    fn test_path_single() {
        let client = scaffold_client(false);
        let response = client.get("/test").dispatch();

        assert_eq!(response.status(), Status::TemporaryRedirect);
        assert_eq!(
            response.headers().get_one("Location"),
            Some("https://example.com")
        );
    }

    /// Test that some registered path with multiple path elements
    /// returns a 307 status code and a Location
    /// header with the correct value.
    #[test]
    fn test_path_multi() {
        let client = scaffold_client(false);
        let response = client.get("/e/x").dispatch();

        assert_eq!(response.status(), Status::TemporaryRedirect);
        assert_eq!(
            response.headers().get_one("Location"),
            Some("https://example.com")
        );
    }

    /// Test that accessing some descendant of a registered path
    /// redirects you to that path + the added route information. i.e.
    /// if /e/x is registered as https://example.com/, then /e/x/ample
    /// should redirect to https://example.com/ample
    #[test]
    fn test_registered_ancestor() {
        let client = scaffold_client(false);
        let response = client.get("/e/x/ample").dispatch();

        assert_eq!(response.status(), Status::TemporaryRedirect);
        assert_eq!(
            response.headers().get_one("Location"),
            Some("https://example.com/ample")
        );
    }

    /// Test that precedence is done by finding the first matching ancestor path
    #[test]
    fn test_first_registered_ancestor() {
        let client = scaffold_client(false);
        let response = client.get("/e/l/ample").dispatch();

        assert_eq!(response.status(), Status::TemporaryRedirect);
        assert_eq!(
            response.headers().get_one("Location"),
            Some("https://differentexample.com/l/ample")
        );

        let response = client.get("/e/x/ample").dispatch();

        assert_eq!(response.status(), Status::TemporaryRedirect);
        assert_eq!(
            response.headers().get_one("Location"),
            Some("https://example.com/ample")
        );
    }

    /// Test that a path that is not registered returns a 404 status code and a
    /// JSON response.
    #[test]
    fn test_path_not_found() {
        let client = scaffold_client(false);
        let response = client.get("/not-found").dispatch();

        assert_eq!(response.status(), Status::NotFound);
        assert_eq!(
            response.content_type(),
            Some(ContentType::new("application", "json"))
        );
    }

    /// Test that a multipath that is not registered returns a 404 status code and a
    /// JSON response.
    #[test]
    fn test_multipath_not_found() {
        let client = scaffold_client(false);
        let response = client.get("/not/found/at/all").dispatch();

        assert_eq!(response.status(), Status::NotFound);
        assert_eq!(
            response.content_type(),
            Some(ContentType::new("application", "json"))
        );
    }

    /// Test that a request returns a header with the duration of the request if
    /// profiling is enabled, and that the .
    #[test]
    fn test_profiling() {
        let client = scaffold_client(true);
        let response = client.get("/heartbeat").dispatch();

        let max_duration = std::time::Duration::from_micros(500);

        assert_eq!(response.status(), Status::Ok);
        assert!(response.headers().get_one("X-Request-Duration").is_some());

        let duration = duration_from_response(&response);

        assert!(duration < max_duration);
    }
}
