use std::fmt;

use crate::types::{
    Enumerator, Field, FieldType, FileDescriptor, Frequency, Message, OneOf, Syntax,
};

pub struct SchemaPrinter<'a>(pub &'a FileDescriptor);

fn print_enum(e: &Enumerator, indent: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "{:indent$}enum {} {{", "", e.name, indent = indent)?;
    for (name, number) in &e.fields {
        writeln!(f, "{:indent$}  {} = {};", "", name, number, indent = indent)?;
    }
    writeln!(f, "{:indent$}}}", "", indent = indent)?;
    Ok(())
}

fn print_field(field: &Field, indent: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fn frequency_to_string(freq: &Frequency) -> &str {
        match freq {
            Frequency::Repeated => "repeated ",
            _ => "",
        }
    }

    fn type_to_string(typ: &FieldType) -> &str {
        match typ {
            FieldType::Int32 => "int32",
            FieldType::Sint32 => "sint32",
            FieldType::Int64 => "int64",
            FieldType::Sint64 => "sint64",
            FieldType::Uint32 => "uint32",
            FieldType::Uint64 => "uint64",
            FieldType::Bool => "bool",
            FieldType::Enum(_) => "enum",
            FieldType::Fixed32 => "fixed32",
            FieldType::Sfixed32 => "sfixed32",
            FieldType::Float => "float",
            FieldType::Fixed64 => "fixed64",
            FieldType::Sfixed64 => "sfixed64",
            FieldType::Double => "double",
            FieldType::String_ => "string",
            FieldType::Bytes_ => "bytes",
            FieldType::StringCow => "string",
            FieldType::BytesCow => "bytes",
            FieldType::Message(_) => "message",
            FieldType::Map(_, _) => "map",
            FieldType::MessageOrEnum(s) => &s,
        }
    }

    writeln!(
        f,
        "{:indent$}{}{} {} = {};",
        "",
        frequency_to_string(&field.frequency),
        type_to_string(&field.typ),
        field.name,
        field.number,
        indent = indent
    )
}

fn print_oneof(one_of: &OneOf, indent: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "{:indent$}oneof {} {{", "", one_of.name, indent = indent)?;
    for field in &one_of.fields {
        print_field(field, indent + 2, f)?;
    }
    writeln!(f, "{:indent$}}}", "", indent = indent)
}

fn print_message(message: &Message, indent: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(
        f,
        "{:indent$}message {} {{",
        "",
        message.name,
        indent = indent
    )?;

    for e in &message.enums {
        print_enum(e, indent + 2, f)?;
    }

    for msg in &message.messages {
        print_message(msg, indent + 2, f)?;
        writeln!(f)?;
    }

    for oneof in &message.oneofs {
        print_oneof(oneof, indent + 2, f)?;
        writeln!(f)?;
    }

    for field in &message.fields {
        print_field(field, indent + 2, f)?;
    }
    writeln!(f, "{:indent$}}}", "", indent = indent)?;
    Ok(())
}

impl<'a> fmt::Display for SchemaPrinter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.syntax {
            Syntax::Proto2 => {
                writeln!(f, "syntax = \"proto2\";\n")?;
            }
            Syntax::Proto3 => {
                writeln!(f, "syntax = \"proto3\";\n")?;
            }
        }

        writeln!(f, "package {};\n", self.0.package)?;
        for path in &self.0.import_paths {
            writeln!(f, "import \"{}\";", path.display())?;
        }
        writeln!(f)?;

        for e in &self.0.enums {
            print_enum(e, 0, f)?;
        }
        writeln!(f)?;

        for m in &self.0.messages {
            print_message(m, 0, f)?;
            writeln!(f)?;
        }
        Ok(())
    }
}
