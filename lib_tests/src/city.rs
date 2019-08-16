use protobuf_gen::ProtobufGen;

#[derive(Debug, Default, Clone, ProtobufGen, Arbitrary, PartialEq)]
#[protobuf_gen(proxy_mod = "crate::proxy")]
pub struct City {
    pub name: String,
}
