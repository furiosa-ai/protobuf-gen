use protobuf_gen::ProtobufGen;

#[derive(Debug, Default, Clone, ProtobufGen, Arbitrary, PartialEq)]
pub struct City {
    pub name: String,
}
