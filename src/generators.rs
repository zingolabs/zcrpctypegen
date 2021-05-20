use crate::{utils::callsite_ident, TypegenResult};
use proc_macro2::TokenStream;
use quote::quote;
use serde_json::{Map, Value};
mod tokenize;
mod utils;

const RESPONSE_VARIANTS: &[&str] = &["Regular", "Verbose", "VeryVerbose"];

pub(crate) fn response_enumgen(
    inner_nodes: Vec<Value>,
    enum_name: &str,
) -> TypegenResult<Vec<TokenStream>> {
    assert!(inner_nodes.len() <= RESPONSE_VARIANTS.len());
    let mut inner_structs = Vec::new();
    let ident = callsite_ident(enum_name);
    let enum_code: Vec<TokenStream> = inner_nodes
        .into_iter()
        .zip(RESPONSE_VARIANTS.iter())
        .map(|(value, variant_name)| {
            let variant_name_tokens = callsite_ident(&variant_name);
            match value {
                Value::Object(obj) => struct_variant(
                    enum_name,
                    obj,
                    &mut inner_structs,
                    &variant_name_tokens,
                ),
                non_object => {
                    let (variant_body_tokens, new_structs, _terminal_enum) =
                        tokenize::value(&variant_name, non_object)?;
                    inner_structs.extend(new_structs);
                    Ok(quote!(#variant_name_tokens(#variant_body_tokens),))
                }
            }
        })
        .collect::<TypegenResult<Vec<TokenStream>>>()?;
    inner_structs.push(quote!(
            #[derive(Debug, serde::Deserialize, serde::Serialize)]
            pub enum #ident {
                #(#enum_code)*
            }
    ));
    Ok(inner_structs)
}
pub(crate) fn arguments_enumgen(
    inner_nodes: Vec<Value>,
    enum_name: &str,
) -> TypegenResult<Vec<TokenStream>> {
    const ARGUMENT_VARIANTS: &[&str] = &["MultiAddress", "Address"];
    let mut inner_structs = Vec::new();
    let ident = callsite_ident(enum_name);
    let enum_code: Vec<TokenStream> = inner_nodes
        .into_iter()
        .zip(ARGUMENT_VARIANTS.iter())
        .map(|(value, variant_name)| {
            let variant_name_tokens = callsite_ident(&variant_name);
            match value {
                Value::Object(obj) => tuple_variant(
                    enum_name,
                    obj,
                    &mut inner_structs,
                    &variant_name_tokens,
                ),
                non_object => panic!(
                    "Fould {} in args",
                    serde_json::to_string_pretty(&non_object).unwrap()
                ),
            }
        })
        .collect::<TypegenResult<Vec<TokenStream>>>()?;
    inner_structs.push(quote!(
            #[derive(Debug, serde::Deserialize, serde::Serialize)]
            pub enum #ident {
                #(#enum_code)*
            }
    ));
    Ok(inner_structs)
}
pub(crate) fn inner_enumgen(
    inner_nodes: Vec<(Value, &str)>,
    enum_name: &str,
) -> TypegenResult<Vec<TokenStream>> {
    let mut inner_structs = Vec::new();
    let ident = callsite_ident(enum_name);
    let enum_code: Vec<TokenStream> = inner_nodes
        .into_iter()
        .map(|(value, variant_name)| {
            let variant_name_tokens = callsite_ident(&variant_name);
            match value {
                Value::Object(obj) => struct_variant(
                    enum_name,
                    obj,
                    &mut inner_structs,
                    &variant_name_tokens,
                ),
                non_object => {
                    let (variant_body_tokens, new_structs, _terminal_enum) =
                        tokenize::value(&variant_name, non_object)?;
                    inner_structs.extend(new_structs);
                    Ok(quote!(#variant_name_tokens(#variant_body_tokens),))
                }
            }
        })
        .collect::<TypegenResult<Vec<TokenStream>>>()?;
    inner_structs.push(quote!(
            #[derive(Debug, serde::Deserialize, serde::Serialize)]
            pub enum #ident {
                #(#enum_code)*
            }
    ));
    Ok(inner_structs)
}

pub(crate) fn namedfield_structgen(
    inner_nodes: Map<String, Value>,
    struct_name: &str,
) -> TypegenResult<(utils::FourXs, Vec<TokenStream>)> {
    let ident = callsite_ident(struct_name);
    let field_data = utils::handle_named_fields(struct_name, inner_nodes)?;
    let mut ident_val_tokens = field_data.ident_val_tokens;
    let body = match field_data.case {
        utils::FourXs::False => {
            utils::add_pub_keywords(&mut ident_val_tokens);
            quote!(
                pub struct #ident {
                    #(#ident_val_tokens)*
                }
            )
        }
        utils::FourXs::True => {
            return Ok((utils::FourXs::True, field_data.inner_structs));
        }
    };

    let mut generated_code = vec![quote!(
        #[derive(Debug, serde::Deserialize, serde::Serialize)]
        #body
    )];
    generated_code.extend(field_data.inner_structs);
    Ok((utils::FourXs::False, generated_code))
}

