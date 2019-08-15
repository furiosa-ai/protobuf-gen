use std::io::{self, Write};

use heck::SnakeCase;
use pb_rs::types::{FileDescriptor, Message};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{self, Fields, FieldsNamed, File, Ident, ItemEnum, ItemStruct, Type, TypePath, Variant};

use super::{extract, parse};
use extract::Extract;

struct ConversionGenerator<'a> {
    file_descriptor: &'a FileDescriptor,
    token_stream: TokenStream,
}

impl<'a> Extract for ConversionGenerator<'a> {
    fn extract_message_with_fields_named(
        &mut self,
        item_struct: &ItemStruct,
        fields_named: &FieldsNamed,
    ) {
        let ident = &item_struct.ident;
        let proxy = quote!(proxy);

        let (ref bindings, ref from_proxy, ref to_proxy) =
            self.generate_assignments(ident, fields_named);

        self.token_stream.extend(quote! {
            impl TryFrom<#ident> for #proxy::#ident {
                type Error = Error;

                fn try_from(#ident { #(#bindings)* .. }: #ident) -> Fallible<Self> {
                    Ok(Self {
                        #(#to_proxy)*
                    })
                }
            }
        });

        self.token_stream.extend(quote! {
            impl TryFrom<#proxy::#ident> for #ident {
                type Error = Error;

                fn try_from(#proxy::#ident { #(#bindings)* }: #proxy::#ident) -> Fallible<Self> {
                    Ok(Self {
                        #(#from_proxy)*
                    })
                }
            }
        });

        self.add_derive_protobuf_gen(ident, proxy);
    }

    fn extract_message_with_fields_unit(&mut self, item_struct: &ItemStruct) {
        let ident = &item_struct.ident;
        let proxy = quote!(proxy);

        self.token_stream.extend(quote! {
            impl TryFrom<#ident> for #proxy::#ident {
                type Error = Error;

                fn try_from(_: #ident) -> Fallible<Self> {
                    Ok(Self {})
                }
            }

            impl TryFrom<#proxy::#ident> for #ident {
                type Error = Error;

                fn try_from(_: #proxy::#ident) -> Fallible<Self> {
                    Ok(Self {})
                }
            }
        });

        self.add_derive_protobuf_gen(ident, proxy);
    }

    fn extract_nested_message_with_fields_named(
        &mut self,
        item_enum: &ItemEnum,
        variant: &Variant,
        fields_named: &FieldsNamed,
    ) {
        let ident = &item_enum.ident;
        let proxy = quote!(proxy);

        let (bindings, assignments, _) = self.generate_assignments(ident, fields_named);

        let inner_mod: Ident = syn::parse_str(&ident.to_string().to_snake_case()).unwrap();
        let variant_inner: Ident = syn::parse_str(&format!("{}Inner", variant.ident)).unwrap();
        let variant_ident = &variant.ident;

        self.token_stream.extend(quote! {
            impl TryFrom<#proxy::#inner_mod::#variant_inner> for #ident {
                type Error = Error;

                fn try_from(#proxy::#inner_mod::#variant_inner { #(#bindings)* }: #proxy::#inner_mod::#variant_inner) -> Fallible<Self> {
                    Ok(#ident::#variant_ident {
                        #(#assignments)*
                    })
                }
            }
        });
    }

    fn extract_nested_message_with_fields_unit(&mut self, item_enum: &ItemEnum, variant: &Variant) {
        let ident = &item_enum.ident;
        let proxy = quote!(proxy);

        let inner_mod: Ident = syn::parse_str(&ident.to_string().to_snake_case()).unwrap();
        let variant = &variant.ident;
        let variant_inner: Ident = syn::parse_str(&format!("{}Inner", variant)).unwrap();

        self.token_stream.extend(quote! {
            impl TryFrom<#proxy::#inner_mod::#variant_inner> for #ident {
                type Error = Error;

                fn try_from(_: #proxy::#inner_mod::#variant_inner) -> Fallible<Self> {
                    Ok(#ident::#variant {})
                }
            }
        });
    }

    fn extract_one_of(&mut self, item_enum: &ItemEnum) {
        let ident = &item_enum.ident;
        let proxy = quote!(proxy);
        let inner_mod: Ident = syn::parse_str(&ident.to_string().to_snake_case()).unwrap();

        let cases = item_enum.variants.iter().map(|v| {
            let variant = &v.ident;
            let variant_inner: Ident = syn::parse_str(&format!("{}Inner", variant)).unwrap();
            match &v.fields {
                Fields::Unit => quote!{
                    #ident::#variant {} => #proxy::#ident {
                        inner: Some(#proxy::#inner_mod::Inner::#variant(proxy::job::#variant_inner {})),
                    },
                },
                Fields::Named(fields_named) => {
                    let (bindings, assignments, _) = self.generate_assignments(ident, fields_named);
                    quote!{
                        #ident::#variant { #(#bindings)* } => #proxy::#ident {
                            inner: Some(#proxy::#inner_mod::Inner::#variant(#proxy::#inner_mod::#variant_inner {
                                #(#assignments)*
                            })),
                        },
                    }
                },
                Fields::Unnamed(_) => quote!{
                    #ident::#variant(inner) => #proxy::#ident {
                        inner: Some(#proxy::#inner_mod::Inner::#variant(inner.try_into()?)),
                    },
                },
            }
        });

        self.token_stream.extend(quote! {
            impl TryFrom<#ident> for #proxy::#ident {
                type Error = Error;

                fn try_from(other: #ident) -> Fallible<Self> {
                    Ok(match other {
                        #(#cases)*
                    })
                }
            }
        });

        let cases = item_enum.variants.iter().map(|v| {
            let variant = &v.ident;
            quote!(#proxy::#inner_mod::Inner::#variant(inner) => inner.try_into(),)
        });

        self.token_stream.extend(quote! {
            impl TryFrom<#proxy::#ident> for #ident {
                type Error = Error;

                fn try_from(#proxy::#ident { inner }: #proxy::#ident) -> Fallible<Self> {
                    match inner.ok_or_else(|| format_err!("{} doesn't have a value.", stringify!(#ident)))? {
                        #(#cases)*
                    }
                }
            }
        });

        self.add_derive_protobuf_gen(ident, proxy);
    }

    fn extract_enumerator(&mut self, item_enum: &ItemEnum) {
        let ident = &item_enum.ident;
        let proxy = quote!(proxy);

        let cases = item_enum.variants.iter().map(|v| {
            let variant = &v.ident;
            quote!(#ident::#variant => #proxy::#ident::#variant,)
        });

        self.token_stream.extend(quote! {
            impl TryFrom<#ident> for #proxy::#ident {
                type Error = Error;

                fn try_from(other: #ident) -> Fallible<Self> {
                    Ok(match other {
                        #(#cases)*
                    })
                }
            }
        });

        let cases = item_enum.variants.iter().map(|v| {
            let variant = &v.ident;
            quote!(#proxy::#ident::#variant => #ident::#variant,)
        });

        self.token_stream.extend(quote! {
            impl TryFrom<#proxy::#ident> for #ident {
                type Error = Error;

                fn try_from(other: #proxy::#ident) -> Fallible<Self> {
                    Ok(match other {
                        #(#cases)*
                    })
                }
            }
        });

        self.add_derive_protobuf_gen(ident, proxy);
    }
}

