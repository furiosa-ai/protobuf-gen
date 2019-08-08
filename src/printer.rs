use std::collections::HashMap;
use std::fmt;

use heck::SnakeCase;
use syn::{
    self, Field, Fields, File, GenericArgument, Ident, Item, ItemEnum, ItemStruct, PathArguments,
    Type, TypePath, Variant, Visibility,
};

type ItemDictionary = HashMap<Ident, Item>;

fn build_item_dictionary(items: &[Item]) -> ItemDictionary {
    let mut dict = HashMap::new();
    for item in items {
        match item {
            Item::Struct(inner) => {
                dict.insert(inner.ident.clone(), item.clone());
            }
            Item::Type(inner) => {
                dict.insert(inner.ident.clone(), item.clone());
            }
            Item::Enum(inner) => {
                dict.insert(inner.ident.clone(), item.clone());
            }
            _ => {}
        }
    }
    dict
}

fn collect_target_items(items: &[Item]) -> Vec<&Item> {
    items
        .iter()
        .filter(|item| match item {
            Item::Struct(_) | Item::Enum(_) => true,
            _ => false,
        })
        .collect()
}

fn type_path_ident(type_path: &TypePath) -> &Ident {
    &type_path.path.segments.last().unwrap().value().ident
}

fn generic_type_of(type_path: &TypePath) -> Option<&Type> {
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

lazy_static! {
    static ref TYPE_REPLACEMENT: HashMap<&'static str, &'static str> = vec![
        ("f64", "double"),
        ("f32", "float"),
        ("i8", "int32"),
        ("i16", "int32"),
        ("i32", "int32"),
        ("i64", "int64"),
        ("u8", "uint32"),
        ("u16", "uint32"),
        ("u32", "uint32"),
        ("u64", "uint64"),
        ("i32", "sint32"),
        ("i64", "sint64"),
        ("String", "string"),
    ]
    .into_iter()
    .collect();
}

#[derive(new)]
pub struct TypePrinter<'a> {
    inner: &'a Type,
}

impl<'a> fmt::Display for TypePrinter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.inner {
            Type::Array(type_array) => {
                write!(f, "repeated {}", Self::new(&*type_array.elem))?;
            }
            Type::Path(type_path) => {
                let ident = type_path_ident(type_path);
                if let Some(ty) = TYPE_REPLACEMENT.get(ident.to_string().as_str()) {
                    write!(f, "{}", ty)?;
                } else if ident == "Vec" || ident == "HashSet" {
                    write!(
                        f,
                        "repeated {}",
                        Self::new(generic_type_of(&type_path).unwrap())
                    )?;
                } else {
                    write!(f, "{}", ident)?;
                }
            }
            _ => panic!(),
        }
        Ok(())
    }
}

#[derive(new)]
pub struct FieldPrinter<'a> {
    field: &'a Field,
    number: usize,
}

impl<'a> fmt::Display for FieldPrinter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(substitute) = syn_util::get_attribute_value::<String>(
            &self.field.attrs,
            &["protobuf_gen", "substitute"],
        ) {
            writeln!(
                f,
                "{} {} = {};",
                substitute,
                self.field.ident.as_ref().unwrap(),
                self.number
            )?;
            return Ok(());
        }

        writeln!(
            f,
            "{} {} = {};",
            TypePrinter::new(&self.field.ty),
            self.field.ident.as_ref().unwrap(),
            self.number
        )?;
        Ok(())
    }
}

#[derive(new)]
pub struct VariantPrinter<'a> {
    variant: &'a Variant,
    number: usize,
}

impl<'a> fmt::Display for VariantPrinter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.variant.fields {
            Fields::Unnamed(_) => {
                writeln!(
                    f,
                    "{} {} = {};",
                    self.variant.ident,
                    self.variant.ident.to_string().to_snake_case(),
                    self.number
                )?;
            }
            Fields::Named(_) | Fields::Unit => {
                writeln!(
                    f,
                    "{}Inner {} = {};",
                    self.variant.ident,
                    self.variant.ident.to_string().to_snake_case(),
                    self.number
                )?;
            }
        }
        Ok(())
    }
}

