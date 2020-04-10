use protobuf_gen::Config;

pub fn main() -> anyhow::Result<()> {
    let mut config = Config::new("protos", Some("proxy"));
    config.add_source("src/person.rs", "yellow_book");
    config.add_source("src/city.rs", "yellow_book");

    config.generate()?;
    Ok(())
}
