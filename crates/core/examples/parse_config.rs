use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let config_path = args.get(1).expect("Usage: parse_config <config.toml>");

    let content = fs::read_to_string(config_path).expect("Failed to read config");
    let config: toml::Value = toml::from_str(&content).expect("Failed to parse TOML");

    println!("✓ Configuration is valid TOML");
    println!("{:#?}", config);
}
