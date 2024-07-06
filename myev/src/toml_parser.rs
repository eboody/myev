use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Config {
    device: String,
    mappings: Vec<Mapping>,
}

#[derive(Deserialize, Debug)]
struct Mapping {
    key: String,
    tap: String,
    hold: String,
}

fn parse_toml() {
    let config: Config = toml::from_str(
        r#"
   device = '/dev/input/by-path/platform-i8042-serio-0-event-kbd'

[[mappings]]
key = "Capslock"
tap = "Esc"
hold = "LMeta"

"#,
    )
    .unwrap();

    println!("{config:#?}");
}
