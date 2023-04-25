use std::result;

use protobuf_gen::{Config, ConfigError};

pub fn main() -> result::Result<(), ConfigError> {
    let mut config = Config::new("protos", Some("proxy"));
    config.add_source("src/person.rs", "yellow_book");
    config.add_source("src/city.rs", "yellow_book");
    config.add_source("src/tree.rs", "tree");
    config.opaque_type("Car");
    config.opaque_type("CarTag");

    config.generate()?;
    Ok(())
}
