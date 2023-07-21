use config::{Config, ConfigError, Environment};
use serde::{Deserialize, Serialize};

/// A struct defining default behaviour and deserialization
/// of values for configuring the application.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppConfig {
    #[serde(default = "_default_false")]
    profiling: bool,

    #[serde(default = "_default_false")]
    log_all: bool,

    #[serde(default = "_default_format")]
    time_format: String,

    #[serde(default = "_default_level")]
    log_level: String,

    #[serde(default = "_default_links_file")]
    routes: String,

    #[serde(default = "_default_false")]
    watch: bool,
}

impl AppConfig {
    /// Creates the application config based on environment variables
    pub fn build() -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(Environment::with_prefix("GOLINKS").try_parsing(true))
            .build()?
            .try_deserialize()
    }

    /// Gets a value indicating whether profiling should be
    /// enabled or not.
    pub fn profiling_enabled(&self) -> bool {
        self.profiling
    }

    /// Gets a value indicating whether the application
    /// should watch the links file for changes and reload
    /// the routes when it does.
    pub fn watch(&self) -> bool {
        self.watch
    }

    /// Sets the flag to enable/disable profiling
    pub fn enable_profiling(&mut self, val: bool) {
        self.profiling = val;
    }

    /// Gets a value specifying the format that times should appear
    /// with in the logs
    pub fn time_format(&self) -> &str {
        &self.time_format
    }

    /// Gets a value indicating the log level filter to use
    /// for logging
    pub fn level(&self) -> log::LevelFilter {
        match self.log_level.to_ascii_lowercase().as_str() {
            "off" => log::LevelFilter::Off,
            "error" => log::LevelFilter::Error,
            "warn" => log::LevelFilter::Warn,
            "info" => log::LevelFilter::Info,
            "debug" => log::LevelFilter::Debug,
            "trace" => log::LevelFilter::Trace,
            _ => panic!("Could not determine log level"),
        }
    }

    /// Gets the path to the file containing the links
    /// to use for redirection
    pub fn links_file(&self) -> &str {
        &self.routes
    }

    /// Gets a value indicating whether logs should be filtered to
    /// only be emitted from the service (false), or if all logs generated
    /// by library dependencies should be included
    pub fn log_all(&self) -> bool {
        self.log_all
    }
}

fn _default_false() -> bool {
    false
}

fn _default_format() -> String {
    "%Y-%m-%d - %H:%M:%S".to_string()
}

fn _default_level() -> String {
    "info".to_string()
}

fn _default_links_file() -> String {
    "links.yaml".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::collections::HashMap;
    use std::env::{self, VarError};

    fn with_env_vars<F>(expected_values: HashMap<&str, &str>, function: F) -> AppConfig
    where
        F: Fn() -> Result<AppConfig, config::ConfigError>,
    {
        let vars: HashMap<&str, Result<String, VarError>> = expected_values
            .iter()
            .map(|(&k, _)| (k, env::var(k)))
            .collect();

        for (k, v) in expected_values {
            env::set_var(k, v);
        }

        // Run closure
        let res = function();

        // Reset value
        for (k, v) in vars {
            if let Ok(val) = v {
                env::set_var(k, val)
            } else {
                env::remove_var(k)
            }
        }

        res.expect("Could not create configuration")
    }

    #[test]
    #[serial]
    fn test_enable_profiling() {
        let values = HashMap::from([("GOLINKS_PROFILING", "1")]);

        let configs = with_env_vars(values, AppConfig::build);

        assert!(configs.profiling_enabled());
        assert!(configs.profiling);
    }
}
