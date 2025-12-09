pub mod error;

use std::{fmt, fs, path::Path};

use abi_stable::library::RootModule;
use cli_dev::plugin::PluginModRef;
use tracing::{debug, error, instrument, trace, warn};

use crate::{
    APP_NAME,
    config::data::MyConfig,
    plugin_loader::{
        error::Error,
        metric::{LOAD_ALL_PLUGINS_METRIC, LOAD_PLUGIN_BY_NAME_METRIC, PLUGIN_EXECUTION_METRIC},
    },
};

#[derive(Debug)]
pub struct PluginManager {
    plugins: Vec<LoadedPlugin>,
}

struct LoadedPlugin {
    name: String,
    module: PluginModRef,
}

impl fmt::Debug for LoadedPlugin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoadedPlugin")
            .field("name", &self.name)
            .field("has_execute", &self.module.execute().is_some())
            .field("has_get_info", &self.module.get_info().is_some())
            .field("has_get_help", &self.module.get_help().is_some())
            .finish()
    }
}

impl PluginManager {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub const fn count(&self) -> usize {
        self.plugins.len()
    }

    #[instrument(skip(self))]
    fn load_plugin(&mut self, path: &Path) -> Result<(), Error> {
        let start = &*LOAD_PLUGIN_BY_NAME_METRIC;
        debug!(target: APP_NAME, "Loading plugin from: {}", path.display());

        let module = PluginModRef::load_from_file(path)
            .inspect_err(|_| {
                LOAD_PLUGIN_BY_NAME_METRIC.record(false, path.to_string_lossy().to_string())
            })
            .map_err(|er| Error::Load(er.to_string()))?;

        let Some(info) = module.get_info().map(|f| f()) else {
            LOAD_PLUGIN_BY_NAME_METRIC.record(false, path.to_string_lossy().to_string());
            return Err(Error::NotFound(path.to_string_lossy().to_string()));
        };
        debug!(target: APP_NAME, "  Loaded: {} v{}", info.name, info.version);
        debug!(target: APP_NAME, "  Description: {}", info.description);
        debug!(target: APP_NAME, "  Author: {}", info.author);

        self.plugins.push(LoadedPlugin {
            name: info.name.to_string(),
            module,
        });

        start.record(true, info.name.to_string());
        Ok(())
    }

