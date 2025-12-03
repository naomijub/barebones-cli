# Barebones-CLI

> [`abi_stable`](https://crates.io/crates/abi_stable) based Plugin System for Rust CLI Applications

A CLI template repository to build extensible plugin based CLI applications in Rust. The template allows you to create and modify systems that are loaded dynamically at runtime, guaranteeing compatibility even if compiled with different Rust versions.

## Features

- **Dynamic Loading**: Loads plugins at runtime without needing to recompe the CLI.
- **Type Safety**: Leverages `abi_stable` types for safe  rust-to-rust FFI.
- **Simple Interface**: Clean three-function contract for plugin development.
- **Cross-Platform**: Linux, macOS, Windows
- **Self-Updating Binary**: Binary checks for update on every execution.

## Quick Start

### 1. Necessary changes

- Rename package in `Cargo.toml`:
```toml
[package]
name = "barebones" # <-- here
version = "0.1.0"
edition = "2024"

[[bin]]
name = "barebones-cli" # <-- here
path = "src/bin/main.rs"
```

- Rename panic hooks on `src/bin/main.rs`:

```rust
setup_panic!(
    Metadata::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
        .authors("Barebones CLI Corp <support@barebones-cli.corp>")
        .homepage("support.barebones-cli.corp")
        .support("- Open a support request by email to support@barebones-cli.corp")
);
```

- `APP_NAME` constant in `src/lib.rs`.

- You might need to configure your own Settings Default in `src/config/data.rs`

```rust
impl Default for MyConfig {
    fn default() -> Self {
        #[expect(
            clippy::unwrap_used,
            reason = "We guarantee this version works in compile time"
        )]
        Self {
            name: "Julia Naomi".to_string(), // Not required
            is_machine: false, // Required if you want clean output for other machines
            version: Version::parse("1.0.0").unwrap(), // Not required
            plugins: vec![ // REQUIRED
                "your_plugins_names" // <-- a list of your plugins
            ],
        }
    }
}
```

### 2. Create Your First Plugin

Navigate to the plugins source directory:

```bash
cd plugins
cargo new --lib my-first-plugin
cd my-first-plugin
```

Update `Cargo.toml` to build as a dynamic library:

```toml
[package]
name = "my-first-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
cli-dev.workspace = true

abi_stable.workspace = true

[lints]
workspace = true
```

Copy the contents of `plugins/greeter/src/lib.rs` to start:

Implement your plugin changes in ``plugins/my_first_plugin/src/lib.rs``:

```rust
use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    sabi_extern_fn,
    std_types::{RString, RVec},
};
use cli_dev::{
    logging::log_debug,
    plugin::{CommandResult, PluginInfo, PluginMod, PluginModRef},
};

#[export_root_module]
pub fn instantiate_root_module() -> PluginModRef {
    PluginMod {
        get_info,
        execute,
        get_help,
    }
    .leak_into_prefix()
}

#[sabi_extern_fn]
pub fn get_info() -> PluginInfo {
    PluginInfo {
        name: "my-plugin".into(),
        version: "0.1.0".into(),
        description: "My awesome plugin".into(),
        author: "Your Name".into(),
    }
}

#[sabi_extern_fn]
pub fn execute(args: RVec<RString>) -> CommandResult {
    // Your plugin logic here
    CommandResult::ok("Plugin executed successfully!")
}

#[sabi_extern_fn]
pub fn get_help() -> RString {
    "my-first-plugin - Does something awesome
    
    USAGE:
        your-cli-name my-first-plugin [args]".into()
}
```

### 3. Build and Install Your Plugin

Build the plugin:

```bash
cargo build --release -p my-first-plugin
```

Copy the compiled library to the cli config directory `$HOME/.cli-name/`:

```bash
# Linux
cp target/release/libmy_first_plugin.so $HOME/.cli-name/

# macOS
cp target/release/libmy_first_plugin.dylib $HOME/.cli-name/

# Windows
copy target\release\my_first_plugin.dll $HOME/.cli-name/
```

### 4. Run Your CLI Application

```bash
./target/release/your-cli-app list
./target/release/your-cli-app my-first-plugin --your args
./target/release/your-cli-app help my-first-plugin
```

## Plugin Development Guide

### The Plugin "Interface"

Every plugin must implement three functions:

#### 1. `get_info()` - Plugin Metadata

Returns information about your plugin:

```rust
#[sabi_extern_fn]
pub fn get_info() -> PluginInfo {
    PluginInfo {
        name: "my-first-plugin".into(),
        version: "1.0.0".into(),
        description: "What your plugin does".into(),
        author: "Your Name".into(),
    }
}
```

#### 2. `execute()` - Main Logic

Executes your plugin's functionality:

```rust
#[sabi_extern_fn]
pub fn execute(args: RVec<RString>) -> CommandResult {
    // Parse arguments
    if args.is_empty() {
        return CommandResult::err("No arguments provided", exitcode::USAGE);
    }
    
    // Do your plugin's work
    let result = do_something(&args[0]);
    
    // Return success or error
    CommandResult::ok(format!("Result: {}", result))
}
```

#### 3. `get_help()` - Documentation

Returns help text for users:

```rust
#[sabi_extern_fn]
pub fn get_help() -> RString {
    r#"my-first-plugin - Description

USAGE:
    your-cli-app my-first-plugin [OPTIONS] [ARGS]

OPTIONS:
    --flag    Description of flag

EXAMPLES:
    your-cli-app my-first-plugin --flag value
"#.into()
}
```

### Working with Arguments

Arguments are passed as `RVec<RString>`, which are `abi_stable` versions of `Vec<String>`:

```rust
#[sabi_extern_fn]
pub fn execute(args: RVec<RString>) -> CommandResult {
    // Check argument count
    if args.len() < 2 {
        return CommandResult::err("Need at least 2 arguments", exitcode::USAGE);
    }
    
    // Parsing can also be done using `Clap::parse_from` iterator
    // Access arguments
    let first = args[0].as_str();
    let second = args[1].as_str();
    
    // Parse values
    let number: i32 = match first.parse() {
        Ok(n) => n,
        Err(_) => return CommandResult::err("Invalid number", exitcode::DATAERR),
    };
    
    // Check for flags
    if args.iter().any(|a| a.as_str() == "--verbose") {
        // Verbose mode
    }
    
    CommandResult::ok("Success!")
}
```

### Returning Results

Use `CommandResult::ok()` for success and `CommandResult::err()` for failures:

```rust
// Success
return CommandResult::ok("Operation completed successfully");

// Error with exit code
return CommandResult::err("Something went wrong", exitcode::SOFTWARE);

// Format output
return CommandResult::ok(format!("Processed {} items", count));
```

## Examples

### Example 1: Simple Greeting Plugin

```rust
#[sabi_extern_fn]
pub fn execute(args: RVec<RString>) -> CommandResult {
    let name = if args.is_empty() {
        "World"
    } else {
        args[0].as_str()
    };
    
    CommandResult::ok(format!("Hello, {}!", name))
}
```

### Example 2: Calculator Plugin

```rust
#[sabi_extern_fn]
pub fn execute(args: RVec<RString>) -> CommandResult {
    if args.len() != 3 {
        return CommandResult::err("Usage: calc <NUM> <OP> <NUM>", exitcode::USAGE);
    }
    
    let Ok(a) = args[0].parse::<f64>() else {
        return CommandResult::err(format!("first argument must be a number. Provided `{}`", args[0]), exitcode::DATAERR);
    };
    let op = args[1].as_str();
    let Ok(b) = args[2].parse::<f64>() else {
        return CommandResult::err(format!("third argument must be a number. Provided `{}`", args[0]), exitcode::DATAERR);
    };
    
    let result = match op {
        "add" => a + b,
        "sub" => a - b,
        "mul" => a * b,
        "div" if b != 0.0 => a / b,
        "div" => return CommandResult::err("Division by zero", exitcode::DATAERR),
        _ => return CommandResult::err("Unknown operation", exitcode::DATAERR),
    };
    
    CommandResult::ok(format!("{}", result))
}
```

### Example 3: File Processing Plugin

```rust
#[sabi_extern_fn]
pub fn execute(args: RVec<RString>) -> CommandResult {
    if args.is_empty() {
        return CommandResult::err("Please provide a filename", exitcode::USAGE);
    }
    
    let filename = args[0].as_str();
    
    match std::fs::read_to_string(filename) {
        Ok(content) => {
            CommandResult::ok(content)
        }
        Err(e) => CommandResult::err(format!("Error reading file: {}", e), exitcode::IOERR),
    }
}
```

## Customizing the Main Application

### Changing the Plugin Directory

In `src/main.rs`, modify this line:

```rust
manager.load_plugins_from_dir("./plugins")?;
```

instead of:

```rust
let settings = config_show()?;

if let Err(e) = manager.load_plugins_from_dir(&settings) // ...
```

### Adding Plugin Validation

You can add validation logic in the `load_plugin()` method:

```rust
fn load_plugin(&mut self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let module = PluginModRef::load_from_file(path)?.into_inner();
    let info = (module.get_info())();
    
    // Validate version
    if !info.version.starts_with("1.") {
        return Err("Plugin version not supported".into());
    }
    
    // Check for duplicates
    if self.plugins.iter().any(|p| p.name == info.name.as_str()) {
        return Err(format!("Plugin '{}' already loaded", info.name).into());
    }
    
    // ... rest of loading logic
}
```

## Tips and Best Practices

1. **Always copy the plugin interface**: Each plugin needs an exact copy of the interface definition (`crates/cli-dev/plugin/mod.rs::PluginMod`).

2. **Keep the interface stable**: Once you release your CLI, avoid changing the interface to maintain plugin compatibility.

3. **Error handling**: Always return meaningful error messages to users.

4. **Documentation**: Write comprehensive help text for your plugins.

5. **Testing**: Test plugins independently before integrating them.

6. **Distribution**: You can distribute plugins as separate packages that users install to their plugins directory.

## Troubleshooting

### Disabling Auto-Update

To disable auto-update, open `$HOME/.barebones/config.toml` and set `should_auto_update = false`. To run manually run the update command execute `barebones-cli update`. To auto accept downloads use the `--accept` flag `barebones-cli --accept update`

### Plugin Not Loading

- Ensure the file extension is correct for your platform (.so/.dylib/.dll).
- Check that `abi_stable` versions match between main app and plugin.
- Verify the plugin exports `instantiate_root_module`.

### Compilation Errors

- Make sure `plugin interface` is identical in both main app and plugin.
- Check that `crate-type = ["cdylib"]` is set in plugin's Cargo.toml.
- Ensure all types used are `#[repr(C)]` and `StableAbi`.

### Runtime Crashes

- Never use non-ABI-stable types across the FFI boundary.
- Always use `RString`, `RVec`, etc. instead of `String`, `Vec`.
- Avoid unwrap() in plugin code; log errors and return them instead.

## Resources

- [abi_stable documentation](https://docs.rs/abi_stable/)
- [abi_stable examples](https://github.com/rodrimati1992/abi_stable_crates/tree/master/examples)
- [The Rust FFI Omnibus](http://jakegoulding.com/rust-ffi-omnibus/)