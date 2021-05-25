use crate::{error, generators, utils::callsite_ident};
use error::TypegenResult;
use proc_macro2::TokenStream;
use quote::quote;

fn handle_terminal_enum(label: &str, name: &str) -> TokenStream {
    let variants = label
        .strip_prefix("ENUM:")
        .unwrap()
        .split(',')
        .map(|x| x.trim());
    let variant_idents = variants
        .clone()
        .map(|x| {
            proc_macro2::TokenTree::Ident(callsite_ident(
                &x.split('-')
                    .map(crate::utils::under_to_camel)
                    .collect::<String>(),
            ))
            .into()
        })
        .collect::<Vec<TokenStream>>();
    let variant_idents_renames = variants
        .map(|x| format!("#[serde(rename = \"{}\")]", x).parse().unwrap())
        .collect::<Vec<TokenStream>>();
    let name_tokens = callsite_ident(&format!("{}Response", name));
    quote!(
        #[derive(Debug, serde::Deserialize, serde::Serialize)]
        pub enum #name_tokens {
            #(#variant_idents_renames #variant_idents,)*
        }
    )
}

pub(crate) fn value(
    name: &str,
    val: serde_json::Value,
) -> TypegenResult<(TokenStream, Vec<TokenStream>, bool)> {
    match val {
        serde_json::Value::String(label) => terminal(name, label.as_str()),
        serde_json::Value::Array(vec) => {
            array(name, vec).map(|x| (x.0, x.1, false))
        }
        serde_json::Value::Object(obj) => {
            object(name, obj).map(|x| (x.0, x.1, false))
        }
        otherwise => Err(error::QuizfaceAnnotationError {
            kind: error::InvalidAnnotationKind::from(otherwise),
            location: name.to_string(),
        })?,
    }
}

fn terminal(
    name: &str,
    label: &str,
) -> TypegenResult<(TokenStream, Vec<TokenStream>, bool)> {
    Ok((
        match label {
            "Decimal" => quote!(rust_decimal::Decimal),
            "bool" => quote!(bool),
            "String" => quote!(String),
            "hexadecimal" => quote!(String),
            "INSUFFICIENT" => {
                return Err(error::TypegenError::from(
                    error::QuizfaceAnnotationError {
                        kind: error::InvalidAnnotationKind::Insufficient,
                        location: name.to_string(),
                    },
                ))
            }
            enumeration if enumeration.starts_with("ENUM:") => {
                let ident = callsite_ident(name);
                let enum_tokens = handle_terminal_enum(enumeration, name);
                return Ok((quote!(#ident), vec![enum_tokens], true));
            }
            otherwise => {
                return Err(error::QuizfaceAnnotationError {
                    kind: error::InvalidAnnotationKind::from(
                        serde_json::Value::String(otherwise.to_string()),
                    ),
                    location: name.to_string(),
                }
                .into())
            }
        },
        Vec::new(),
        false,
    ))
}

const Z_GETOPERATION_VARIANTS: &[&str] = &["Executing", "Success", "Failed"];
fn array(
    name: &str,
    mut array_of: Vec<serde_json::Value>,
) -> TypegenResult<(TokenStream, Vec<TokenStream>)> {
    match array_of.len() {
        0 => Err(error::QuizfaceAnnotationError {
            kind: error::InvalidAnnotationKind::EmptyArray,
            location: name.to_string(),
        })?,
        1 => {
            let (val, inner_structs, _terminal_enum) =
                value(name, array_of.pop().unwrap())?;
            Ok((quote!(Vec<#val>), inner_structs))
        }
        _ => {
            let ident = callsite_ident(name);
            generators::inner_enumgen(
                array_of
                    .into_iter()
                    .zip(Z_GETOPERATION_VARIANTS)
                    .map(|(x, y)| (x, *y))
                    .collect(),
                name,
            )
            .map(|x| (quote!(#ident), x))
        }
    }
}

fn object(
    name: &str,
    val: serde_json::Map<String, serde_json::Value>,
) -> TypegenResult<(TokenStream, Vec<TokenStream>)> {
    let ident = callsite_ident(name);
    let (case, inner_structs) = generators::namedfield_structgen(val, name)?;
    if case {
        Ok((
            quote!(std::collections::HashMap<String, #ident>),
            inner_structs,
        ))
    } else {
        Ok((quote!(#ident), inner_structs))
    }
}

#[cfg(test)]
mod unit {
    use super::*;
    #[test]
    fn tokenize_value_string() {
        let quoted_string = value("some_field", serde_json::json!("String"));
        assert_eq!(
            quote!(String).to_string(),
            quoted_string.unwrap().0.to_string(),
        );
    }
    #[test]
    fn tokenize_value_number() {
        let quoted_number = value("some_field", serde_json::json!("Decimal"));
        assert_eq!(
            quote!(rust_decimal::Decimal).to_string(),
            quoted_number.unwrap().0.to_string(),
        );
    }
    #[test]
    fn tokenize_value_bool() {
        let quoted_bool = value("some_field", serde_json::json!("bool"));
        assert_eq!(
            quote!(bool).to_string(),
            quoted_bool.unwrap().0.to_string(),
        );
    }
    #[test]
    fn tokenize_object_simple_unnested() {
        let quoted_object = value(
            "somefield",
            serde_json::json!(
                {
                    "inner_a": "String",
                    "inner_b": "bool",
                    "inner_c": "Decimal",
                }
            ),
        )
        .unwrap();
        assert_eq!(quote!(somefield).to_string(), quoted_object.0.to_string(),);
        assert_eq!(
            quoted_object.1[0].to_string(),
            test_consts::SIMPLE_UNNESTED_RESPONSE,
        );
    }
    #[test]
    fn test_invalid_annotation_error() {
        let expected_err = crate::error::QuizfaceAnnotationError::from((
            crate::error::InvalidAnnotationKind::Null,
            "foo".to_string(),
        ));
        let err = value("foo", serde_json::Value::Null).unwrap_err();
        assert_eq!(crate::error::TypegenError::Annotation(expected_err), err);
    }
    mod test_consts {
        pub(super) const SIMPLE_UNNESTED_RESPONSE: &str = "# [derive (Debug , \
    serde :: Deserialize , serde :: Serialize)] pub struct somefield { pub \
    inner_a : String , pub inner_b : bool , pub inner_c : rust_decimal :: \
    Decimal , }";
    }
}
