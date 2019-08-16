// use std::convert::{TryFrom, TryInto};
// use std::io::{Read, Write};

// use failure::{Error, Fallible};
// use prost::Message;
use protobuf_gen::ProtobufGen;

// use crate::proxy;

#[derive(Debug, Default, Clone, ProtobufGen, Arbitrary, PartialEq)]
// #[protobuf_gen(proxy_mod = "proxy")]
pub struct City {
    pub name: String,
}