pub(crate) fn emptygen(struct_name: &str) -> Vec<TokenStream> {
    let ident = callsite_ident(struct_name);
    vec![quote!(
        #[derive(Debug, serde::Deserialize, serde::Serialize)]
        pub struct #ident;
    )]
}

pub(crate) fn argumentgen(
    inner_nodes: Map<String, Value>,
    struct_name: &str,
) -> TypegenResult<(utils::FourXs, Vec<TokenStream>)> {
    let ident = callsite_ident(struct_name);
    let field_data = utils::handle_named_fields(struct_name, inner_nodes)?;
    let mut ident_val_tokens = field_data.ident_val_tokens;
    let body = match field_data.case {
        utils::FourXs::False => {
            utils::add_pub_keywords(&mut ident_val_tokens);
            quote!(
                pub struct #ident (
                    #(#ident_val_tokens)*
                )
            )
        }
        utils::FourXs::True => {
            return Ok((utils::FourXs::True, field_data.inner_structs));
        }
    };

    let mut generated_code = vec![quote!(
        #[derive(Debug, serde::Deserialize, serde::Serialize)]
        #body
    )];
    generated_code.extend(field_data.inner_structs);
    Ok((utils::FourXs::False, generated_code))
}

pub(crate) fn alias(
    data: Value,
    name: &str,
) -> TypegenResult<Vec<TokenStream>> {
    let ident = callsite_ident(&name);
    let (type_body, mut inner_structs, terminal_enum) = tokenize::value(
        &[&name.trim_end_matches("Response"), "Element"].concat(),
        data,
    )?;
    if !terminal_enum {
        let aliased = quote!(
            pub type #ident = #type_body;
        );
        inner_structs.push(aliased);
    }
    Ok(inner_structs)
}

fn struct_variant(
    enum_name: &str,
    obj: serde_json::Map<String, serde_json::Value>,
    inner_structs: &mut std::vec::Vec<TokenStream>,
    variant_name_tokens: &proc_macro2::Ident,
) -> TypegenResult<TokenStream> {
    let field_data = utils::handle_named_fields(enum_name, obj)?;
    inner_structs.extend(field_data.inner_structs);
    let variant_body_tokens = field_data.ident_val_tokens;
    Ok(quote!(
                            #variant_name_tokens {
                                #(#variant_body_tokens)*
                            },))
}
fn tuple_variant(
    enum_name: &str,
    obj: serde_json::Map<String, serde_json::Value>,
    inner_structs: &mut std::vec::Vec<TokenStream>,
    variant_name_tokens: &proc_macro2::Ident,
) -> TypegenResult<TokenStream> {
    let field_data = utils::handle_named_fields(enum_name, obj)?;
    inner_structs.extend(field_data.inner_structs);
    let variant_body_tokens = field_data.ident_val_tokens;
    Ok(quote!(
                            #variant_name_tokens {
                                #(#variant_body_tokens)*
                            },))
}
