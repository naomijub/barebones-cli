use abi_stable::{
    StableAbi,
    library::RootModule,
    package_version_strings,
    sabi_types::VersionStrings,
    std_types::{RString, RVec},
};

/// The root module that plugins must export
#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = PluginModRef)))]
pub struct PluginMod {
    /// Get plugin metadata
    pub get_info: extern "C" fn() -> PluginInfo,

    /// Execute the plugin command
    pub execute: extern "C" fn(args: RVec<RString>) -> CommandResult,

    /// Get help text for this plugin
    pub get_help: extern "C" fn() -> RString,
}

impl RootModule for PluginModRef {
    abi_stable::declare_root_module_statics! {PluginModRef}

    const BASE_NAME: &'static str = "default_plugin_name";
    const NAME: &'static str = "default_plugin_name";
    const VERSION_STRINGS: VersionStrings = package_version_strings!();
}

/// Plugin metadata
#[repr(C)]
#[derive(StableAbi, Debug, Clone)]
pub struct PluginInfo {
    pub name: RString,
    pub version: RString,
    pub description: RString,
    pub author: RString,
}

/// Result of command execution
#[repr(C)]
#[derive(StableAbi, Debug)]
pub struct CommandResult {
    pub success: bool,
    pub output: RString,
    pub exit_code: i32,
}

impl CommandResult {
    pub fn ok(output: impl Into<RString>) -> Self {
        Self {
            success: true,
            output: output.into(),
            exit_code: 0,
        }
    }

    pub fn err(output: impl Into<RString>, exit_code: i32) -> Self {
        Self {
            success: false,
            output: output.into(),
            exit_code,
        }
    }
}
