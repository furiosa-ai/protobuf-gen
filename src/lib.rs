#![recursion_limit = "128"]

#[macro_use]
extern crate log;
extern crate proc_macro2;
#[allow(unused_imports)]
#[macro_use]
extern crate protobuf_gen_derive;
#[macro_use]
extern crate quote;

pub mod convert;
pub mod extract;
pub mod parse;
pub mod print;

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use failure::Fallible;
use pb_rs::types::FieldType;

use crate::parse::SchemaFile;
use crate::print::SchemaPrinter;
pub use protobuf_gen_derive::*;

pub trait ProtobufGen: Sized {
    fn to_protobuf<W: Write>(self, w: &mut W) -> Fallible<()>;
    fn from_protobuf<R: Read>(r: &mut R) -> Fallible<Self>;
}

pub struct Config {
    pub proto_target_dir: PathBuf,
    pub proxy_target_dir: Option<PathBuf>,
    pub sources: HashMap<String, Vec<PathBuf>>,
    pub type_replacement: HashMap<String, String>,
}

impl Config {
    pub fn new<P: Into<PathBuf>, Q: Into<PathBuf>>(
        proto_target_dir: P,
        proxy_target_dir: Option<Q>,
    ) -> Self {
        Self {
            proto_target_dir: proto_target_dir.into(),
            proxy_target_dir: proxy_target_dir.map(|p| p.into()),
            sources: HashMap::new(),
            type_replacement: HashMap::new(),
        }
    }

    pub fn replace_type(&mut self, old: String, new: String) {
        self.type_replacement.insert(old, new);
    }

    pub fn add_source<P: Into<PathBuf>, S: Into<String>>(&mut self, file: P, package: S) {
        self.sources
            .entry(package.into())
            .or_default()
            .push(file.into());
    }

    fn create_proto_file<P: AsRef<str>>(&self, package: P) -> io::Result<(File, PathBuf)> {
        let package: Vec<_> = package.as_ref().split('.').collect();
        let (dir, file) = package.split_at(package.len() - 1);

        let mut dir_path = self.proto_target_dir.clone();
        dir_path.extend(dir);

        let mut file_path = dir_path.as_path().join(PathBuf::from(file[0].to_string()));
        file_path.set_extension("proto");

        fs::create_dir_all(dir_path)?;
        Ok((File::create(file_path.as_path())?, file_path))
    }

    fn build_context(&self) -> Fallible<Context> {
        let mut context = Context::default();
        for (old, new) in &self.type_replacement {
            context.add_type_replacement(old.to_string(), new.to_string());
        }

        // generate item dictionary
        for (package, sources) in &self.sources {
            for source in sources {
                let file: syn::File = syn::parse_str(&fs::read_to_string(source)?).unwrap();
                context.item_dictionary.collect(&file.items, package);
            }
        }
        Ok(context)
    }

    pub fn generate(&self) -> Fallible<()> {
        let mut in_files = Vec::new();

        let mut file = File::create("conversion")?;
        // generate protobuf schemas from Rust
        for (package, sources) in &self.sources {
            let mut context = self.build_context()?;
            let mut schema_file = SchemaFile::default();
            schema_file.package = package.clone();
            context.current_package = package.clone();
            for source in sources {
                debug!("processing {} in {}", source.display(), package);
                let syn_file: syn::File = syn::parse_str(&fs::read_to_string(source)?).unwrap();

                schema_file.import_paths.extend(
                    parse::collect_required_imports(&context, &syn_file)
                        .into_iter()
                        .map(|s| Path::new(&s.replace(".", "/")).with_extension("proto")),
                );

                schema_file.merge(&mut parse::build_schema_file(&context, &syn_file));
            }

            for source in sources {
                let syn_file: syn::File = syn::parse_str(&fs::read_to_string(source)?).unwrap();
                convert::generate_conversion_apis(&schema_file, &syn_file, &mut file)?;
            }

            let (mut file, file_path) = self.create_proto_file(package)?;
            write!(file, "{}", SchemaPrinter(&schema_file))?;

            in_files.push(file_path);
        }

        // generate Rust bindings for protobuf
        if let Some(ref proxy_target_dir) = self.proxy_target_dir {
            fs::create_dir_all(proxy_target_dir)?;

            let mut config = prost_build::Config::new();
            config.out_dir(proxy_target_dir);
            config
                .compile_protos(&in_files, &[PathBuf::from(&self.proto_target_dir)])
                .unwrap();
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ItemDictionary {
    package_map: HashMap<String, String>,
}

impl ItemDictionary {
    pub fn collect(&mut self, items: &[syn::Item], package: &str) {
        for item in items {
            match item {
                syn::Item::Struct(inner) => {
                    self.package_map
                        .insert(inner.ident.to_string(), package.to_string());
                }
                syn::Item::Enum(inner) => {
                    self.package_map
                        .insert(inner.ident.to_string(), package.to_string());
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug)]
pub struct Context {
    current_package: String,
    type_replacement: HashMap<String, FieldType>,
    item_dictionary: ItemDictionary,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            type_replacement: vec![
                ("f64".to_string(), FieldType::Double),
                ("f32".to_string(), FieldType::Float),
                ("i8".to_string(), FieldType::Int32),
                ("i16".to_string(), FieldType::Int32),
                ("i32".to_string(), FieldType::Int32),
                ("i64".to_string(), FieldType::Int64),
                ("u8".to_string(), FieldType::Uint32),
                ("u16".to_string(), FieldType::Uint32),
                ("u32".to_string(), FieldType::Uint32),
                ("u64".to_string(), FieldType::Uint64),
                ("usize".to_string(), FieldType::Uint64),
                ("i32".to_string(), FieldType::Sint32),
                ("i64".to_string(), FieldType::Sint64),
                ("String".to_string(), FieldType::String_),
            ]
            .into_iter()
            .collect(),
            current_package: Default::default(),
            item_dictionary: Default::default(),
        }
    }
}

impl Context {
    pub fn add_type_replacement(&mut self, old: String, new: String) {
        self.type_replacement
            .insert(old, FieldType::MessageOrEnum(new));
    }
}
