use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    sabi_extern_fn,
    std_types::{RStr, RString, RVec},
};
use clap::Parser;
use cli_dev::{
    logging::debug,
    plugin::{CommandResult, PluginInfo, PluginMod, PluginModRef},
    prelude::exitcode,
};
use tracing::instrument;

use crate::commands::Args;

mod commands;
const TARGET_NAME: &str = "greeter";
pub const PLUGIN_NAME: RStr<'static> = RStr::from_str(TARGET_NAME);

/// Export the plugin root module
#[export_root_module]
#[instrument]
pub fn get_plugin() -> PluginModRef {
    PluginMod {
        get_info,
        execute,
        get_help,
    }
    .leak_into_prefix()
}

/// Return plugin metadata
#[sabi_extern_fn]
#[instrument]
pub fn get_info() -> PluginInfo {
    PluginInfo {
        name: "greeter".into(),
        version: "0.1.0".into(),
        description: "A simple greeter plugin".into(),
        author: "Julia Naomi".into(),
    }
}

/// Execute the plugin command
#[sabi_extern_fn]
#[instrument]
pub extern "C" fn execute(args: RVec<RString>) -> CommandResult {
    debug!(
        target: TARGET_NAME,
        "received arguments: {}", args.join(" "),
    );
    if args.is_empty() {
        return CommandResult::ok("Hello, World!");
    }

    let args = args.iter().map(|arg| arg.to_string());
    let args = Args::parse_from(args);

    if let Some(name) = args.name {
        CommandResult::ok(format!("Hello, {}!", name))
    } else if let Some(uppercase) = args.uppercase {
        CommandResult::ok(format!("HELLO, {}!", uppercase.to_uppercase()))
    } else {
        CommandResult::err(
            "`NAME` or `--uppercase UPPERCASE_NAME` are required",
            exitcode::USAGE,
        )
    }
}

/// Return help text
#[sabi_extern_fn]
#[instrument]
pub fn get_help() -> RString {
    r#"greeter - A simple greeting plugin

USAGE:
    barebones-cli greeter [OPTIONS] [NAME]

OPTIONS:
    --uppercase    Convert the greeting to uppercase

EXAMPLES:
    barebones-cli greeter
    barebones-cli greeter Foo
    barebones-cli greeter --uppercase bar
"#
    .into()
}
