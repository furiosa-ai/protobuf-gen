#[macro_use]
extern crate protobuf_gen_derive;

pub enum Job {
    Programmer { skill: String, grade: i32 },
    Scientist { major: String, code: u8 },
}

#[derive(ProtobufGen)]
pub struct Person {
    _inner: i32,
    #[protobuf_type(bytes)]
    pub number: [u8; 8],
    pub hobbies: Vec<u8>,
    pub job: Job,
}
