const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const VERSION: &str = const_format::formatcp!("nk version {}", CARGO_PKG_VERSION);

pub fn version() -> Result<(), Box<dyn std::error::Error>> {
    Ok(println!("{}", VERSION))
}
