fn get_name() -> &'static str {
    option_env!("CARGO_PKG_NAME").unwrap_or("<program name not set>")
}

fn get_version() -> &'static str {
    option_env!("CARGO_PKG_VERSION").unwrap_or("<unknown>")
}

pub fn command() -> i8 {
    println!("{} version {}", get_name(), get_version());
    0
}
