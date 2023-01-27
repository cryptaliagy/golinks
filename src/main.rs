mod models;
mod utils;

#[macro_use]
extern crate rocket;

use std::env;
use std::fs;
use std::path::PathBuf;

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
fn path(path: PathBuf, routes: &State<Routes>) -> Option<Redirect> {
    fetch_link(path.to_str().unwrap(), routes)
}

fn fetch_link(link: &str, routes: &State<Routes>) -> Option<Redirect> {
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
        .mount("/", routes![heartbeat, path])
        .register("/", catchers![not_found])
}

#[launch]
fn rocket() -> _ {
    println!("Application startup...");
    let enable_profiling = env::var("GOLINKS_PROFILING").unwrap_or("false".to_string()) != "false";
    let links_file = env::var("GOLINKS_ROUTES").unwrap_or("links.yaml".to_string());

    let config_file =
        fs::File::open(&links_file).expect(format!("Unable to open {}", &links_file).as_str());

    println!(
        "Finished reading routes from {}. Parsing into routes object...",
        &links_file
    );

    let routes: Routes = serde_yaml::from_reader(config_file)
        .expect(format!("Unable to parse {}", &links_file).as_str());

    println!("Finished parsing routes. Starting server...");
    build_rocket(routes, enable_profiling)
}

#[cfg(test)]
mod tests {
    use crate::models::Routes;

    use std::collections::HashMap;

    use rocket::http::{ContentType, Status};
    use rocket::local::blocking::Client;
    use rocket::{Build, Rocket};

    fn scaffold_rocket() -> Rocket<Build> {
        let route_map = HashMap::from([
            ("test".to_string(), "https://example.com".to_string()),
            ("e/x".to_string(), "https://example.com".to_string()),
        ]);
        let routes = Routes::with_routes(route_map);

        crate::build_rocket(routes, false)
    }

    fn scaffold_client() -> Client {
        Client::tracked(scaffold_rocket()).expect("valid rocket instance")
    }

    /// Test that the heartbeat endpoint returns a 200 status code and a JSON
    /// response.
    #[test]
    fn test_heartbeat() {
        let client = scaffold_client();
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
        let client = scaffold_client();
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
        let client = scaffold_client();
        let response = client.get("/e/x").dispatch();

        assert_eq!(response.status(), Status::TemporaryRedirect);
        assert_eq!(
            response.headers().get_one("Location"),
            Some("https://example.com")
        );
    }

    /// Test that a path that is not registered returns a 404 status code and a
    /// JSON response.
    #[test]
    fn test_path_not_found() {
        let client = scaffold_client();
        let response = client.get("/not-found").dispatch();

        assert_eq!(response.status(), Status::NotFound);
        assert_eq!(
            response.content_type(),
            Some(ContentType::new("application", "json"))
        );
    }
}
