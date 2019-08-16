#[macro_use]
extern crate proptest_derive;
#[macro_use]
extern crate failure;

pub mod city;
pub mod person;

pub mod proxy {
    include!("../proxy/yellow_book.rs");
}
