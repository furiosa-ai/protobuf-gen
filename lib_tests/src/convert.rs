use std::convert::{TryFrom, TryInto};
use std::io::{Read, Write};

use failure::{Error, Fallible};
use prost::Message;
use protobuf_gen::ProtobufGen;

use crate::city::City;
use crate::person::{AreaCode, Designer, Dummy, Job, Person};
use crate::proxy;

impl TryInto<Option<proxy::City>> for City {
    type Error = Error;
    fn try_into(self) -> Fallible<Option<proxy::City>> {
        let City { name, .. } = self;
        Ok(Some(proxy::City {
            name: name.try_into()?,
        }))
    }
}
impl TryInto<proxy::City> for City {
    type Error = Error;
    fn try_into(self) -> Fallible<proxy::City> {
        let City { name, .. } = self;
        Ok(proxy::City {
            name: name.try_into()?,
        })
    }
}
impl TryFrom<Option<proxy::City>> for City {
    type Error = Error;
    fn try_from(other: Option<proxy::City>) -> Fallible<Self> {
        let proxy::City { name } = other
            .ok_or_else(|| format_err!("empty {} object", stringify!(proxy::City)))?
            .try_into()?;
        Ok(Self {
            name: name.try_into()?,
        })
    }
}
impl TryFrom<proxy::City> for City {
    type Error = Error;
    fn try_from(proxy::City { name }: proxy::City) -> Fallible<Self> {
        Ok(Self {
            name: name.try_into()?,
        })
    }
}
impl ProtobufGen for City {
    fn to_protobuf<W: Write>(self, w: &mut W) -> Fallible<()> {
        let proxy: proxy::City = self.try_into()?;
        let mut buffer = Vec::with_capacity(proxy.encoded_len());
        proxy.encode(&mut buffer)?;
        w.write_all(&buffer)?;
        Ok(())
    }
    fn from_protobuf<R: Read>(r: &mut R) -> Fallible<Self> {
        let mut buffer = Vec::new();
        r.read_to_end(&mut buffer)?;
        let proxy: proxy::City = Message::decode(buffer)?;
        proxy.try_into()
    }
}
impl TryInto<Option<proxy::Dummy>> for Dummy {
    type Error = Error;
    fn try_into(self) -> Fallible<Option<proxy::Dummy>> {
        let Dummy { .. } = self;
        Ok(Some(proxy::Dummy {}))
    }
}
impl TryInto<proxy::Dummy> for Dummy {
    type Error = Error;
    fn try_into(self) -> Fallible<proxy::Dummy> {
        let Dummy { .. } = self;
        Ok(proxy::Dummy {})
    }
}
impl TryFrom<Option<proxy::Dummy>> for Dummy {
    type Error = Error;
    fn try_from(other: Option<proxy::Dummy>) -> Fallible<Self> {
        let proxy::Dummy {} = other
            .ok_or_else(|| format_err!("empty {} object", stringify!(proxy::Dummy)))?
            .try_into()?;
        Ok(Self {})
    }
}
impl TryFrom<proxy::Dummy> for Dummy {
    type Error = Error;
    fn try_from(proxy::Dummy {}: proxy::Dummy) -> Fallible<Self> {
        Ok(Self {})
    }
}
impl ProtobufGen for Dummy {
    fn to_protobuf<W: Write>(self, w: &mut W) -> Fallible<()> {
        let proxy: proxy::Dummy = self.try_into()?;
        let mut buffer = Vec::with_capacity(proxy.encoded_len());
        proxy.encode(&mut buffer)?;
        w.write_all(&buffer)?;
        Ok(())
    }
    fn from_protobuf<R: Read>(r: &mut R) -> Fallible<Self> {
        let mut buffer = Vec::new();
        r.read_to_end(&mut buffer)?;
        let proxy: proxy::Dummy = Message::decode(buffer)?;
        proxy.try_into()
    }
}
impl TryInto<Option<proxy::Designer>> for Designer {
    type Error = Error;
    fn try_into(self) -> Fallible<Option<proxy::Designer>> {
        let Designer { id, name, .. } = self;
        Ok(Some(proxy::Designer {
            id: id.try_into()?,
            name: name.try_into()?,
        }))
    }
}
impl TryInto<proxy::Designer> for Designer {
    type Error = Error;
    fn try_into(self) -> Fallible<proxy::Designer> {
        let Designer { id, name, .. } = self;
        Ok(proxy::Designer {
            id: id.try_into()?,
            name: name.try_into()?,
        })
    }
}
impl TryFrom<Option<proxy::Designer>> for Designer {
    type Error = Error;
    fn try_from(other: Option<proxy::Designer>) -> Fallible<Self> {
        let proxy::Designer { id, name } = other
            .ok_or_else(|| format_err!("empty {} object", stringify!(proxy::Designer)))?
            .try_into()?;
        Ok(Self {
            id: id.try_into()?,
            name: name.try_into()?,
        })
    }
}
impl TryFrom<proxy::Designer> for Designer {
    type Error = Error;
    fn try_from(proxy::Designer { id, name }: proxy::Designer) -> Fallible<Self> {
        Ok(Self {
            id: id.try_into()?,
            name: name.try_into()?,
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
impl TryInto<proxy::Job> for Job {
    type Error = Error;
    fn try_into(self) -> Fallible<proxy::Job> {
        Ok(match self {
            Job::None {} => proxy::Job {
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
impl TryInto<Option<proxy::Job>> for Job {
    type Error = Error;
    fn try_into(self) -> Fallible<Option<proxy::Job>> {
        Ok(Some(match self {
            Job::None {} => proxy::Job {
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
        }))
    }
}
impl TryFrom<proxy::Job> for Job {
    type Error = Error;
    fn try_from(proxy::Job { inner }: proxy::Job) -> Fallible<Self> {
        match inner.ok_or_else(|| format_err!("{} doesn't have a value.", stringify!(Job)))? {
            proxy::job::Inner::None(inner) => inner.try_into(),
            proxy::job::Inner::Programmer(inner) => inner.try_into(),
            proxy::job::Inner::Designer(inner) => inner.try_into(),
        }
    }
}
impl TryFrom<Option<proxy::Job>> for Job {
    type Error = Error;
    fn try_from(other: Option<proxy::Job>) -> Fallible<Self> {
        let proxy::Job { inner } = other
            .ok_or_else(|| format_err!("empty {} object", stringify!(proxy::Job)))?
            .try_into()?;
        match inner
            .ok_or_else(|| format_err!("{} doesn't have a value.", stringify!(proxy::Job)))?
        {
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
impl TryFrom<proxy::job::NoneInner> for Job {
    type Error = Error;
    fn try_from(_: proxy::job::NoneInner) -> Fallible<Self> {
        Ok(Job::None {})
    }
}
impl TryFrom<proxy::job::ProgrammerInner> for Job {
    type Error = Error;
    fn try_from(
        proxy::job::ProgrammerInner { skill, grade }: proxy::job::ProgrammerInner,
    ) -> Fallible<Self> {
        Ok(Job::Programmer {
            skill: skill.try_into()?,
            grade: grade.try_into()?,
        })
    }
}
impl TryFrom<proxy::Designer> for Job {
    type Error = Error;
    fn try_from(other: proxy::Designer) -> Fallible<Self> {
        Ok(Job::Designer(other.try_into()?))
    }
}
impl TryFrom<AreaCode> for proxy::AreaCode {
    type Error = Error;
    fn try_from(other: AreaCode) -> Fallible<Self> {
        Ok(match other {
            AreaCode::Seoul => proxy::AreaCode::Seoul,
            AreaCode::Seongnam => proxy::AreaCode::Seongnam,
            AreaCode::Jinhae => proxy::AreaCode::Jinhae,
        })
    }
}
impl TryFrom<proxy::AreaCode> for AreaCode {
    type Error = Error;
    fn try_from(other: proxy::AreaCode) -> Fallible<Self> {
        Ok(match other {
            proxy::AreaCode::Seoul => AreaCode::Seoul,
            proxy::AreaCode::Seongnam => AreaCode::Seongnam,
            proxy::AreaCode::Jinhae => AreaCode::Jinhae,
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
            .ok_or_else(|| format_err!("invalid {}", stringify!(AreaCode)))?;
        proxy.try_into()
    }
}
impl TryInto<Option<proxy::Person>> for Person {
    type Error = Error;
    fn try_into(self) -> Fallible<Option<proxy::Person>> {
        let Person {
            id,
            number,
            hobbies,
            job,
            city,
            area_code,
            ..
        } = self;
        Ok(Some(proxy::Person {
            id: id.try_into()?,
            number: number.try_into()?,
            hobbies: hobbies.try_into()?,
            job: job.try_into()?,
            city: city.try_into()?,
            area_code: area_code.try_into()?,
        }))
    }
}
impl TryInto<proxy::Person> for Person {
    type Error = Error;
    fn try_into(self) -> Fallible<proxy::Person> {
        let Person {
            id,
            number,
            hobbies,
            job,
            city,
            area_code,
            ..
        } = self;
        Ok(proxy::Person {
            id: id.try_into()?,
            number: number.try_into()?,
            hobbies: hobbies.try_into()?,
            job: job.try_into()?,
            city: city.try_into()?,
            area_code: area_code.try_into()?,
        })
    }
}
impl TryFrom<Option<proxy::Person>> for Person {
    type Error = Error;
    fn try_from(other: Option<proxy::Person>) -> Fallible<Self> {
        let proxy::Person {
            id,
            number,
            hobbies,
            job,
            city,
            area_code,
        } = other
            .ok_or_else(|| format_err!("empty {} object", stringify!(proxy::Person)))?
            .try_into()?;
        Ok(Self {
            id: id.try_into()?,
            number: number.try_into()?,
            hobbies: hobbies.try_into()?,
            job: job.try_into()?,
            city: city.try_into()?,
            area_code: area_code.try_into()?,
            _inner: Default::default(),
        })
    }
}
impl TryFrom<proxy::Person> for Person {
    type Error = Error;
    fn try_from(
        proxy::Person {
            id,
            number,
            hobbies,
            job,
            city,
            area_code,
        }: proxy::Person,
    ) -> Fallible<Self> {
        Ok(Self {
            id: id.try_into()?,
            number: number.try_into()?,
            hobbies: hobbies.try_into()?,
            job: job.try_into()?,
            city: city.try_into()?,
            area_code: area_code.try_into()?,
            _inner: Default::default(),
        })
    }
}
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
