use lib_tests::city::City;
use lib_tests::person::*;
use proptest::prelude::*;
use std::io::Cursor;

use protobuf_gen::ProtobufGen;

#[test]
fn test_encode_decode_person() {
    let person = Person {
        _inner: 0,
        id: 23,
        number: vec![1, 2, 3].into(),
        hobbies: vec![7, 2],
        job: Job::Programmer {
            skill: "Rust".to_string(),
            grade: Some(3),
        },
        city: Some(City {
            name: "Seoul".to_string(),
        }),
        area_code: AreaCode::Seongnam,
        car: Car { number: 1213 },
        cars: vec![Car { number: 3 }, Car { number: 13 }, Car { number: 12 }],
    };

    do_test_encode_decode_person(person).unwrap();
}

#[test]
fn test_encode_decode_vec_of_opaque() {
    let vec_of_car = vec![Car { number: 1 }, Car { number: 2 }, Car { number: 3 }];
    let mut buffer = Vec::new();
    for car in &vec_of_car {
        car.clone()
            .to_protobuf_length_delimited(&mut buffer)
            .unwrap();
        dbg!(buffer.len());
    }
    let mut decoded = Vec::new();
    let mut cursor = Cursor::new(buffer);
    while dbg!(cursor.position()) < cursor.get_ref().len() as u64 {
        decoded.push(Car::from_protobuf_length_delimited(&mut cursor).unwrap());
    }
    assert_eq!(vec_of_car, decoded);
}

fn do_test_encode_decode_person(mut person: Person) -> eyre::Result<()> {
    let mut buffer = Vec::new();
    person.clone().to_protobuf(&mut buffer)?;
    let mut decoded = Person::from_protobuf(&mut Cursor::new(buffer))?;

    person._inner = 0;
    decoded._inner = 0;
    eyre::ensure!(person == decoded, "encoding failed");
    Ok(())
}

proptest::proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]
    #[test]
    fn proptest_encode_decode_person(person in any::<Person>()) {
        prop_assert!(do_test_encode_decode_person(person).is_ok());
    }
}
