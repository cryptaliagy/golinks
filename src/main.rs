#[macro_use]
extern crate rocket;

use std::fs;
use std::path::{Path, PathBuf};

use fern::colors::{Color, ColoredLevelConfig};
use log::{debug, error, info};

use rocket::fairing::AdHoc;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::{Build, Request, Rocket, State};
use tokio::sync::mpsc::{channel, Receiver, Sender};

use notify::{Event, RecommendedWatcher, Watcher};

use golinks::config::AppConfig;
use golinks::models::{RequestTimer, Routes, StatusMessage};

#[catch(404)]
fn not_found(req: &Request) -> Json<StatusMessage> {
    Json(StatusMessage {
        status: "error".to_string(),
        message: format!("'{}' is not a known link.", req.uri()),
    })
}

/// A route that returns a 200 status code and a short json message. This is used to
/// confirm that the web server is receiving requests but not performing any specific
/// operation.
#[get("/heartbeat")]
async fn heartbeat() -> Json<StatusMessage> {
    Json(StatusMessage {
        status: "ok".to_string(),
        message: "The server is running".to_string(),
    })
}

/// A debug route (this only compiles when using the debug profile, so it doesn't exist in
/// production) to retrieve the current running configuration of the application. This shows
/// how to retrieve state managed by the application
#[cfg(debug_assertions)]
#[get("/config")]
async fn show_configs(configs: &State<AppConfig>) -> Json<&AppConfig> {
    Json(configs)
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

/// Constructs the rocket that will be used based on the configuration passed to this function.
/// This will then be used by the `rocket()` function to launch the application.
///
/// The configuration passed in will be made available to routes using the `&State<AppConfig>`
/// type as a parameter in the function.
fn build_rocket(configs: AppConfig, registered_routes: Routes) -> Rocket<Build> {
    info!("Building rocket...");
    let ship = rocket::build();

    // All fairings should be attached below here and before the routes
    // vector is constructed. This ensures that the logging fairings are the
    // last ones to be executed.
    let ship = if configs.profiling_enabled() {
        debug!("Profiling enabled! Attaching fairing...");
        ship.attach(RequestTimer::default())
    } else {
        ship
    };

    #[allow(unused_mut)]
    let mut routes = routes![heartbeat, path];

    // Since `show_configs` doesn't exist when compiling the release profile,
    // we need to use the same macro under this scope to prevent the scope from being
    // compiled in release mode. This is useful if there's any routes that would
    // be a security risk in production but are useful to have in development.
    //
    // If we remove the `#[cfg(debug_assertions)]` macro from the `show_configs`
    // route, we could still add the route only conditionally by using
    // `if cfg!(debug_assertions) {}`
    #[cfg(debug_assertions)]
    {
        debug!("Debug profile enabled! Adding debug routes to routes vector...");
        let mut debug_routes = routes![show_configs];

        routes.append(&mut debug_routes);
    };

    debug!("Mounting state and routes...");
    ship.attach(AdHoc::on_ignite("logging ignite", |rocket| async {
        info!("Ignition complete! Launching rocket...");
        rocket
    }))
    .attach(AdHoc::on_liftoff("logging liftoff", |_| {
        Box::pin(async { info!("Launch complete! Service 'golinks' is running") })
    }))
    .attach(AdHoc::on_shutdown("logging shutdown", |_| {
        Box::pin(async { info!("Shutting down service...") })
    }))
    .manage(configs)
    .manage(registered_routes)
    .mount("/", routes)
    .register("/", catchers![not_found])
}

async fn create_rocket_from(configs: AppConfig) -> Rocket<Build> {
    info!("Building routes...");
    let links_file = configs.links_file();
    let config_file =
        fs::File::open(links_file).unwrap_or_else(|_| panic!("Unable to open {}", links_file));

    debug!("Finished reading data from {}, parsing...", links_file);

    let routes: Routes = serde_yaml::from_reader(config_file)
        .unwrap_or_else(|_| panic!("Unable to parse {}", links_file));

    debug!("Finished parsing {}", links_file);
    let ship = build_rocket(configs.clone(), routes);

    info!("Rocket build complete!");
    ship
}

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<Event>)> {
    let (tx, rx) = channel(1);

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let watcher = RecommendedWatcher::new(
        move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                debug!("Received event: {:?}", event);
                if event.kind.is_modify() && !tx.is_closed() {
                    tx.blocking_send(event).unwrap_or_else(|err| {
                        error!("Could not send event: {}", err);
                    });
                }
            }
        },
        notify::Config::default().with_compare_contents(true),
    )?;

    Ok((watcher, rx))
}

async fn shutdown_on_event(shutdown: rocket::Shutdown, configs: AppConfig, tx: Sender<bool>) {
    let (mut watcher, mut rx) = async_watcher().unwrap();

    if configs.watch() {
        info!(
            "File watching is enabled! Watching {} for changes...",
            configs.links_file()
        );

        watcher
            .watch(
                Path::new(configs.links_file()),
                notify::RecursiveMode::NonRecursive,
            )
            .unwrap_or_else(|_| error!("Could not watch links file"));

        if let Some(event) = rx.recv().await {
            debug!("Received shutdown event: {:?}", event);
            info!("Links file changed! Requesting shutdown...");
            // Notify the main thread that we should reload
            tx.send(true).await.unwrap();

            shutdown.notify();
        }
    }
}

