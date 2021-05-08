use vergen::{Config, vergen};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    return Ok(vergen(Config::default())?);
}
