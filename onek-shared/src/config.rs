use std::error::Error;
use std::fs;
use toml::Table;

pub struct Config {
    table: toml::Table,
    service: String,
    err: Option<Box<dyn Error>>,
}

impl Config {
    /// service is the name of the config section to check for default overrides.
    pub fn load(service: &str) -> Config {
        let service = service.to_string();
        match fs::read_to_string("config.toml") {
            Ok(contents) => match contents.parse::<Table>() {
                Ok(table) => Config {
                    table,
                    service,
                    err: None,
                },
                Err(err) => Config {
                    table: "".parse::<Table>().unwrap(),
                    service,
                    err: Some(Box::new(err)),
                },
            },
            Err(err) => Config {
                table: "".parse::<Table>().unwrap(),
                service,
                err: Some(Box::new(err)),
            },
        }
    }

    /// If there was an error reading the config file it will be returned here and the
    /// config will be empty. This is awkward but services typically report errors via
    /// logging and logging is initialized after the config loads.
    pub fn error(&self) -> &Option<Box<dyn Error>> {
        &self.err
    }

    pub fn str_value(&self, key: &str, default: &str) -> String {
        self.section_str_value(&self.service, key)
            .unwrap_or_else(|| self.section_str_value("default", key).unwrap_or(default.to_string()))
    }

    fn section_str_value(&self, section: &str, key: &str) -> Option<String> {
        self.table
            .get(section)
            .and_then(|value| match value {
                toml::Value::Table(table) => Some(table),
                _ => None,
            })
            .and_then(|table| table.get(key))
            .and_then(|value| match value.as_str() {
                Some(s) => Some(s.to_string()),
                None => None, // awkward case: found value but it wasn't a string
            })
    }
}
