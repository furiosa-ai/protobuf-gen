use std::convert::{TryFrom, TryInto};

use failure::{Error, Fallible};
use protobuf_gen::ProtobufGen;

use crate::city::City;

#[derive(Debug, Clone, ProtobufGen, Arbitrary, PartialEq)]
pub struct Dummy {}

#[derive(Debug, Default, Clone, ProtobufGen, Arbitrary, PartialEq)]
pub struct Designer {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, ProtobufGen, Arbitrary, PartialEq)]
pub enum Job {
    None,
    Programmer { skill: String, grade: u8 },
    Designer(Designer),
}

impl Default for Job {
    fn default() -> Self {
        Job::None
    }
}

#[derive(Debug, Clone, ProtobufGen, Arbitrary, PartialEq)]
pub enum AreaCode {
    Seoul,
    Seongnam,
    Jinhae,
}

impl TryInto<i32> for AreaCode {
    type Error = Error;

    fn try_into(self) -> Fallible<i32> {
        Ok(match self {
            AreaCode::Seoul => 0,
            AreaCode::Seongnam => 1,
            AreaCode::Jinhae => 2,
        })
    }
}

impl TryFrom<i32> for AreaCode {
    type Error = Error;

    fn try_from(n: i32) -> Fallible<Self> {
        Ok(match n {
            0 => AreaCode::Seoul,
            1 => AreaCode::Seongnam,
            2 => AreaCode::Jinhae,
            _ => bail!("invalid discriminant"),
        })
    }
}

#[derive(Debug, Default, Clone, Arbitrary, PartialEq)]
pub struct NumberBuffer(Vec<u8>);

impl TryInto<Vec<u8>> for NumberBuffer {
    type Error = Error;

    fn try_into(self) -> Fallible<Vec<u8>> {
        Ok(self.0)
    }
}

impl TryFrom<Vec<u8>> for NumberBuffer {
    type Error = Error;

    fn try_from(vs: Vec<u8>) -> Fallible<Self> {
        Ok(NumberBuffer(vs))
    }
}

#[derive(Debug, Clone, ProtobufGen, Arbitrary, PartialEq)]
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
