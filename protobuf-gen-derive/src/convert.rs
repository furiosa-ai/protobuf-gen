use std::collections::HashSet;

use heck::SnakeCase;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{self, Field, Fields, FieldsNamed, Ident, ItemEnum, ItemStruct, TypePath, Variant};

use extract::Extract;

pub(crate) struct ConversionGenerator {
    pub(crate) token_stream: TokenStream,
    pub(crate) proxy_mod: TypePath,
}

impl Extract for ConversionGenerator {
    fn extract_message_with_fields_named(
        &mut self,
        item_struct: &ItemStruct,
        fields_named: &FieldsNamed,
    ) {
        let ident = &item_struct.ident;
        let proxy = &self.proxy_mod;

        let (ref bindings, ref assignments) = self.generate_assignments(fields_named);

        self.token_stream.extend(quote! {
            impl ::std::convert::TryInto<Option<#proxy::#ident>> for #ident {
                type Error = ::failure::Error;

                fn try_into(self) -> ::failure::Fallible<Option<#proxy::#ident>> {
                    use std::convert::TryInto;

                    let #ident { #(#bindings)* .. } = self;
                    Ok(Some(#proxy::#ident {
                        #(#assignments)*
                    }))
                }
            }

            impl ::std::convert::TryInto<#proxy::#ident> for #ident {
                type Error = ::failure::Error;

                fn try_into(self) -> ::failure::Fallible<#proxy::#ident> {
                    use std::convert::TryInto;

