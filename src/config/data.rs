use std::fmt::Display;

use semver::Version;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::logger::log_error;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct MyConfig {
    name: String,
    pub is_machine: bool,
    pub should_auto_update: bool,
    #[serde(
        deserialize_with = "deserialize_version",
        serialize_with = "serialize_version"
    )]
    version: Version,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub plugins: Vec<String>,
}

pub fn serialize_version<S>(value: &Version, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(value.to_string().as_str())
}

/// Deserialize a Unix epoch timestamp into a `chrono::NaiveDateTime`.
pub fn deserialize_version<'de, D>(d: D) -> Result<Version, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(d)?;

    Version::parse(&buf).map_err(|err| serde::de::Error::custom(err.to_string()))
}

impl Default for MyConfig {
    fn default() -> Self {
        #[expect(
            clippy::unwrap_used,
            reason = "We guarantee this version works in compile time"
        )]
        Self {
            name: "Julia Naomi".to_string(),
            is_machine: false,
            should_auto_update: true,
            version: Version::parse("1.0.0").unwrap(),
            plugins: Vec::new(),
        }
    }
}

impl Display for MyConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let toml_config = toml::to_string_pretty(&self).map_err(|_err| std::fmt::Error)?;
        writeln!(f, "{toml_config}")
    }
}

impl MyConfig {
    pub fn contains_plugins(&self, plugin: &String) {
        let contains = self.plugins.contains(plugin);
        if !contains {
            log_error(format!("{} plugin not available", plugin));
            std::process::exit(exitcode::UNAVAILABLE);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_config() {
        let myconfig = MyConfig::default();

        let json = serde_json::to_string(&myconfig).unwrap();
        assert_eq!(
            json,
            "{\"name\":\"Julia Naomi\",\"is_machine\":false,\"should_auto_update\":true,\"version\":\"1.0.0\"}"
        );

        let config: MyConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, myconfig);
    }
}
