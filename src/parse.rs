use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use std::path::Path;

use crate::types::{
    Enumerator, Field, FieldType, FileDescriptor, Frequency, Message, OneOf, Syntax,
};
use syn::visit::{self, Visit};
use syn::{
    self, Fields, FieldsNamed, File, GenericArgument, Ident, ItemEnum, ItemStruct, PathArguments,
    Type, TypePath, Variant,
};

use super::Context;
use crate::extract::{self, Extract};

struct RequiredImportsCollector<'a> {
    context: &'a Context,
    imports: HashSet<String>,
}

impl<'a> Extract for RequiredImportsCollector<'a> {
    fn extract_message_with_fields_named<'ast>(
        &mut self,
        _: &ItemStruct,
        fields_named: &'ast FieldsNamed,
    ) {
        self.visit_fields_named(fields_named);
    }
    fn extract_nested_message_with_fields_named(
        &mut self,
        _: &ItemEnum,
        _: &Variant,
        fields_named: &FieldsNamed,
    ) {
        self.visit_fields_named(fields_named);
    }
}

impl<'a, 'ast> Visit<'ast> for RequiredImportsCollector<'a> {
    fn visit_type_path(&mut self, type_path: &TypePath) {
        let ident = type_path_ident(type_path);
        if !self
            .context
            .type_replacement
            .contains_key(ident.to_string().as_str())
        {
            if let Some(package) = self
                .context
                .item_dictionary
                .package_map
                .get(&ident.to_string())
                .filter(|&package| package != &self.context.current_package)
            {
                self.imports.insert(package.to_string());
            }
        }
        visit::visit_type_path(self, type_path);
    }
}

pub fn collect_required_imports<'a>(context: &'a Context, file: &File) -> HashSet<String> {
    let mut collector = RequiredImportsCollector {
        context,
        imports: Default::default(),
    };
    extract::extract_from_file(&mut collector, file);
    collector.imports
}

struct SchemaFileBuilder<'a> {
    context: &'a Context,
    file_descriptor: FileDescriptor,
}

impl<'a> Extract for SchemaFileBuilder<'a> {
    fn extract_message_with_fields_named(
        &mut self,
        item_struct: &ItemStruct,
        fields_named: &FieldsNamed,
    ) {
        let fields = fields_named
            .named
            .iter()
            .enumerate()
            .map(|(i, field)| self.field_to_schema(field, i + 1))
            .collect();
        self.add_message(Message {
            name: item_struct.ident.to_string(),
            fields,
            ..Message::default()
        });
    }

    fn extract_nested_message_with_fields_named(
        &mut self,
        item_enum: &ItemEnum,
        variant: &Variant,
        fields_named: &FieldsNamed,
    ) {
        let fields = fields_named
            .named
            .iter()
            .enumerate()
            .map(|(i, field)| self.field_to_schema(field, i + 1))
            .collect();
        self.add_nested_message(
            &item_enum.ident,
            Message {
                name: format!("{}Inner", &variant.ident),
                fields,
                ..Message::default()
            },
        );
    }

    fn extract_nested_message_with_fields_unit(&mut self, item_enum: &ItemEnum, variant: &Variant) {
        self.add_nested_message(
            &item_enum.ident,
            Message {
                name: format!("{}Inner", &variant.ident),
                ..Message::default()
            },
        );
    }

    fn extract_one_of(&mut self, item_enum: &ItemEnum) {
        let fields: Vec<_> = item_enum
            .variants
            .iter()
            .enumerate()
            .map(|(i, variant)| {
                let i = i + 1;
                if let Fields::Unnamed(fields_unnamed) = &variant.fields {
                    let mut field = fields_unnamed.unnamed.clone().pop().unwrap().into_value();
                    field.ident = Some(variant.ident.clone());
                    self.field_to_schema(&field, i)
                } else {
                    Field {
                        name: variant.ident.to_string(),
                        typ: FieldType::MessageOrEnum(format!("{}Inner", variant.ident)),
                        number: i as i32,
                        frequency: Frequency::Required,
                        default: None,
                        packed: None,
                        boxed: false,
                        deprecated: false,
                    }
                }
            })
            .collect();

        self.add_message(Message {
            name: item_enum.ident.to_string(),
            oneofs: vec![OneOf {
                name: "inner".to_string(),
                fields,
                ..OneOf::default()
            }],
            ..Message::default()
        });
    }

    fn extract_enumerator(&mut self, item_enum: &ItemEnum) {
        let fields = item_enum
            .variants
            .iter()
            .enumerate()
            .map(|(i, variant)| (variant.ident.to_string(), i as i32))
            .collect();
        self.add_enum(Enumerator {
            name: item_enum.ident.to_string(),
            fields,
            ..Enumerator::default()
        });
    }
}

