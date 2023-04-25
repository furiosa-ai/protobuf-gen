use std::collections::{HashMap, HashSet};
use std::convert::TryInto;

use protobuf_gen::ProtobufGen;

use crate::city::City;

#[derive(Debug, Default, Clone, ProtobufGen, Arbitrary, PartialEq, Eq, Hash)]
#[protobuf_gen(proxy_mod = "crate::proxy")]
pub struct Dummy {
    pub id: u32,
}

#[derive(Debug, Default, Clone, ProtobufGen, Arbitrary, PartialEq, Eq, Hash)]
#[protobuf_gen(proxy_mod = "crate::proxy")]
pub struct Designer {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Default, Clone, ProtobufGen, Arbitrary, PartialEq, Eq, Hash)]
#[protobuf_gen(proxy_mod = "crate::proxy")]
pub enum Job {
    #[default]
    None,
    Programmer {
        skill: String,
        grade: Option<u8>,
    },
    Designer {
        designer: Designer,
    },
    DesignerOpaque {
        #[protobuf_gen(opaque)]
        designer: Designer,
    },
}

#[derive(Debug, Default, Clone, ProtobufGen, Arbitrary, PartialEq, Eq, Hash)]
#[protobuf_gen(proxy_mod = "crate::proxy")]
pub enum AreaCode {
    #[default]
    Seoul,
    Seongnam,
    Jinhae,
}

#[derive(Debug, Default, Clone, Arbitrary, PartialEq, Eq, Hash)]
pub struct NumberBuffer(Vec<u8>);

impl Into<Vec<u8>> for NumberBuffer {
    fn into(self) -> Vec<u8> {
        self.0
    }
}

impl From<Vec<u8>> for NumberBuffer {
    fn from(vs: Vec<u8>) -> Self {
        NumberBuffer(vs)
    }
}

#[derive(Debug, Default, Clone, ProtobufGen, Arbitrary, PartialEq, Eq, Hash)]
#[protobuf_gen(proxy_mod = "crate::proxy")]
pub struct Car {
    pub number: usize,
}

#[derive(Debug, Default, Clone, ProtobufGen, Arbitrary, PartialEq, Eq, Hash)]
#[protobuf_gen(proxy_mod = "crate::proxy")]
pub enum CarTag {
    #[default]
    None,
    Number,
}

#[derive(Debug, Default, Clone, ProtobufGen, Arbitrary, PartialEq, Eq, Hash)]
#[protobuf_gen(proxy_mod = "crate::proxy")]
pub struct Person {
    #[protobuf_gen(skip)]
    pub _inner: i32,
    pub id: u8,
    #[protobuf_gen(substitute = "bytes")]
    pub number: NumberBuffer,
    pub hobbies: Vec<u32>,
    pub job: Job,
    pub city: Option<City>,
    pub area_code: AreaCode,
    pub car: Car,
    pub cars: Vec<Car>,
    pub car_tag: CarTag,
}

#[derive(Debug, Default, Clone, ProtobufGen, Arbitrary, PartialEq)]
#[protobuf_gen(proxy_mod = "crate::proxy")]
pub struct VecOfPerson {
    pub vec: Vec<Person>,
    #[protobuf_gen(opaque)]
    pub opaque_vec: Vec<Person>,
}

#[derive(Debug, Default, Clone, ProtobufGen, Arbitrary, PartialEq)]
#[protobuf_gen(proxy_mod = "crate::proxy")]
pub struct SetOfPerson {
    pub set: HashSet<Person>,
    #[protobuf_gen(opaque)]
    pub opaque_set: HashSet<Person>,
}

#[derive(Debug, Default, Clone, ProtobufGen, Arbitrary, PartialEq)]
#[protobuf_gen(proxy_mod = "crate::proxy")]
pub struct OptionOfPerson {
    pub option: Option<Person>,
    #[protobuf_gen(opaque)]
    pub opaque_option: Option<Person>,
}

#[derive(Debug, Default, Clone, ProtobufGen, Arbitrary, PartialEq)]
#[protobuf_gen(proxy_mod = "crate::proxy")]
pub struct MapOfPerson {
    #[protobuf_gen(substitute = "map<string, Person>")]
    pub map: HashMap<String, Person>,
}

#[derive(Debug, Default, Clone, ProtobufGen, Arbitrary, PartialEq)]
#[protobuf_gen(proxy_mod = "crate::proxy")]
pub enum VariousPerson {
    #[default]
    None,
    Person {
        inner: Person,
    },
    VecOfPerson {
        inner: Vec<Person>,
    },
    SetOfPerson {
        inner: HashSet<Person>,
    },
    MapOfPerson {
        #[protobuf_gen(substitute = "map<string, Person>")]
        inner: HashMap<String, Person>,
    },
    OptionOfPerson {
        inner: Option<Person>,
    },
    OpaqueVecOfPerson {
        #[protobuf_gen(opaque)]
        inner: Vec<Person>,
    },
    OpaqueSetOfPerson {
        #[protobuf_gen(opaque)]
        inner: HashSet<Person>,
    },
    OpaqueOptionOfPerson {
        #[protobuf_gen(opaque)]
        inner: Option<Person>,
    },
    OpaqueMapOfPersone {
        // this is a W/A, opaque HashMap is not supported.
        #[protobuf_gen(opaque)]
        inner: MapOfPerson,
    },
}