impl<'a> ConversionGenerator<'a> {
    fn add_derive_protobuf_gen<T, U>(&mut self, ident: T, proxy: U)
    where
        T: ToTokens,
        U: ToTokens,
    {
        self.token_stream.extend(quote! {
            impl ProtobufGen for #ident {
                fn to_protobuf<W: Write>(self, w: &mut W) -> Fallible<()> {
                    let proxy: #proxy::#ident = self.try_into()?;
                    let mut buffer = Vec::with_capacity(proxy.encoded_len());
                    proxy.encode(&mut buffer)?;
                    w.write_all(&buffer)?;
                    Ok(())
                }

                fn from_protobuf<R: Read>(r: &mut R) -> Fallible<Self> {
                    let mut buffer = Vec::new();
                    r.read_to_end(&mut buffer)?;
                    let proxy: #proxy::#ident = Message::decode(buffer)?;
                    proxy.try_into()
                }
            }
        });
    }

    fn is_optional(&self, typ: &Type) -> bool {
        fn is_optional_impl(type_path: &TypePath, messages: &[Message]) -> bool {
            messages.iter().any(|m| {
                parse::type_path_ident(type_path) == &m.name
                    || is_optional_impl(type_path, &m.messages)
            })
        }

        if let Type::Path(type_path) = typ {
            is_optional_impl(type_path, &self.file_descriptor.messages)
        } else {
            false
        }
    }

    fn generate_assignments(
        &self,
        ident: &Ident,
        fields_named: &FieldsNamed,
    ) -> (Vec<TokenStream>, Vec<TokenStream>, Vec<TokenStream>) {
        let bindings = fields_named
            .named
            .iter()
            .map(|x| {
                let field = x.ident.as_ref().unwrap();
                quote!(#field,)
            })
            .collect();

        let from_proxy = fields_named
            .named
            .iter()
            .map(|x| {
                let field = x.ident.as_ref().unwrap();
                if self.is_optional(&x.ty) {
                    quote!(#field : #field
                       .ok_or_else(|| format_err!("'{}' is empty.", stringify!(#ident::#field)))?
                       .try_into()?,)
                } else {
                    quote!(#field : #field.try_into()?,)
                }
            })
            .collect();

        let to_proxy = fields_named
            .named
            .iter()
            .map(|x| {
                let field = x.ident.as_ref().unwrap();
                if self.is_optional(&x.ty) {
                    quote!(#field : Some(#field.try_into()?),)
                } else {
                    quote!(#field : #field.try_into()?,)
                }
            })
            .collect();

        (bindings, from_proxy, to_proxy)
    }
}

pub fn generate_conversion_apis<'a, W: Write>(
    file_descriptor: &FileDescriptor,
    file: &File,
    w: &mut W,
) -> io::Result<()> {
    let mut builder = ConversionGenerator {
        file_descriptor,
        token_stream: TokenStream::default(),
    };
    extract::extract_from_file(&mut builder, file);
    write!(w, "{}", builder.token_stream)
}
