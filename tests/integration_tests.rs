use std::result;

use protobuf_gen::{Config, ConfigError};

#[test]
fn unittest_yellow_book() -> result::Result<(), ConfigError> {
    env_logger::try_init().unwrap_or_default();

    let mut config = Config::new("protos", Some("proxy"));
    config.add_source("lib_tests/src/person.rs", "yellow_book");
    config.add_source("lib_tests/src/city.rs", "yellow_book");

    config.generate()?;
    Ok(())
}
