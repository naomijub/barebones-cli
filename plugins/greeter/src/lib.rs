use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    sabi_extern_fn,
    std_types::{RString, RVec},
};
use cli_dev::plugin::{CommandResult, PluginInfo, PluginMod, PluginModRef};

/// Export the plugin root module
#[export_root_module]
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
pub extern "C" fn execute(args: RVec<RString>) -> CommandResult {
    if args.is_empty() {
        return CommandResult::ok("Hello, World!");
    }

    // Check for flags
    if args[0].as_str() == "--uppercase" {
        if args.len() < 2 {
            return CommandResult::err("--uppercase requires a name argument", 1);
        }
        let name = args[1].as_str();
        return CommandResult::ok(format!("HELLO, {}!", name.to_uppercase()));
    }

    // Just say hello to the provided name
    let name = args[0].as_str();
    CommandResult::ok(format!("Hello, {}!", name))
}

/// Return help text
#[sabi_extern_fn]
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
