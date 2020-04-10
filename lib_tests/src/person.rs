use std::convert::{TryFrom, TryInto};

use protobuf_gen::ProtobufGen;

use crate::city::City;

#[derive(Debug, Clone, ProtobufGen, Arbitrary, PartialEq)]
#[protobuf_gen(proxy_mod = "crate::proxy")]
pub struct Dummy {
    pub id: u32,
}

#[derive(Debug, Default, Clone, ProtobufGen, Arbitrary, PartialEq)]
#[protobuf_gen(proxy_mod = "crate::proxy")]
pub struct Designer {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, ProtobufGen, Arbitrary, PartialEq)]
#[protobuf_gen(proxy_mod = "crate::proxy")]
pub enum Job {
    None,
    Programmer { skill: String, grade: u8 },
    Designer { designer: Designer },
}

impl Default for Job {
    fn default() -> Self {
        Job::None
    }
}

#[derive(Debug, Clone, ProtobufGen, Arbitrary, PartialEq)]
#[protobuf_gen(proxy_mod = "crate::proxy")]
pub enum AreaCode {
    Seoul,
    Seongnam,
    Jinhae,
}

#[derive(Debug, Default, Clone, Arbitrary, PartialEq)]
pub struct NumberBuffer(Vec<u8>);

impl TryInto<Vec<u8>> for NumberBuffer {
    type Error = anyhow::Error;

    fn try_into(self) -> anyhow::Result<Vec<u8>, Self::Error> {
        Ok(self.0)
    }
}

impl TryFrom<Vec<u8>> for NumberBuffer {
    type Error = anyhow::Error;

    fn try_from(vs: Vec<u8>) -> anyhow::Result<Self, Self::Error> {
        Ok(NumberBuffer(vs))
    }
}

#[derive(Debug, Clone, ProtobufGen, Arbitrary, PartialEq)]
#[protobuf_gen(proxy_mod = "crate::proxy")]
pub struct Person {
    pub(crate) _inner: i32,
    pub id: u8,
    #[protobuf_gen(substitute = "bytes")]
    pub number: NumberBuffer,
    pub hobbies: Vec<u32>,
    pub job: Job,
    pub city: City,
    pub area_code: AreaCode,
}
