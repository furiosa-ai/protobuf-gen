use std::convert::{TryFrom, TryInto};
use std::io::{Read, Write};

use failure::{Error, Fallible};
use prost::Message;
use protobuf_gen::ProtobufGen;

use crate::person::{AreaCode, Designer, Job, Person};
use crate::proxy;

impl TryFrom<Designer> for proxy::Designer {
    type Error = Error;

    fn try_from(this: Designer) -> Fallible<Self> {
        Ok(Self {
            id: this.id.try_into()?,
            name: this.name.try_into()?,
        })
    }
}

impl TryFrom<proxy::Designer> for Designer {
    type Error = Error;

    fn try_from(other: proxy::Designer) -> Fallible<Self> {
        Ok(Designer {
            id: other.id.try_into()?,
            name: other.name.try_into()?,
        })
    }
}

impl ProtobufGen for Designer {
    fn to_protobuf<W: Write>(self, w: &mut W) -> Fallible<()> {
        let proxy: proxy::Designer = self.try_into()?;
        let mut buffer = Vec::with_capacity(proxy.encoded_len());
        proxy.encode(&mut buffer)?;
        w.write_all(&buffer)?;
        Ok(())
    }

    fn from_protobuf<R: Read>(r: &mut R) -> Fallible<Self> {
        let mut buffer = Vec::new();
        r.read_to_end(&mut buffer)?;
        let proxy: proxy::Designer = Message::decode(buffer)?;
        proxy.try_into()
    }
}

impl TryFrom<proxy::job::NoneInner> for Job {
    type Error = Error;

    fn try_from(_: proxy::job::NoneInner) -> Fallible<Self> {
        Ok(Job::None)
    }
}

impl TryFrom<proxy::job::ProgrammerInner> for Job {
    type Error = Error;

    fn try_from(proxy: proxy::job::ProgrammerInner) -> Fallible<Self> {
        Ok(Job::Programmer {
            skill: proxy.skill.try_into()?,
            grade: proxy.grade.try_into()?,
        })
    }
}

impl TryFrom<proxy::Designer> for Job {
    type Error = Error;

    fn try_from(proxy: proxy::Designer) -> Fallible<Self> {
        Ok(Job::Designer(proxy.try_into()?))
    }
}

impl TryFrom<Job> for proxy::Job {
    type Error = Error;

    fn try_from(this: Job) -> Fallible<Self> {
        Ok(match this {
            Job::None => proxy::Job {
                inner: Some(proxy::job::Inner::None(proxy::job::NoneInner {})),
            },
            Job::Programmer { skill, grade } => proxy::Job {
                inner: Some(proxy::job::Inner::Programmer(proxy::job::ProgrammerInner {
                    skill: skill.try_into()?,
                    grade: grade.try_into()?,
                })),
            },
            Job::Designer(inner) => proxy::Job {
                inner: Some(proxy::job::Inner::Designer(inner.try_into()?)),
            },
        })
    }
}

impl TryFrom<proxy::Job> for Job {
    type Error = Error;

    fn try_from(proxy::Job { inner }: proxy::Job) -> Fallible<Self> {
        match inner.ok_or_else(|| format_err!("oneof doesn't have a value."))? {
            proxy::job::Inner::None(inner) => inner.try_into(),
            proxy::job::Inner::Programmer(inner) => inner.try_into(),
            proxy::job::Inner::Designer(inner) => inner.try_into(),
        }
    }
}

impl ProtobufGen for Job {
    fn to_protobuf<W: Write>(self, w: &mut W) -> Fallible<()> {
        let proxy: proxy::Job = self.try_into()?;
        let mut buffer = Vec::with_capacity(proxy.encoded_len());
        proxy.encode(&mut buffer)?;
        w.write_all(&buffer)?;
        Ok(())
    }

    fn from_protobuf<R: Read>(r: &mut R) -> Fallible<Self> {
        let mut buffer = Vec::new();
        r.read_to_end(&mut buffer)?;
        let proxy: proxy::Job = Message::decode(buffer)?;
        proxy.try_into()
    }
}

impl TryFrom<AreaCode> for proxy::AreaCode {
    type Error = Error;

    fn try_from(this: AreaCode) -> Fallible<Self> {
        Ok(match this {
            AreaCode::Seoul => proxy::AreaCode::Seoul,
            AreaCode::Seongnam => proxy::AreaCode::Seongnam,
            AreaCode::Jinhae => proxy::AreaCode::Jinhae,
        })
    }
}

