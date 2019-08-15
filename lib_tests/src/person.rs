use std::convert::{TryFrom, TryInto};

use failure::{Error, Fallible};
use protobuf_gen::ProtobufGen;

use crate::city::City;

#[derive(Debug, Clone, ProtobufGen, Arbitrary, PartialEq)]
pub struct Dummy {}

#[derive(Debug, Clone, ProtobufGen, Arbitrary, PartialEq)]
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

#[derive(Debug, Clone, Arbitrary, PartialEq)]
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
    _inner: i32,
    pub id: u8,
    #[protobuf_gen(substitute = "bytes")]
    pub number: NumberBuffer,
    pub hobbies: Vec<u32>,
    pub job: Job,
    pub city: City,
    pub area_code: AreaCode,
}

use crate::proxy;

impl TryFrom<proxy::Person> for Person {
    type Error = Error;

    fn try_from(other: proxy::Person) -> Fallible<Self> {
        Ok(Person {
            _inner: Default::default(),
            id: other.id.try_into()?,
            number: other.number.try_into()?,
            hobbies: other.hobbies.try_into()?,
            job: other
                .job
                .ok_or_else(|| format_err!("empty {} field", stringify!(Person::job)))?
                .try_into()?,
            area_code: other.area_code.try_into()?,
            city: other
                .city
                .ok_or_else(|| format_err!("empty city field"))?
                .try_into()?,
        })
    }
}