                    let #ident { #(#bindings)* .. } = self;
                    Ok(#proxy::#ident {
                        #(#assignments)*
                    })
                }
            }
        });

        let ref private_fields =
            if let Fields::Named(FieldsNamed { named, .. }) = &item_struct.fields {
                let total_fields: HashSet<_> = named.iter().collect();
                let proto_fields: HashSet<_> = fields_named.named.iter().collect();
                (&total_fields - &proto_fields)
                    .into_iter()
                    .map(|f| {
                        let ident = &f.ident;
                        quote!(#ident: Default::default())
                    })
                    .collect()
            } else {
                Vec::new()
            };

        self.token_stream.extend(quote! {
            impl ::std::convert::TryFrom<Option<#proxy::#ident>> for #ident {
                type Error = ::failure::Error;

                fn try_from(other: Option<#proxy::#ident>) -> ::failure::Fallible<Self> {
                    use std::convert::TryInto;

                    let #proxy::#ident { #(#bindings)* } = other
                        .ok_or_else(|| format_err!("empty \"{}\" object", stringify!(#proxy::#ident)))?
                        .try_into()?;

                    Ok(Self {
                        #(#assignments)*
                        #(#private_fields)*
                    })
                }
            }

            impl ::std::convert::TryFrom<#proxy::#ident> for #ident {
                type Error = ::failure::Error;

                fn try_from(#proxy::#ident { #(#bindings)* }: #proxy::#ident) -> ::failure::Fallible<Self> {
                    use std::convert::TryInto;

                    Ok(Self {
                        #(#assignments)*
                        #(#private_fields)*
                    })
                }
            }
        });

        self.add_derive_protobuf_gen(ident);
    }

    fn extract_message_with_fields_unit(&mut self, item_struct: &ItemStruct) {
        let ident = &item_struct.ident;
        let proxy = &self.proxy_mod;

        self.token_stream.extend(quote! {
            impl ::std::convert::TryFrom<#ident> for #proxy::#ident {
                type Error = ::failure::Error;

                fn try_from(_: #ident) -> ::failure::Fallible<Self> {
                    Ok(Self {})
                }
            }

            impl ::std::convert::TryFrom<#proxy::#ident> for #ident {
                type Error = ::failure::Error;

                fn try_from(_: #proxy::#ident) -> ::failure::Fallible<Self> {
                    Ok(Self {})
                }
            }
        });

        self.add_derive_protobuf_gen(ident);
    }

    fn extract_nested_message_with_fields_named(
        &mut self,
        item_enum: &ItemEnum,
        variant: &Variant,
        fields_named: &FieldsNamed,
    ) {
        let ident = &item_enum.ident;
        let proxy = &self.proxy_mod;
        let inner_mod: Ident = syn::parse_str(&ident.to_string().to_snake_case()).unwrap();
        let variant = &variant.ident;
        let variant_inner: Ident = syn::parse_str(&format!("{}Inner", variant)).unwrap();

        let (bindings, assignments) = self.generate_assignments(fields_named);

        self.token_stream.extend(quote! {
            impl ::std::convert::TryFrom<#proxy::#inner_mod::#variant_inner> for #ident {
                type Error = ::failure::Error;

                fn try_from(#proxy::#inner_mod::#variant_inner { #(#bindings)* }: #proxy::#inner_mod::#variant_inner) -> ::failure::Fallible<Self> {
                    use std::convert::TryInto;

                    Ok(#ident::#variant {
                        #(#assignments)*
                    })
                }
            }
        });
    }

    fn extract_nested_message_with_field_unnamed(
        &mut self,
        item_enum: &ItemEnum,
        variant: &Variant,
        _: &Field,
    ) {
        let ident = &item_enum.ident;
        let proxy = &self.proxy_mod;
        let variant = &variant.ident;

        self.token_stream.extend(quote! {
            impl ::std::convert::TryFrom<#proxy::#variant> for #ident {
                type Error = ::failure::Error;

                fn try_from(other: #proxy::#variant) -> ::failure::Fallible<Self> {
                    use std::convert::TryInto;

                    Ok(#ident::#variant(other.try_into()?))
                }
            }
        });
    }

    fn extract_nested_message_with_fields_unit(&mut self, item_enum: &ItemEnum, variant: &Variant) {
        let ident = &item_enum.ident;
        let proxy = &self.proxy_mod;
        let inner_mod: Ident = syn::parse_str(&ident.to_string().to_snake_case()).unwrap();
        let variant = &variant.ident;
        let variant_inner: Ident = syn::parse_str(&format!("{}Inner", variant)).unwrap();

        self.token_stream.extend(quote! {
            impl ::std::convert::TryFrom<#proxy::#inner_mod::#variant_inner> for #ident {
                type Error = ::failure::Error;

                fn try_from(_: #proxy::#inner_mod::#variant_inner) -> ::failure::Fallible<Self> {
                    Ok(#ident::#variant {})
                }
            }
        });
    }

    fn extract_one_of(&mut self, item_enum: &ItemEnum) {
        let ident = &item_enum.ident;
        let proxy = &self.proxy_mod;
        let inner_mod: Ident = syn::parse_str(&ident.to_string().to_snake_case()).unwrap();

        let ref cases = item_enum.variants.iter().map(|v| {
            let variant = &v.ident;
            let variant_inner: Ident = syn::parse_str(&format!("{}Inner", variant)).unwrap();
            match &v.fields {
                Fields::Unit => quote!{
                    #ident::#variant {} => #proxy::#ident {
                        inner: Some(#proxy::#inner_mod::Inner::#variant(#proxy::#inner_mod::#variant_inner {})),
                    },
                },
                Fields::Named(fields_named) => {
                    let (bindings, assignments) = self.generate_assignments(fields_named);
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
        }).collect::<Vec<_>>();

        self.token_stream.extend(quote! {
            impl ::std::convert::TryInto<#proxy::#ident> for #ident {
                type Error = ::failure::Error;

                fn try_into(self) -> ::failure::Fallible<#proxy::#ident> {
                    use std::convert::TryInto;

                    Ok(match self {
                        #(#cases)*
                    })
                }
            }

            impl ::std::convert::TryInto<Option<#proxy::#ident>> for #ident {
                type Error = ::failure::Error;

                fn try_into(self) -> ::failure::Fallible<Option<#proxy::#ident>> {
                    use std::convert::TryInto;

                    Ok(Some(match self {
                        #(#cases)*
                    }))
                }
            }
        });

        let ref cases = item_enum
            .variants
            .iter()
            .map(|v| {
                let variant = &v.ident;
                quote!(#proxy::#inner_mod::Inner::#variant(inner) => inner.try_into(),)
            })
            .collect::<Vec<_>>();

        self.token_stream.extend(quote! {
            impl ::std::convert::TryFrom<#proxy::#ident> for #ident {
                type Error = ::failure::Error;

                fn try_from(#proxy::#ident { inner }: #proxy::#ident) -> ::failure::Fallible<Self> {
                    use std::convert::TryInto;

                    match inner.ok_or_else(|| format_err!("\"{}\" doesn't have a value.", stringify!(#ident)))? {
                        #(#cases)*
                    }
                }
            }

            impl ::std::convert::TryFrom<Option<#proxy::#ident>> for #ident {
                type Error = ::failure::Error;

                fn try_from(other: Option<#proxy::#ident>) -> ::failure::Fallible<Self> {
                    use std::convert::TryInto;

                    let #proxy::#ident { inner } = other
                        .ok_or_else(|| format_err!("empty \"{}\" object", stringify!(#proxy::#ident)))?
                        .try_into()?;
                    match inner.ok_or_else(|| format_err!("\"{}\" doesn't have a value.", stringify!(#proxy::#ident)))? {
                        #(#cases)*
                    }
                }
            }
        });

        self.add_derive_protobuf_gen(ident);
    }

    fn extract_enumerator(&mut self, item_enum: &ItemEnum) {
        let ident = &item_enum.ident;
        let proxy = &self.proxy_mod;

        let cases = item_enum.variants.iter().map(|v| {
            let variant = &v.ident;
            quote!(#ident::#variant => #proxy::#ident::#variant,)
        });

        self.token_stream.extend(quote! {
            impl ::std::convert::TryFrom<#ident> for #proxy::#ident {
                type Error = ::failure::Error;

                fn try_from(other: #ident) -> ::failure::Fallible<Self> {
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
            impl ::std::convert::TryFrom<#proxy::#ident> for #ident {
                type Error = ::failure::Error;

                fn try_from(other: #proxy::#ident) -> ::failure::Fallible<Self> {
                    Ok(match other {
                        #(#cases)*
                    })
                }
            }
        });

        self.token_stream.extend(quote! {
            impl ProtobufGen for #ident {
                fn to_protobuf<W: ::std::io::Write>(self, w: &mut W) -> ::failure::Fallible<()> {
                    use std::convert::TryInto;
                    use prost::Message;

                    let proxy: #proxy::#ident = self.try_into()?;
                    let proxy: i32 = proxy.into();
                    let mut buffer = Vec::with_capacity(proxy.encoded_len());
                    proxy.encode(&mut buffer)?;
                    w.write_all(&buffer)?;
                    Ok(())
                }

                fn from_protobuf<R: ::std::io::Read>(r: &mut R) -> ::failure::Fallible<Self> {
                    use std::convert::TryInto;

                    let mut buffer = Vec::new();
                    r.read_to_end(&mut buffer)?;
                    let proxy = #proxy::#ident::from_i32(prost::Message::decode(buffer)?)
                        .ok_or_else(|| format_err!("invalid \"{}\"", stringify!(#ident)))?;
                    proxy.try_into()
                }
            }
        });
    }
}

impl ConversionGenerator {
    fn add_derive_protobuf_gen<T>(&mut self, ident: T)
    where
        T: ToTokens,
    {
        let proxy = &self.proxy_mod;

        self.token_stream.extend(quote! {
            impl ProtobufGen for #ident {
                fn to_protobuf<W: ::std::io::Write>(self, w: &mut W) -> ::failure::Fallible<()> {
                    use std::convert::TryInto;
                    use prost::Message;

                    let proxy: #proxy::#ident = self.try_into()?;
                    let mut buffer = Vec::with_capacity(proxy.encoded_len());
                    proxy.encode(&mut buffer)?;
                    w.write_all(&buffer)?;
                    Ok(())
                }

                fn from_protobuf<R: ::std::io::Read>(r: &mut R) -> ::failure::Fallible<Self> {
                    use std::convert::TryInto;

                    let mut buffer = Vec::new();
                    r.read_to_end(&mut buffer)?;
                    let proxy: #proxy::#ident = prost::Message::decode(buffer)?;
                    proxy.try_into()
                }
            }
        });
    }

    fn generate_assignments(
        &self,
        fields_named: &FieldsNamed,
    ) -> (Vec<TokenStream>, Vec<TokenStream>) {
        let bindings = fields_named
            .named
            .iter()
            .map(|x| {
                let field = x.ident.as_ref().unwrap();
                quote!(#field,)
            })
            .collect();

        let assignments = fields_named
            .named
            .iter()
            .map(|x| {
                let field = x.ident.as_ref().unwrap();
                quote!(#field : #field.try_into()?,)
            })
            .collect();

        (bindings, assignments)
    }
}
