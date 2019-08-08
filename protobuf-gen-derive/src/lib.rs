#![recursion_limit = "128"]

extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate synstructure;
extern crate syn_util;

use proc_macro2::TokenStream;
use syn::{Attribute, Expr, Lit, LitStr};
use syn_util::{contains_attribute, get_attribute_value};
use synstructure::{BindStyle::Ref, BindStyle::RefMut, Structure};

const OPCODE_BITS: usize = 7;

decl_derive!([ProtobufGen, attributes(protobuf_gen)] => derive_protobuf_gen);

fn derive_protobuf_gen(mut s: Structure) -> TokenStream {
    Default::default()
}
