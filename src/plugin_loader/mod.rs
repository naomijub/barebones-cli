pub mod error;

use abi_stable::library::RootModule;
use cli_dev::{
    logging::{log_debug, log_error, log_info},
    plugin::PluginModRef,
};
use std::fs;
use std::path::Path;

use crate::plugin_loader::error::Error;
use crate::{config::data::MyConfig, logger::log_warn};

pub struct PluginManager {
    plugins: Vec<LoadedPlugin>,
}

struct LoadedPlugin {
    name: String,
    module: PluginModRef,
}

impl PluginManager {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    fn load_plugin(&mut self, path: &Path) -> Result<(), Error> {
        crate::logger::log_debug(format!("Loading plugin from: {}", path.display()));

        let module =
            PluginModRef::load_from_file(path).map_err(|er| Error::Load(er.to_string()))?;

        let Some(info) = module.get_info().map(|f| f()) else {
            return Err(Error::NotFound(path.to_string_lossy().to_string()));
        };
        log_info(module, format!("  Loaded: {} v{}", info.name, info.version));
        log_debug(module, format!("  Description: {}", info.description));
        log_debug(module, format!("  Author: {}", info.author));

        self.plugins.push(LoadedPlugin {
            name: info.name.to_string(),
            module,
        });

        Ok(())
    }

    pub fn load_plugins_from_dir(&mut self, settings: &MyConfig) -> Result<(), Error> {
        let plugin_dir = crate::config::config_dir();

        if !plugin_dir.exists() {
            crate::logger::log_error(format!(
                "Plugin directory '{}' does not exist",
                plugin_dir.to_string_lossy()
            ));
            return Ok(());
        }

        for entry in fs::read_dir(plugin_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Load .so on Linux, .dylib on macOS, .dll on Windows
            #[cfg(target_os = "linux")]
            let is_plugin = path.extension().is_some_and(|e| e == "so");

            #[cfg(target_os = "macos")]
            let is_plugin = path.extension().is_some_and(|e| e == "dylib");

            #[cfg(target_os = "windows")]
            let is_plugin = path.extension().is_some_and(|e| e == "dll");

            let is_expected = settings.plugins.contains(
                &path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
            ) || settings.plugins.contains(
                &path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .strip_prefix("lib")
                    .unwrap_or_default()
                    .to_string(),
            );

            if is_plugin
                && is_expected
                && let Err(e) = self.load_plugin(&path)
            {
                crate::logger::log_error(format!("Error loading plugin {}: {}", path.display(), e));
            }
        }

        Ok(())
    }

    pub fn execute_plugin(&self, name: &str, args: Vec<String>) -> Result<(), Error> {
        let plugin = self
            .plugins
            .iter()
            .find(|p| p.name == name)
            .ok_or_else(|| Error::NotFound(format!("Plugin '{}' not found", name)))?;

        let args_rvec: abi_stable::std_types::RVec<_> =
            args.into_iter().map(|s| s.into()).collect();

        let Some(result) = (plugin.module.execute()).map(|f| f(args_rvec.clone())) else {
            log_error(
                plugin.module,
                format!(
                    "failed to execute plugin with args: `{}`",
                    args_rvec.join(",")
                ),
            );
            return Err(Error::Execution);
        };

        if !result.success {
            log_error(plugin.module, result.output);
            std::process::exit(result.exit_code);
        } else {
            log_info(plugin.module, result.output);
            std::process::exit(exitcode::OK);
        }
    }

    pub fn list_plugins(&self) {
        if self.plugins.is_empty() {
            log_warn("No plugins loaded");
            return;
        }

        crate::logger::log_info("Available plugins:");
        for plugin in &self.plugins {
            let info = (plugin.module.get_info()).unwrap()();
            crate::logger::log_info(format!(
                "  {} (v{}): {}",
                info.name, info.version, info.description
            ));
        }
    }

    pub fn show_help(&self, plugin_name: &str) -> Result<(), Error> {
        let plugin = self
            .plugins
            .iter()
            .find(|p| p.name == plugin_name)
            .ok_or_else(|| Error::NotFound(plugin_name.to_string()))?;

        let help = (plugin.module.get_help()).unwrap()();
        println!("{}", help);

        Ok(())
    }
}
