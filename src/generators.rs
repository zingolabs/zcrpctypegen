use crate::{utils::callsite_ident, TypegenResult};
use proc_macro2::TokenStream;
use quote::quote;
use serde_json::{Map, Value};
mod fieldinterpreters;
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
            let variant_ident_token = callsite_ident(&variant_name);
            match value {
                Value::Object(obj) => build_structvariant(
                    enum_name,
                    obj,
                    &mut inner_structs,
                    &variant_ident_token,
                ),
                non_object => {
                    let (variant_body_tokens, new_structs, _terminal_enum) =
                        tokenize::value(&variant_name, non_object)?;
                    inner_structs.extend(new_structs);
                    Ok(quote!(#variant_ident_token(#variant_body_tokens),))
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
            let variant_ident_token = callsite_ident(&variant_name);
            match value {
                Value::Object(obj) => build_argumentenum_tuplevariant(
                    obj,
                    &mut inner_structs,
                    &variant_ident_token,
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
            let variant_ident_token = callsite_ident(&variant_name);
            match value {
                Value::Object(obj) => build_structvariant(
                    enum_name,
                    obj,
                    &mut inner_structs,
                    &variant_ident_token,
                ),
                non_object => {
                    let (variant_body_tokens, new_structs, _terminal_enum) =
                        tokenize::value(&variant_name, non_object)?;
                    inner_structs.extend(new_structs);
                    Ok(quote!(#variant_ident_token(#variant_body_tokens),))
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
    let field_data =
        fieldinterpreters::interpret_named_fields(struct_name, inner_nodes)?;
    let mut outerattr_or_identandtype = field_data.outerattr_or_identandtype;
    let body = match field_data.case {
        utils::FourXs::False => {
            utils::add_pub_keywords(&mut outerattr_or_identandtype);
            quote!(
                pub struct #ident {
                    #(#outerattr_or_identandtype)*
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
    let field_data = fieldinterpreters::handle_enumerated_fields(inner_nodes)?;
    let field_type = field_data.indexed_type;
    let body = quote!( pub struct #ident (
            #(#field_type)*
        );
    );

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

fn build_structvariant(
    enum_name: &str,
    obj: serde_json::Map<String, serde_json::Value>,
    inner_structs: &mut std::vec::Vec<TokenStream>,
    variant_ident_token: &proc_macro2::Ident,
) -> TypegenResult<TokenStream> {
    let field_data = fieldinterpreters::interpret_named_fields(enum_name, obj)?;
    inner_structs.extend(field_data.inner_structs);
    let variant_body_tokens = field_data.outerattr_or_identandtype;
    Ok(quote![
        #variant_ident_token {
            #(#variant_body_tokens)*
        },
    ])
}
fn build_argumentenum_tuplevariant(
    obj: serde_json::Map<String, serde_json::Value>,
    inner_structs: &mut std::vec::Vec<TokenStream>,
    variant_ident_token: &proc_macro2::Ident,
) -> TypegenResult<TokenStream> {
    let field_data = fieldinterpreters::handle_enumerated_fields(obj)?;
    inner_structs.extend(field_data.inner_structs);
    let variant_body_tokens = field_data.indexed_type;
    Ok(quote![
        #variant_ident_token (
            #(#variant_body_tokens)*
        ),
    ])
}

#[cfg(test)]
mod test {
    use super::*;
    mod build_argumentenum_tuplevariant {
        #[ignore]
        #[test]
        fn happy() {}
    }
    mod emptygen {
        use super::*;
        #[test]
        fn happy_path() {
            let observed_empty_struct_vec = emptygen("InsertNameHere");
            assert_eq!(1, observed_empty_struct_vec.len());
            let observed_empty_struct = &observed_empty_struct_vec[0];
            let expected_empty_struct = quote!(
                #[derive(Debug, serde::Deserialize, serde::Serialize)]
                pub struct InsertNameHere;
            );
            assert_eq!(
                expected_empty_struct.to_string(),
                observed_empty_struct.to_string()
            );
        }
        #[ignore]
        #[test]
        fn keyword_struct_name() {
            //! Keyword checking happens before emptygen is called.
        }
        #[test]
        #[should_panic(expected = "\"1nvalid\" is not a valid Ident")]
        fn invalid_ident_chars_in_struct_name() {
            //! callsite_ident currently panics on invalid Idents.
            //! Maybe we should change this.
            let _observed_empty_struct_vec = emptygen("1nvalid");
        }
    }
}