#[rocket::main]
async fn main() {
    #[cfg(debug_assertions)]
    println!("Building configuration...");
    let configs = AppConfig::build().expect("Could not build configuration from environment");

    #[cfg(debug_assertions)]
    println!("Building logger...");

    let date_fmt: String = configs.time_format().to_string();

    let colors = ColoredLevelConfig::new()
        .debug(Color::Cyan)
        .info(Color::Green)
        .warn(Color::Yellow)
        .error(Color::Red);

    let mut log_config = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}]\t{}",
                chrono::Utc::now().format(&date_fmt),
                record.target(),
                colors.color(record.level()),
                message,
            ))
        })
        .level(configs.level())
        .chain(std::io::stdout());

    if !configs.log_all() {
        log_config = log_config.filter(|metadata| metadata.target().starts_with("golinks"))
    }

    log_config.apply().unwrap();

    debug!("Logger configuration finished!");

    loop {
        let configs = configs.clone();

        println!(
            r#"
  _____       _      _       _        
 / ____|     | |    (_)     | |       
| |  __  ___ | |     _ _ __ | | _____ 
| | |_ |/ _ \| |    | | '_ \| |/ / __|
| |__| | (_) | |____| | | | |   <\__ \
 \_____|\___/|______|_|_| |_|_|\_\___/
 ==================================================                                     
        "#
        );

        info!("Initializing application...");
        let ship = create_rocket_from(configs.clone())
            .await
            .ignite()
            .await
            .unwrap();

        let shutdown = ship.shutdown();

        let (tx, mut rx) = channel(1);

        let watcher_task = rocket::tokio::spawn(async move {
            shutdown_on_event(shutdown, configs, tx).await;
        });

        ship.launch().await.unwrap();

        // We send the reload signal before the shutdown one,
        // so if the reload receiver is empty we know that
        // it was not because of changes to the links file
        if rx.try_recv().is_err() {
            break;
        }

        debug!("Shutting down watcher...");
        watcher_task.abort();
        info!("Requesting service reload...\n\n\n")
    }

    info!("Service 'golinks' successfully shut down");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::time::Duration;

    use rocket::http::{ContentType, Status};
    use rocket::local::blocking::{Client, LocalResponse};

    /// Creates a test client using the specified configuration
    fn scaffold_client_with(configs: AppConfig) -> Client {
        let route_map = HashMap::from([
            ("test".to_string(), "https://example.com".to_string()),
            ("e/x".to_string(), "https://example.com".to_string()),
            ("e".to_string(), "https://differentexample.com".to_string()),
        ]);
        let routes = Routes::with_routes(route_map);
        Client::tracked(build_rocket(configs, routes)).expect("valid rocket instance")
    }

    /// Creates a test client using the default configuration
    fn scaffold_client() -> Client {
        scaffold_client_with(AppConfig::default())
    }

    /// Given a local response generated from a test client, parses the X-Request-Duration header
    /// injected by the request timer faring and returns the integer portion as a duration
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
        let client = scaffold_client();
        let response = client.get("/heartbeat").dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.content_type(),
            Some(ContentType::new("application", "json"))
        );
    }

    /// Test that a request returns a header with the duration of the request if
    /// profiling is enabled, and that the duration is smaller than 1 ms (if debug)
    /// or smaller than 200 μs (if release).
    #[test]
    fn test_profiling() {
        let mut configs = AppConfig::default();
        configs.enable_profiling(true);

        let client = scaffold_client_with(configs);
        let response = client.get("/heartbeat").dispatch();

        let max_duration = if cfg!(debug_assertions) {
            std::time::Duration::from_micros(1000)
        } else {
            std::time::Duration::from_micros(200)
        };

        assert_eq!(response.status(), Status::Ok);
        assert!(response.headers().get_one("X-Request-Duration").is_some());

        let duration = duration_from_response(&response);

        assert!(duration < max_duration);
    }

    /// Test that the config route returns the accurate configuration as passed in
    /// to the scaffolding.
    ///
    /// This test only runs when `--release` is NOT passed into `cargo test`.
    #[cfg(debug_assertions)]
    #[test]
    fn test_configs() {
        let mut configs = AppConfig::default();
        configs.enable_profiling(true);

        let cloned = configs.clone();

        assert_eq!(configs, cloned);

        let client = scaffold_client_with(cloned);
        let response = client.get("/config").dispatch();

        assert_eq!(response.status(), Status::Ok);

        let returned_configs: AppConfig = response.into_json().unwrap();

        assert_eq!(configs, returned_configs);
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

    /// Test that accessing some descendant of a registered path
    /// redirects you to that path + the added route information. i.e.
    /// if /e/x is registered as https://example.com/, then /e/x/ample
    /// should redirect to https://example.com/ample
    #[test]
    fn test_registered_ancestor() {
        let client = scaffold_client();
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
        let client = scaffold_client();
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
        let client = scaffold_client();
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
        let client = scaffold_client();
        let response = client.get("/not/found/at/all").dispatch();

        assert_eq!(response.status(), Status::NotFound);
        assert_eq!(
            response.content_type(),
            Some(ContentType::new("application", "json"))
        );
    }
}