    #[instrument(skip_all)]
    pub fn load_plugins_from_dir(&mut self, settings: &MyConfig) -> Result<(), Error> {
        let start = &*LOAD_ALL_PLUGINS_METRIC;
        let plugin_dir = crate::config::config_dir();

        if !plugin_dir.exists() {
            error!(target: APP_NAME,
                "Plugin directory '{}' does not exist",
                plugin_dir.to_string_lossy()
            );
            start.record(false);
            return Ok(());
        }

        for entry in
            fs::read_dir(plugin_dir).inspect_err(|_| LOAD_ALL_PLUGINS_METRIC.record(false))?
        {
            let entry = entry.inspect_err(|_| LOAD_ALL_PLUGINS_METRIC.record(false))?;
            let path = entry.path();

            // Load .so on Linux, .dylib on macOS, .dll on Windows
            #[cfg(target_os = "linux")]
            let is_plugin = path.extension().is_some_and(|e| e == "so");

            #[cfg(target_os = "macos")]
            let is_plugin = path.extension().is_some_and(|e| e == "dylib");

            #[cfg(target_os = "windows")]
            let is_plugin = path.extension().is_some_and(|e| e == "dll");

            if cfg!(target_os = "linux") {
                trace!("OS = Linux");
            } else if cfg!(target_os = "macos") {
                trace!("OS = macOS");
            } else if cfg!(target_os = "windows") {
                trace!("OS = Windows");
            } else {
                trace!("OS = Unknown OS {}", std::env::consts::OS);
            }

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

        start.record(true);
        Ok(())
    }

    #[instrument]
    pub fn execute_plugin(&self, name: &str, args: Vec<String>) -> Result<(), Error> {
        let start = &*PLUGIN_EXECUTION_METRIC;
        let execution_name = name.to_string();
        trace!(target: APP_NAME, "execute_plugin");
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
            .ok_or_else(|| Error::PluginInterface(name.to_string(), "get_info".to_string()))
            .inspect_err(|_| PLUGIN_EXECUTION_METRIC.record(false, execution_name.clone()))?;

        let Some(result) = (plugin.module.execute()).map(|f| f(args_rvec.clone())) else {
            error!(
                target: APP_NAME,

                "failed to execute plugin {} with args: `{}`",
                plugin_name,
                args_rvec.join(",")

            );
            start.record(false, execution_name);
            return Err(Error::Execution);
        };

        if !result.success {
            let binding = (plugin.module.get_info())
                .map(|f| f().name)
                .ok_or_else(|| Error::PluginInterface(name.to_string(), "get_info".to_string()))
                .inspect_err(|_| PLUGIN_EXECUTION_METRIC.record(false, execution_name.clone()))?;
            let name = binding.as_str();
            eprintln!("{}: {}", name, result.output);
            start.record(false, execution_name);
            std::process::exit(result.exit_code);
        } else {
            let binding = (plugin.module.get_info())
                .map(|f| f().name)
                .ok_or_else(|| Error::PluginInterface(name.to_string(), "get_info".to_string()))
                .inspect_err(|_| PLUGIN_EXECUTION_METRIC.record(false, execution_name.clone()))?;
            let name = binding.as_str();
            println!("{}: {}", name, result.output);
            start.record(true, execution_name);
            std::process::exit(exitcode::OK);
        }
    }

    #[instrument]
    pub fn list_plugins(&self) {
        trace!(target: APP_NAME, "list_plugins");
        if self.plugins.is_empty() {
            warn!(target: APP_NAME, "No plugins loaded");
            return;
        }

        println!("Available plugins:");
        for (index, plugin) in self.plugins.iter().enumerate() {
            let Some(info) = plugin.module.get_info().map(|f| f()) else {
                error!(target: APP_NAME, "loaded UNKNOWN plugin interface incomplete: get_info");
                std::process::exit(exitcode::UNAVAILABLE);
            };
            println!(
                " {}. {} (v{}): {}",
                index + 1,
                info.name,
                info.version,
                info.description
            );
        }
        std::process::exit(exitcode::OK);
    }

    #[instrument]
    pub fn show_help(&self, plugin_name: &str) -> Result<(), Error> {
        trace!(target: APP_NAME, "help");
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

// METRICS
pub mod metric {
    use std::{sync::LazyLock, time::Instant};

    use opentelemetry::{KeyValue, global};

    pub struct LoadPluginsMetric {
        load_plugins: opentelemetry::metrics::Histogram<f64>,
        time: Instant,
    }

    impl LoadPluginsMetric {
        fn new(load_plugins: opentelemetry::metrics::Histogram<f64>) -> Self {
            Self {
                load_plugins,
                time: Instant::now(),
            }
        }

        pub fn record(&self, is_success: bool) {
            let time = self.time.elapsed();
            self.load_plugins.record(
                time.as_secs_f64(),
                &[
                    KeyValue::new("OS", std::env::consts::OS),
                    KeyValue::new("execution", if is_success { "OK" } else { "FAIL" }),
                ],
            );
        }
    }

    fn load_all_plugins_metric() -> LoadPluginsMetric {
        let meter = global::meter("load-all=plugins");
        // Histogram: Track update duration distribution
        let duration = meter
            .f64_histogram("plugins.load_all_plugins_time")
            .with_description("time to load all required plugins")
            .with_unit("s")
            .build();
        LoadPluginsMetric::new(duration)
    }

    pub static LOAD_ALL_PLUGINS_METRIC: LazyLock<LoadPluginsMetric> =
        LazyLock::new(load_all_plugins_metric);

    pub struct LoadPluginByNameMetric {
        load_plugin: opentelemetry::metrics::Histogram<f64>,
        time: Instant,
    }

    impl LoadPluginByNameMetric {
        fn new(load_plugin: opentelemetry::metrics::Histogram<f64>) -> Self {
            Self {
                load_plugin,
                time: Instant::now(),
            }
        }

        pub fn record(&self, is_success: bool, name: String) {
            let time = self.time.elapsed();
            self.load_plugin.record(
                time.as_secs_f64(),
                &[
                    KeyValue::new("name", name),
                    KeyValue::new("OS", std::env::consts::OS),
                    KeyValue::new("execution", if is_success { "OK" } else { "FAIL" }),
                ],
            );
        }
    }

    fn load_plugin_name_metric() -> LoadPluginByNameMetric {
        let meter = global::meter("load-plugin-by-name");
        // Histogram: Track update duration distribution
        let duration = meter
            .f64_histogram("plugins.load_plugin_by_name_time")
            .with_description("time to load all required plugins")
            .with_unit("s")
            .build();
        LoadPluginByNameMetric::new(duration)
    }

    pub static LOAD_PLUGIN_BY_NAME_METRIC: LazyLock<LoadPluginByNameMetric> =
        LazyLock::new(load_plugin_name_metric);

    pub struct PluginExecutionMetric {
        plugin_exec: opentelemetry::metrics::Histogram<f64>,
        time: Instant,
    }

    impl PluginExecutionMetric {
        fn new(plugin_exec: opentelemetry::metrics::Histogram<f64>) -> Self {
            Self {
                plugin_exec,
                time: Instant::now(),
            }
        }

        pub fn record(&self, is_success: bool, name: String) {
            let time = self.time.elapsed();
            self.plugin_exec.record(
                time.as_secs_f64(),
                &[
                    KeyValue::new("OS", std::env::consts::OS),
                    KeyValue::new("name", name),
                    KeyValue::new("execution", if is_success { "OK" } else { "FAIL" }),
                ],
            );
        }
    }

    fn plugin_execution_metric() -> PluginExecutionMetric {
        let meter = global::meter("plugin-execution");
        // Histogram: Track update duration distribution
        let duration = meter
            .f64_histogram("plugins.execution_time")
            .with_description("plugin execution time")
            .with_unit("s")
            .build();
        PluginExecutionMetric::new(duration)
    }

    pub static PLUGIN_EXECUTION_METRIC: LazyLock<PluginExecutionMetric> =
        LazyLock::new(plugin_execution_metric);
}
