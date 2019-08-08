use failure::Fallible;

use protobuf_gen::FilePrinter;

#[test]
fn unittest_parse_file() -> Fallible<()> {
    println!("{}", FilePrinter::new(include_str!("text.rs")));
    Ok(())
}