impl TryFrom<proxy::AreaCode> for AreaCode {
    type Error = Error;

    fn try_from(proxy: proxy::AreaCode) -> Fallible<Self> {
        Ok(match proxy {
            proxy::AreaCode::Seoul => AreaCode::Seoul,
            proxy::AreaCode::Seongnam => AreaCode::Seongnam,
            proxy::AreaCode::Jinhae => AreaCode::Jinhae,
        })
    }
}

impl TryFrom<Person> for proxy::Person {
    type Error = Error;

    fn try_from(this: Person) -> Fallible<Self> {
        Ok(Self {
            id: this.id.try_into()?,
            number: this.number.try_into()?,
            hobbies: this.hobbies.try_into()?,
            job: Some(this.job.try_into()?),
            area_code: this.area_code.try_into()?,
        })
    }
}

impl ProtobufGen for AreaCode {
    fn to_protobuf<W: Write>(self, w: &mut W) -> Fallible<()> {
        let proxy: proxy::AreaCode = self.try_into()?;
        let proxy: i32 = proxy.into();
        let mut buffer = Vec::with_capacity(proxy.encoded_len());
        proxy.encode(&mut buffer)?;
        w.write_all(&buffer)?;
        Ok(())
    }

    fn from_protobuf<R: Read>(r: &mut R) -> Fallible<Self> {
        let mut buffer = Vec::new();
        r.read_to_end(&mut buffer)?;
        let proxy = proxy::AreaCode::from_i32(Message::decode(buffer)?)
            .ok_or_else(|| format_err!("invalid AreaCode"))?;
        proxy.try_into()
    }
}

// impl TryFrom<proxy::Person> for Person {
//     type Error = Error;

//     fn try_from(other: proxy::Person) -> Fallible<Self> {
//         Ok(Person {
//             _inner: Default::default(),
//             id: other.id.try_into()?,
//             number: other.number.try_into()?,
//             hobbies: other.hobbies.try_into()?,
//             job: other
//                 .job
//                 .ok_or_else(|| format_err!("empty job field"))?
//                 .try_into()?,
//             area_code: other.area_code.try_into()?,
//         })
//     }
// }

impl ProtobufGen for Person {
    fn to_protobuf<W: Write>(self, w: &mut W) -> Fallible<()> {
        let proxy: proxy::Person = self.try_into()?;
        let mut buffer = Vec::with_capacity(proxy.encoded_len());
        proxy.encode(&mut buffer)?;
        w.write_all(&buffer)?;
        Ok(())
    }

    fn from_protobuf<R: Read>(r: &mut R) -> Fallible<Self> {
        let mut buffer = Vec::new();
        r.read_to_end(&mut buffer)?;
        let proxy: proxy::Person = Message::decode(buffer)?;
        proxy.try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn proptest_from_to_protobuf_designer(
            original in any::<Designer>()
        ) {
            let mut buffer = Vec::new();
            original.clone().to_protobuf(&mut buffer).unwrap();

            let decoded = ProtobufGen::from_protobuf(&mut buffer.as_slice()).unwrap();
            prop_assert_eq!(original, decoded);
        }

        #[test]
        fn proptest_from_to_protobuf_job(
            original in any::<Job>()
        ) {
            let mut buffer = Vec::new();
            original.clone().to_protobuf(&mut buffer).unwrap();

            let decoded = ProtobufGen::from_protobuf(&mut buffer.as_slice()).unwrap();
            prop_assert_eq!(original, decoded);
        }

        #[test]
        fn proptest_from_to_protobuf_area_code(
            original in any::<AreaCode>()
        ) {
            let mut buffer = Vec::new();
            original.clone().to_protobuf(&mut buffer).unwrap();

            let decoded = ProtobufGen::from_protobuf(&mut buffer.as_slice()).unwrap();
            prop_assert_eq!(original, decoded);
        }

        #[test]
        fn proptest_from_to_protobuf_person(
            original in any::<Person>()
        ) {
            dbg!(&original);
            let mut buffer = Vec::new();
            original.clone().to_protobuf(&mut buffer).unwrap();

            let decoded: Person = ProtobufGen::from_protobuf(&mut buffer.as_slice()).unwrap();
            prop_assert_eq!(original.id, decoded.id);
            prop_assert_eq!(original.number, decoded.number);
            prop_assert_eq!(original.hobbies, decoded.hobbies);
            prop_assert_eq!(original.job, decoded.job);
            prop_assert_eq!(original.area_code, decoded.area_code);
        }
    }
}
