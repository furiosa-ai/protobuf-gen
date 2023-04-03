use protobuf_gen::ProtobufGen;

#[derive(Debug, Default, Clone, ProtobufGen, PartialEq)]
#[protobuf_gen(proxy_mod = "crate::proxy::tree")]
pub enum Node {
    #[default]
    None,
    Tree {
        nodes: Vec<Node>,
    },
    Tree2 {
        nodes: Option<Node2>,
    },
}

#[derive(Debug, Default, Clone, ProtobufGen, PartialEq)]
#[protobuf_gen(proxy_mod = "crate::proxy::tree")]
pub struct Node2 {
    pub x: Option<usize>,
}

#[derive(Debug, Default, Clone, ProtobufGen, PartialEq)]
#[protobuf_gen(proxy_mod = "crate::proxy::tree")]
pub struct Tree {
    pub nodes: Vec<Node>,
}
