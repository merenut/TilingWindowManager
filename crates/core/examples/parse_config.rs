use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let config_path = args
        .get(1)
        .expect("Usage: cargo run --example parse_config <path-to-config.toml>");

    let content = fs::read_to_string(config_path).unwrap_or_else(|err| {
        eprintln!("Failed to read config file '{}': {}", config_path, err);
        std::process::exit(1);
    });

    let config: toml::Value = toml::from_str(&content).unwrap_or_else(|err| {
        eprintln!("Failed to parse TOML from '{}': {}", config_path, err);
        std::process::exit(1);
    });

    println!("✓ Configuration is valid TOML");
    println!("{:#?}", config);
}