fn filter_field(field: &Field) -> bool {
    if let Visibility::Public(_) = field.vis {
        true
    } else {
        false
    }
}

#[derive(new)]
pub struct StructPrinter<'a> {
    inner: &'a ItemStruct,
}

impl<'a> fmt::Display for StructPrinter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "message {} {{", self.inner.ident)?;
        match &self.inner.fields {
            Fields::Named(fields_named) => {
                let mut field_number = 1;
                for field in fields_named.named.iter().cloned().filter(filter_field) {
                    let field_string = FieldPrinter::new(&field, field_number).to_string();
                    if !field_string.is_empty() {
                        write!(f, "  {}", field_string)?;
                        field_number += 1;
                    }
                }
            }
            _ => {}
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}

#[derive(new)]
pub struct EnumPrinter<'a> {
    inner: &'a ItemEnum,
}

impl<'a> fmt::Display for EnumPrinter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let field_is_unit = |f: &Fields| {
            if let Fields::Unit = f {
                true
            } else {
                false
            }
        };

        if self.inner.variants.iter().all(|v| field_is_unit(&v.fields)) {
            writeln!(f, "enum {} {{", self.inner.ident)?;
            for (i, variant) in self.inner.variants.iter().enumerate() {
                writeln!(f, "  {} = {};", variant.ident, i)?;
            }
            writeln!(f, "}}")?;
        } else {
            self.extract_inner_types(f, self.inner)?;

            writeln!(f, "message {} {{", self.inner.ident)?;
            writeln!(
                f,
                "  oneof {}_inner {{",
                self.inner.ident.to_string().to_snake_case()
            )?;
            for (i, variant) in self.inner.variants.iter().enumerate() {
                write!(f, "    {}", VariantPrinter::new(variant, i + 1))?;
            }
            writeln!(f, "  }}")?;
            writeln!(f, "}}")?;
        }
        Ok(())
    }
}

impl<'a> EnumPrinter<'a> {
    pub fn extract_inner_types(
        &self,
        f: &mut fmt::Formatter<'_>,
        item_enum: &ItemEnum,
    ) -> fmt::Result {
        for variant in &item_enum.variants {
            match &variant.fields {
                Fields::Named(fields_named) => {
                    writeln!(f, "message {}Inner {{", variant.ident)?;
                    for (i, field) in fields_named.named.iter().enumerate() {
                        let field_string = FieldPrinter::new(&field, i + 1).to_string();
                        if !field_string.is_empty() {
                            write!(f, "  {}", field_string)?;
                        }
                    }
                    writeln!(f, "}}")?;
                }
                Fields::Unit => {
                    writeln!(f, "message {}Inner {{", variant.ident)?;
                    writeln!(f, "}}")?;
                }
                _ => {}
            }
        }
        Ok(())
    }
}

#[derive(new)]
pub struct ItemPrinter<'a> {
    item: &'a Item,
}

impl<'a> fmt::Display for ItemPrinter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.item {
            Item::Struct(item_struct) => {
                writeln!(f, "{}", StructPrinter::new(item_struct))?;
            }
            Item::Enum(item_enum) => {
                writeln!(f, "{}", EnumPrinter::new(item_enum))?;
            }
            _ => {}
        }
        Ok(())
    }
}

pub struct FilePrinter {
    file: File,
}

impl fmt::Display for FilePrinter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in collect_target_items(&self.file.items) {
            writeln!(f, "{}", ItemPrinter::new(item))?;
        }
        Ok(())
    }
}

impl FilePrinter {
    pub fn new(text: &str) -> Self {
        let file: File = syn::parse_str(text).unwrap();
        build_item_dictionary(&file.items);
        Self { file }
    }
}
