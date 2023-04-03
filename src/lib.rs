#[macro_use]
extern crate log;
extern crate protobuf_gen_extract as extract;

pub mod error;
pub mod parse;
pub mod print;
mod types;

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::option_env;
use std::path::PathBuf;
use std::result;

use syn::{Ident, ItemEnum, ItemStruct};
use thiserror::Error;

use crate::parse::SchemaFile;
use crate::print::SchemaPrinter;
use crate::types::FieldType;
pub use error::Error;
pub use protobuf_gen_derive::*;

pub trait ProtobufGen: Sized {
    type Error;

    fn to_protobuf<W: Write>(self, w: &mut W) -> result::Result<(), Self::Error>;
    fn from_protobuf<R: Read>(r: &mut R) -> result::Result<Self, Self::Error>;
}

pub struct Config {
    pub proto_target_dir: PathBuf,
    pub proxy_target_dir: Option<PathBuf>,
    pub sources: HashMap<String, Vec<PathBuf>>,
    pub type_replacement: HashMap<String, String>,
    btree_map_targets: Vec<String>,
    additional_imports: HashMap<String, Vec<PathBuf>>,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("failed to read a file.")]
    IoError(#[from] io::Error),
    #[error("failed to parse a string.")]
    ParseError(#[from] syn::Error),
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
            btree_map_targets: Vec::new(),
            additional_imports: HashMap::new(),
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

    fn build_context(&self) -> result::Result<Context, ConfigError> {
        let mut context = Context::default();
        for (old, new) in &self.type_replacement {
            context.add_type_replacement(old.to_string(), new.to_string());
        }

        // generate item dictionary
        for (package, sources) in &self.sources {
            for source in sources {
                let file: syn::File = syn::parse_str(&fs::read_to_string(source)?)?;
                context.item_dictionary.collect(&file.items, package);
            }
        }
        Ok(context)
    }

    pub fn btree_map(&mut self, path: &str) {
        self.btree_map_targets.push(path.to_owned())
    }

    pub fn add_import(&mut self, package: impl Into<String>, path: impl Into<PathBuf>) {
        self.additional_imports
            .entry(package.into())
            .or_default()
            .push(path.into())
    }

    pub fn generate(&self) -> result::Result<(), ConfigError> {
        let mut in_files = Vec::new();
        let mut context = self.build_context()?;

        // generate protobuf schemas from Rust
        for (package, sources) in &self.sources {
            context.current_package = package.clone();

            let mut schema_file = if let Some(imports) = self.additional_imports.get(package) {
                SchemaFile::new(imports.clone())
            } else {
                SchemaFile::default()
            };

            schema_file.package = package.clone();
            for source in sources {
                eprintln!("processing {} in {}", source.display(), package);
                let syn_file: syn::File = syn::parse_str(&fs::read_to_string(source)?)?;
                schema_file.merge(&mut parse::build_schema_file(&context, &syn_file));
            }

            let (mut file, file_path) = self.create_proto_file(package)?;
            write!(file, "{}", SchemaPrinter(&schema_file))?;
            file.sync_all()?;

            // https://man7.org/linux/man-pages/man2/fdatasync.2.html
            //
            // > Calling fsync() does not necessarily ensure that the entry in the directory
            // > containing the file has also reached disk. For that an explicit fsync() on a file
            // > descriptor for the directory is also needed.
            if let Some(dir) = file_path.parent() {
                File::open(dir)?.sync_all()?;
            }

            in_files.push(file_path);
        }

        // http://localhost:8080/pipeline-syntax/globals#env, assuming a Jenkins controller is
        // running on localhost:8080.
        //
        // > A set of environment variables are made available to all Jenkins projects, including
        // > Pipelines. The following is a general list of variable names that are available.
        // > ...
        // > CI: Statically set to the string "true" to indicate a "continuous integration"
        // > execution environment.
        if option_env!("CI").filter(|&value| value == "true").is_some() {
            // HACK: Puts the current thread to sleep for a momemnt between writing and reading
            // `*.proto`. There appears to be a delay in Amazon EC2 until all in-memory data
            // reaches the filesystem. See <https://github.com/furiosa-ai/npu-tools/issues/2766>.
            //
            // https://man7.org/linux/man-pages/man2/fdatasync.2.html
            //
            // > The fsync() implementations in older kernels and lesser used filesystems do not
            // > know how to flush disk caches. In these cases disk caches need to be disabled
            // > using hdparm(8) or sdparm(8) to guarantee safe operation.
            std::thread::sleep(std::time::Duration::from_secs(2));
        }

        // generate Rust bindings for protobuf
        if let Some(ref proxy_target_dir) = self.proxy_target_dir {
            fs::create_dir_all(proxy_target_dir)?;

            let mut config = prost_build::Config::new();
            config.type_attribute(".", "#[allow(clippy::large_enum_variant)]");
            config.out_dir(proxy_target_dir);
            config.btree_map(&self.btree_map_targets);
            config.compile_protos(&in_files, &[PathBuf::from(&self.proto_target_dir)])?;
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ItemDictionary {
    package_map: HashMap<String, Vec<String>>,
}

impl ItemDictionary {
    pub fn collect(&mut self, items: &[syn::Item], package: &str) {
        for item in items {
            match item {
                syn::Item::Struct(ItemStruct { ident, .. })
                | syn::Item::Enum(ItemEnum { ident, .. }) => {
                    self.package_map
                        .entry(ident.to_string())
                        .or_default()
                        .push(package.to_string());
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
                ("isize".to_string(), FieldType::Int64),
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

    pub fn get_package(&self, ident: &Ident) -> Option<&String> {
        let packages = self.item_dictionary.package_map.get(&ident.to_string())?;
        if packages.contains(&self.current_package) {
            return None;
        }

        fn length_of_common_prefix(a: &str, b: &str) -> usize {
            a.chars().zip(b.chars()).take_while(|(a, b)| a == b).count()
        }

        packages
            .iter()
            .max_by_key(|package| length_of_common_prefix(package, &self.current_package))
    }
}
