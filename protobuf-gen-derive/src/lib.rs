#[macro_use]
extern crate synstructure;

use proc_macro2::TokenStream;
use synstructure::Structure;

decl_derive!([ProtobufGen, attributes(protobuf_gen)] => derive_protobuf_gen);

fn derive_protobuf_gen(_: Structure) -> TokenStream {
    Default::default()
}
