use std::{
    fs::OpenOptions,
    io::{Read, Write},
};

use barebones::config::{config_dir, data::MyConfig};

#[test]
fn cli_tests() {
    disable_auto_update();

    trycmd::TestCases::new().case("tests/cmd/*.md");
}

fn disable_auto_update() {
    // disable auto-update for test
    let path = config_dir().join("config.toml");
    let mut file = OpenOptions::new().read(true).open(&path).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    let mut config: MyConfig = toml::from_str(&content).unwrap();

    config.should_auto_update = false;

    let new_content = toml::to_string_pretty(&config).unwrap();

    let mut file = OpenOptions::new()
        .truncate(true)
        .write(true)
        .append(false)
        .open(path)
        .unwrap();
    file.write_all(new_content.as_bytes()).unwrap();
}
