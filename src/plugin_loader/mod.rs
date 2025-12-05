pub mod error;

use std::{fs, path::Path};

use abi_stable::library::RootModule;
use cli_dev::plugin::PluginModRef;
use tracing::{debug, error, info, warn};

use crate::{APP_NAME, config::data::MyConfig, plugin_loader::error::Error};

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
        debug!(target: APP_NAME, "Loading plugin from: {}", path.display());

        let module =
            PluginModRef::load_from_file(path).map_err(|er| Error::Load(er.to_string()))?;

        let Some(info) = module.get_info().map(|f| f()) else {
            return Err(Error::NotFound(path.to_string_lossy().to_string()));
        };
        debug!(target: APP_NAME, "  Loaded: {} v{}", info.name, info.version);
        debug!(target: APP_NAME, "  Description: {}", info.description);
        debug!(target: APP_NAME, "  Author: {}", info.author);

        self.plugins.push(LoadedPlugin {
            name: info.name.to_string(),
            module,
        });

        Ok(())
    }

    pub fn load_plugins_from_dir(&mut self, settings: &MyConfig) -> Result<(), Error> {
        let plugin_dir = crate::config::config_dir();

        if !plugin_dir.exists() {
            error!(target: APP_NAME,
                "Plugin directory '{}' does not exist",
                plugin_dir.to_string_lossy()
            );
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
                error!(target: APP_NAME, "Error loading plugin {}: {}", path.display(), e);
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

        let plugin_name = plugin
            .module
            .get_info()
            .map(|f| f().name)
            .ok_or_else(|| Error::PluginInterface(name.to_string(), "get_info".to_string()))?;

        let Some(result) = (plugin.module.execute()).map(|f| f(args_rvec.clone())) else {
            error!(
                target: APP_NAME,

                "failed to execute plugin {} with args: `{}`",
                plugin_name,
                args_rvec.join(",")

            );
            return Err(Error::Execution);
        };

        if !result.success {
            let binding = (plugin.module.get_info())
                .map(|f| f().name)
                .ok_or_else(|| Error::PluginInterface(name.to_string(), "get_info".to_string()))?;
            let name = binding.as_str();
            eprintln!("{}: {}", name, result.output);
            std::process::exit(result.exit_code);
        } else {
            let binding = (plugin.module.get_info())
                .map(|f| f().name)
                .ok_or_else(|| Error::PluginInterface(name.to_string(), "get_info".to_string()))?;
            let name = binding.as_str();
            println!("{}: {}", name, result.output);
            std::process::exit(exitcode::OK);
        }
    }

    pub fn list_plugins(&self) {
        if self.plugins.is_empty() {
            warn!(target: APP_NAME, "No plugins loaded");
            return;
        }

        info!(target: APP_NAME, "Available plugins:");
        for plugin in &self.plugins {
            let Some(info) = plugin.module.get_info().map(|f| f()) else {
                error!(target: APP_NAME, "loaded UNKNOWN plugin interface incomplete: get_info");
                std::process::exit(exitcode::UNAVAILABLE);
            };
            info!(target: APP_NAME,
                "  {} (v{}): {}",
                info.name, info.version, info.description
            );
        }
        std::process::exit(exitcode::OK);
    }

    pub fn show_help(&self, plugin_name: &str) -> Result<(), Error> {
        let plugin = self
            .plugins
            .iter()
            .find(|p| p.name == plugin_name)
            .ok_or_else(|| Error::NotFound(plugin_name.to_string()))?;

        let help =
            (plugin.module.get_help()).ok_or_else(|| Error::HelpFn(plugin_name.to_string()))?();
        println!("{}", help);

        std::process::exit(exitcode::OK);
    }
}
