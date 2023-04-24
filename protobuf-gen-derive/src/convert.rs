use std::collections::HashSet;

use heck::SnakeCase;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::{self, Fields, FieldsNamed, Ident, ItemEnum, ItemStruct, Type, TypePath, Variant};

use crate::extract::Extract;

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

        let (ref bindings, ref assignments) = self.generate_assignments(fields_named, true);

        self.token_stream.extend(quote! {
            impl ::std::convert::TryInto<Option<#proxy::#ident>> for #ident {
                type Error = protobuf_gen::Error;

                fn try_into(self) -> ::std::result::Result<Option<#proxy::#ident>, Self::Error> {
                    Ok(Some(self.try_into()?))
                }
            }

            impl ::std::convert::TryInto<#proxy::#ident> for #ident {
                type Error = protobuf_gen::Error;

                fn try_into(self) -> ::std::result::Result<#proxy::#ident, Self::Error> {
                    use std::convert::TryInto;

                    let #ident { #(#bindings)* .. } = self;
                    Ok(#proxy::#ident {
                        #(#assignments)*
                    })
                }
            }
        });

        let (ref bindings, ref assignments) = self.generate_assignments(fields_named, false);

        let private_fields = if let Fields::Named(FieldsNamed { named, .. }) = &item_struct.fields {
            let total_fields: HashSet<_> = named.iter().collect();
            let proto_fields: HashSet<_> = fields_named.named.iter().collect();
            (&total_fields - &proto_fields)
                .into_iter()
                .map(|f| {
                    let ident = &f.ident;
                    quote!(#ident: Default::default(),)
                })
                .collect()
        } else {
            Vec::new()
        };
        let private_fields = &private_fields;

        self.token_stream.extend(quote! {
            impl ::std::convert::TryFrom<Option<#proxy::#ident>> for #ident {
                type Error = protobuf_gen::Error;

                fn try_from(other: Option<#proxy::#ident>) -> ::std::result::Result<Self, Self::Error> {
                    use std::convert::TryInto;

                    if let Some(inner) = other {
                        inner.try_into()
                    }
                    else {
                        Ok(Self::default())
                    }
                }
            }

            impl ::std::convert::TryFrom<#proxy::#ident> for #ident {
                type Error = protobuf_gen::Error;

                fn try_from(#proxy::#ident { #(#bindings)* }: #proxy::#ident) -> ::std::result::Result<Self, Self::Error> {
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

        let (bindings, assignments) = self.generate_assignments(fields_named, false);

        self.token_stream.extend(quote! {
            impl ::std::convert::TryFrom<#proxy::#inner_mod::#variant_inner> for #ident {
                type Error = protobuf_gen::Error;

                fn try_from(#proxy::#inner_mod::#variant_inner { #(#bindings)* }: #proxy::#inner_mod::#variant_inner) -> ::std::result::Result<Self, Self::Error> {
                    use std::convert::TryInto;

                    Ok(#ident::#variant {
                        #(#assignments)*
                    })
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
                type Error = protobuf_gen::Error;

                fn try_from(_: #proxy::#inner_mod::#variant_inner) -> ::std::result::Result<Self, Self::Error> {
                    Ok(#ident::#variant {})
                }
            }
        });
    }

    fn extract_one_of(&mut self, item_enum: &ItemEnum) {
        let ident = &item_enum.ident;
        let proxy = &self.proxy_mod;
        let inner_mod: Ident = syn::parse_str(&ident.to_string().to_snake_case()).unwrap();

        let cases = item_enum.variants.iter().map(|v| {
            let variant = &v.ident;
            let variant_inner: Ident = syn::parse_str(&format!("{}Inner", variant)).unwrap();
            match &v.fields {
                Fields::Unit => quote!{
                    #ident::#variant {} => #proxy::#ident {
                        inner: Some(#proxy::#inner_mod::Inner::#variant(#proxy::#inner_mod::#variant_inner {})),
                    },
                },
                Fields::Named(fields_named) => {
                    let (bindings, assignments) = self.generate_assignments(fields_named, true);
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
                        inner: Some(#proxy::#inner_mod::Inner::#variant(
                            inner.try_into().map_err(|e| protobuf_gen::Error::new_try_from_error(stringify!(#proxy::#ident), e))?
                        )),
                    },
                },
            }
        }).collect::<Vec<_>>();
        let cases = &cases;

        self.token_stream.extend(quote! {
            impl ::std::convert::TryInto<#proxy::#ident> for #ident {
                type Error = protobuf_gen::Error;

                fn try_into(self) -> ::std::result::Result<#proxy::#ident, Self::Error> {
                    use std::convert::TryInto;

                    Ok(match self {
                        #(#cases)*
                    })
                }
            }

            impl ::std::convert::TryInto<Option<#proxy::#ident>> for #ident {
                type Error = protobuf_gen::Error;

                fn try_into(self) -> ::std::result::Result<Option<#proxy::#ident>, Self::Error> {
                    use std::convert::TryInto;

                    Ok(Some(self.try_into()?))
                }
            }
        });

        let cases = item_enum
            .variants
            .iter()
            .map(|v| {
                let variant = &v.ident;
                quote!(#proxy::#inner_mod::Inner::#variant(inner) =>
                    inner.try_into().map_err(|e| protobuf_gen::Error::new_try_from_error(stringify!(#proxy::#ident), e)),
                )
            })
            .collect::<Vec<_>>();
        let cases = &cases;

        self.token_stream.extend(quote! {
            impl ::std::convert::TryFrom<#proxy::#ident> for #ident {
                type Error = protobuf_gen::Error;

                fn try_from(#proxy::#ident { inner }: #proxy::#ident) -> ::std::result::Result<Self, Self::Error> {
                    use std::convert::TryInto;

                    if let Some(inner) = inner {
                        match inner {
                            #(#cases)*
                        }
                    }
                    else {
                        Ok(Default::default())
                    }
                }
            }

            impl ::std::convert::TryFrom<Option<#proxy::#ident>> for #ident {
                type Error = protobuf_gen::Error;

                fn try_from(other: Option<#proxy::#ident>) -> ::std::result::Result<Self, Self::Error> {
                    use std::convert::TryInto;

                    if let Some(inner) = other {
                        let #proxy::#ident { inner } = inner
                            .try_into().map_err(|e| protobuf_gen::Error::new_try_from_error(stringify!(#proxy::#ident), e))?;
                        match inner.ok_or_else(|| protobuf_gen::Error::new_empty_object(stringify!(#proxy::#ident)))? {
                            #(#cases)*
                        }
                    }
                    else {
                        Ok(Default::default())
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
            impl ::std::convert::TryInto<#proxy::#ident> for #ident {
                type Error = protobuf_gen::Error;

                fn try_into(self) -> ::std::result::Result<#proxy::#ident, Self::Error> {
                    Ok(match self {
                        #(#cases)*
                    })
                }
            }

            impl ::std::convert::TryInto<i32> for #ident {
                type Error = protobuf_gen::Error;

                fn try_into(self) -> ::std::result::Result<i32, Self::Error> {
                    let proxy: #proxy::#ident = self.try_into().map_err(|e| {
                        protobuf_gen::Error::new_try_from_error(stringify!(#proxy::#ident), e)
                    })?;

                    Ok(proxy.into())
                }
            }
        });

        let cases = item_enum.variants.iter().map(|v| {
            let variant = &v.ident;
            quote!(#proxy::#ident::#variant => #ident::#variant,)
        });

        self.token_stream.extend(quote! {
            impl ::std::convert::TryFrom<#proxy::#ident> for #ident {
                type Error = protobuf_gen::Error;

                fn try_from(other: #proxy::#ident) -> ::std::result::Result<Self, Self::Error> {
                    Ok(match other {
                        #(#cases)*
                    })
                }
            }

            impl ::std::convert::TryFrom<i32> for #ident {
                type Error = protobuf_gen::Error;

                fn try_from(n: i32) -> ::std::result::Result<Self, Self::Error> {
                    let proxy = #proxy::#ident::from_i32(n)
                        .ok_or_else(|| protobuf_gen::Error::new_invalid_ident(stringify!(#proxy::#ident)))?;
                    proxy.try_into().map_err(|e| {
                        protobuf_gen::Error::new_try_from_error(stringify!(#proxy::#ident), e)
                    })
                }
            }
        });

        self.token_stream.extend(quote! {
            impl ProtobufGen for #ident {
                type Error = protobuf_gen::Error;

                fn to_protobuf<B: protobuf_gen::bytes::BufMut>(self, buffer: &mut B) -> ::std::result::Result<(), Self::Error> {
                    use std::convert::TryInto;
                    use prost::Message;

                    let proxy: #proxy::#ident = self.try_into().map_err(|e| {
                        protobuf_gen::Error::new_try_from_error(stringify!(#proxy::#ident), e)
                    })?;
                    let proxy: i32 = proxy.into();
                    proxy.encode(buffer)?;
                    Ok(())
                }

                fn from_protobuf<B: protobuf_gen::bytes::Buf>(buffer: B) -> ::std::result::Result<Self, Self::Error> {
                    use std::convert::TryInto;

                    let proxy = #proxy::#ident::from_i32(prost::Message::decode(buffer)?)
                        .ok_or_else(|| protobuf_gen::Error::new_invalid_ident(stringify!(#ident).to_string()))?;
                    proxy.try_into().map_err(|e| {
                        protobuf_gen::Error::new_try_from_error(stringify!(#proxy::#ident), e)
                    })
                }

                fn to_protobuf_length_delimited<B: protobuf_gen::bytes::BufMut>(self, buffer: &mut B) -> ::std::result::Result<(), Self::Error> {
                    use std::convert::TryInto;
                    use prost::Message;

                    let proxy: #proxy::#ident = self.try_into().map_err(|e| {
                        protobuf_gen::Error::new_try_from_error(stringify!(#proxy::#ident), e)
                    })?;
                    let proxy: i32 = proxy.into();
                    proxy.encode_length_delimited(buffer)?;
                    Ok(())
                }

                fn from_protobuf_length_delimited<B: protobuf_gen::bytes::Buf>(buffer: B) -> ::std::result::Result<Self, Self::Error> {
                    use std::convert::TryInto;

                    let proxy = #proxy::#ident::from_i32(prost::Message::decode_length_delimited(buffer)?)
                        .ok_or_else(|| protobuf_gen::Error::new_invalid_ident(stringify!(#ident).to_string()))?;
                    proxy.try_into().map_err(|e| {
                        protobuf_gen::Error::new_try_from_error(stringify!(#proxy::#ident), e)
                    })
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
                type Error = protobuf_gen::Error;

                fn to_protobuf<B: protobuf_gen::bytes::BufMut>(self, buffer: &mut B) -> ::std::result::Result<(), Self::Error> {
                    use std::convert::TryInto;
                    use prost::Message;

                    let proxy: #proxy::#ident = self.try_into().map_err(|e| {
                        protobuf_gen::Error::new_try_from_error(stringify!(#proxy::#ident), e)
                    })?;

                    proxy.encode(buffer)?;
                    Ok(())
                }

                fn from_protobuf<B: protobuf_gen::bytes::Buf>(buffer: B) -> ::std::result::Result<Self, Self::Error> {
                    use std::convert::TryInto;

                    let proxy: #proxy::#ident = prost::Message::decode(buffer)?;
                    proxy.try_into().map_err(|e| {
                        protobuf_gen::Error::new_try_from_error(stringify!(#proxy::#ident), e)
                    })
                }

                fn to_protobuf_length_delimited<B: protobuf_gen::bytes::BufMut>(self, buffer: &mut B) -> ::std::result::Result<(), Self::Error> {
                    use std::convert::TryInto;
                    use prost::Message;

                    let proxy: #proxy::#ident = self.try_into().map_err(|e| {
                        protobuf_gen::Error::new_try_from_error(stringify!(#proxy::#ident), e)
                    })?;

                    proxy.encode_length_delimited(buffer)?;
                    Ok(())
                }

                fn from_protobuf_length_delimited<B: protobuf_gen::bytes::Buf>(buffer: B) -> ::std::result::Result<Self, Self::Error> {
                    use std::convert::TryInto;

                    let proxy: #proxy::#ident = prost::Message::decode_length_delimited(buffer)?;
                    proxy.try_into().map_err(|e| {
                        protobuf_gen::Error::new_try_from_error(stringify!(#proxy::#ident), e)
                    })
                }
            }
        });
    }

    fn generate_assignments(
        &self,
        fields_named: &FieldsNamed,
        into_proxy: bool,
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
                if syn_util::contains_attribute(&x.attrs, &["protobuf_gen", "opaque"]) {
                    return if into_proxy {
                        quote!(
                            #field : {
                                let mut buffer = Vec::new();
                                #field.to_protobuf(&mut buffer)?;
                                buffer
                            },
                        )
                    }
                    else {
                        quote!(
                            #field : ProtobufGen::from_protobuf(&mut std::io::Cursor::new(#field))?,
                        )
                    };
                }

                if let Type::Path(type_path) = &x.ty {
                    let type_ident = &type_path.path.segments.last().unwrap().ident;
                    if type_ident == "Vec"
                        || type_ident == "HashSet"
                        || type_ident == "IndexMap"
                        || type_ident == "IndexSet"
                    {
                        return quote!(
                            #field : #field.into_iter().map(|x|
                                x.try_into().map_err(|e|
                                    protobuf_gen::Error::new_try_from_error(stringify!(#field).to_string(), e)
                                )
                            ).collect::<::std::result::Result<_, protobuf_gen::Error>>()?,
                        );
                    } else if type_ident == "HashMap" {
                        return quote!(
                            #field : #field.into_iter().map(|(k, v)| {
                                let k = k.try_into().map_err(|e|
                                    protobuf_gen::Error::new_try_from_error(stringify!(#field).to_string(), e)
                                )?;

                                let v = v.try_into().map_err(|e|
                                    protobuf_gen::Error::new_try_from_error(stringify!(#field).to_string(), e)
                                )?;

                                Ok((k, v))
                            }).collect::<::std::result::Result<_, protobuf_gen::Error>>()?,
                        );
                    }
                    else if type_ident == "Option" {
                        return quote!(
                            #field : #field.map(|v| {
                                v.try_into().map_err(|e|
                                    protobuf_gen::Error::new_try_from_error(stringify!(#field).to_string(), e)
                                )
                            }).transpose()?,
                        );
                    }
                }
                quote!(
                    #field : #field.try_into().map_err(|e| protobuf_gen::Error::new_try_from_error(stringify!(#field).to_string(), e))?,
                )
            })
            .collect();

        (bindings, assignments)
    }
}