pub(crate) fn type_path_ident(type_path: &TypePath) -> &Ident {
    &type_path.path.segments.last().unwrap().value().ident
}

pub(crate) fn generic_type_of(type_path: &TypePath) -> Option<&Type> {
    type_path.path.segments.last().and_then(|x| {
        if let PathArguments::AngleBracketed(g) = &x.value().arguments {
            g.args.first().and_then(|x| {
                if let GenericArgument::Type(g) = x.value() {
                    Some(g)
                } else {
                    None
                }
            })
        } else {
            None
        }
    })
}

fn type_frequency(typ: &Type) -> Frequency {
    match typ {
        Type::Array(_) => Frequency::Repeated,
        Type::Path(type_path) => {
            let ident = type_path_ident(&type_path);
            if ident == "Vec" || ident == "HashSet" {
                Frequency::Repeated
            } else {
                Frequency::Required
            }
        }
        _ => panic!(),
    }
}

impl<'a> SchemaFileBuilder<'a> {
    fn add_message(&mut self, m: Message) {
        debug!("Message {}", m.name);
        self.file_descriptor.messages.push(m);
    }

    fn add_nested_message(&mut self, parent: &Ident, m: Message) {
        debug!("Message(in {}) {}", parent, m.name);
        self.file_descriptor
            .messages
            .iter_mut()
            .find(|m| parent == &m.name)
            .unwrap_or_else(|| panic!("no parent message \"{}\" exists.", parent))
            .messages
            .push(m);
    }

    fn add_enum(&mut self, e: Enumerator) {
        debug!("Enumerator {}", e.name);
        self.file_descriptor.enums.push(e);
    }

    fn type_field_type(&self, typ: &Type) -> FieldType {
        match typ {
            Type::Array(type_array) => self.type_field_type(&type_array.elem),
            Type::Path(type_path) => {
                let ident = type_path_ident(&type_path);
                if let Some(ty) = self
                    .context
                    .type_replacement
                    .get(ident.to_string().as_str())
                {
                    ty.clone()
                } else if ident == "Vec" || ident == "HashSet" {
                    self.type_field_type(generic_type_of(&type_path).unwrap())
                } else if let Some(package) = self
                    .context
                    .item_dictionary
                    .package_map
                    .get(&ident.to_string())
                    .filter(|&package| package != &self.context.current_package)
                {
                    FieldType::MessageOrEnum(package.clone() + "." + &ident.to_string())
                } else {
                    FieldType::MessageOrEnum(ident.to_string())
                }
            }
            _ => panic!("failed to parse: {:?}", typ),
        }
    }

    fn field_to_schema(&self, field: &syn::Field, number: usize) -> Field {
        if let Some(substitute) =
            syn_util::get_attribute_value::<String>(&field.attrs, &["protobuf_gen", "substitute"])
        {
            return Field {
                name: field.ident.as_ref().unwrap().to_string(),
                frequency: Frequency::Required,
                typ: FieldType::MessageOrEnum(substitute),
                number: number as i32,
                default: None,
                packed: None,
                boxed: false,
                deprecated: false,
            };
        }

        Field {
            name: field.ident.as_ref().unwrap().to_string(),
            frequency: type_frequency(&field.ty),
            typ: self.type_field_type(&field.ty),
            number: number as i32,
            default: None,
            packed: None,
            boxed: false,
            deprecated: false,
        }
    }
}

pub fn build_schema_file<'a>(context: &'a Context, file: &File) -> SchemaFile {
    let file_descriptor = FileDescriptor {
        syntax: Syntax::Proto3,
        import_paths: collect_required_imports(&context, &file)
            .into_iter()
            .map(|s| Path::new(&s.replace(".", "/")).with_extension("proto"))
            .collect(),
        ..Default::default()
    };

    let mut builder = SchemaFileBuilder {
        context,
        file_descriptor,
    };
    extract::extract_from_file(&mut builder, file);

    SchemaFile(builder.file_descriptor)
}

pub struct SchemaFile(FileDescriptor);

impl Deref for SchemaFile {
    type Target = FileDescriptor;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SchemaFile {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for SchemaFile {
    fn default() -> Self {
        Self(FileDescriptor {
            syntax: Syntax::Proto3,
            ..Default::default()
        })
    }
}

impl SchemaFile {
    pub fn merge(&mut self, other: &mut SchemaFile) {
        self.0.import_paths.append(&mut other.0.import_paths);
        self.0.import_paths.dedup();
        self.0.enums.append(&mut other.0.enums);
        self.0.messages.append(&mut other.0.messages);
    }

    pub fn release(self) -> FileDescriptor {
        self.0
    }
}
