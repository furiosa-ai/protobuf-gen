use protobuf_gen::ProtobufGen;

#[derive(Debug, ProtobufGen, Arbitrary, PartialEq)]
pub struct City {
    pub name: String,
}
