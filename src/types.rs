use std::path::PathBuf;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Syntax {
    Proto2,
    #[default]
    Proto3,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Frequency {
    Optional,
    Repeated,
    Required,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct MessageIndex {
    indexes: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct EnumIndex {
    msg_index: MessageIndex,
    index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FieldType {
    Int32,
    Int64,
    Uint32,
    Uint64,
    Sint32,
    Sint64,
    Bool,
    Enum(EnumIndex),
    Fixed64,
    Sfixed64,
    Double,
    StringCow,
    BytesCow,
    String_,
    Bytes_,
    Message(MessageIndex),
    MessageOrEnum(String),
    Fixed32,
    Sfixed32,
    Float,
    Map(Box<FieldType>, Box<FieldType>),
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub frequency: Frequency,
    pub typ: FieldType,
    pub number: i32,
    pub default: Option<String>,
    pub packed: Option<bool>,
    pub boxed: bool,
    pub deprecated: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Message {
    pub name: String,
    pub fields: Vec<Field>,
    pub oneofs: Vec<OneOf>,
    pub reserved_nums: Option<Vec<i32>>,
    pub reserved_names: Option<Vec<String>>,
    pub imported: bool,
    pub package: String,        // package from imports + nested items
    pub messages: Vec<Message>, // nested messages
    pub enums: Vec<Enumerator>, // nested enums
    pub module: String,         // 'package' corresponding to actual generated Rust module
    pub path: PathBuf,
    pub import: PathBuf,
    pub index: MessageIndex,
}

#[derive(Debug, Clone, Default)]
pub struct Enumerator {
    pub name: String,
    pub fields: Vec<(String, i32)>,
    pub fully_qualified_fields: Vec<(String, i32)>,
    pub partially_qualified_fields: Vec<(String, i32)>,
    pub imported: bool,
    pub package: String,
    pub module: String,
    pub path: PathBuf,
    pub import: PathBuf,
    pub index: EnumIndex,
}

#[derive(Debug, Clone, Default)]
pub struct OneOf {
    pub name: String,
    pub fields: Vec<Field>,
    pub package: String,
    pub module: String,
    pub imported: bool,
}

#[derive(Debug, Default, Clone)]
pub struct FileDescriptor {
    pub import_paths: Vec<PathBuf>,
    pub package: String,
    pub syntax: Syntax,
    pub messages: Vec<Message>,
    pub enums: Vec<Enumerator>,
    pub module: String,
    pub owned: bool,
}
