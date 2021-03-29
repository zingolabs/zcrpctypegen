use crate::error;
use crate::special_cases;
use error::TypegenResult;
use proc_macro2::TokenStream;
use quote::quote;

fn handle_terminal_enum(
    label: &str,
    name: &str,
    called_by_alias: bool,
) -> TokenStream {
    let variants = label
        .strip_prefix("ENUM:")
        .unwrap()
        .split(',')
        .map(|x| x.trim());
    let variant_idents = variants
        .clone()
        .map(|x| {
            proc_macro2::TokenTree::Ident(crate::callsite_ident(
                &x.split('-')
                    .map(crate::capitalize_first_char)
                    .collect::<String>(),
            ))
            .into()
        })
        .collect::<Vec<TokenStream>>();
    let variant_idents_renames = variants
        .map(|x| format!("#[serde(rename = \"{}\")]", x).parse().unwrap())
        .collect::<Vec<TokenStream>>();
    let name_tokens = crate::callsite_ident(
        &(if called_by_alias {
            format!("{}Response", name)
        } else {
            name.to_string()
        }),
    );
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
    acc: Vec<TokenStream>,
    called_by_alias: bool,
) -> TypegenResult<(TokenStream, Vec<TokenStream>, bool)> {
    match val {
        serde_json::Value::String(label) => {
            terminal(name, label.as_str(), acc, called_by_alias)
        }
        serde_json::Value::Array(vec) => {
            array(name, vec, acc).map(|x| (x.0, x.1, false))
        }
        serde_json::Value::Object(obj) => {
            object(name, obj, acc).map(|x| (x.0, x.1, false))
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
    mut acc: Vec<TokenStream>,
    called_by_alias: bool,
) -> TypegenResult<(TokenStream, Vec<TokenStream>, bool)> {
    Ok((
        match label {
            "Decimal" => quote!(rust_decimal::Decimal),
            "bool" => quote!(bool),
            "String" => quote!(String),
            "hexadecimal" => quote!(String),
            "INSUFFICIENT" => quote!(compile_error!(
                "Insufficient zcash-cli help output to autogenerate type"
            )),
            enumeration if enumeration.starts_with("ENUM:") => {
                let ident = crate::callsite_ident(name);
                acc.push(handle_terminal_enum(
                    enumeration,
                    name,
                    called_by_alias,
                ));
                return Ok((quote!(#ident), acc, true));
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
        acc,
        false,
    ))
}

fn array(
    name: &str,
    mut array_of: Vec<serde_json::Value>,
    acc: Vec<TokenStream>,
) -> TypegenResult<(TokenStream, Vec<TokenStream>)> {
    let (val, acc, _terminal_enum) = value(
        name,
        array_of.pop().ok_or(error::QuizfaceAnnotationError {
            kind: error::InvalidAnnotationKind::EmptyArray,
            location: name.to_string(),
        })?,
        acc,
        false,
    )?;
    Ok((quote!(Vec<#val>), acc))
}

fn object(
    name: &str,
    val: serde_json::Map<String, serde_json::Value>,
    acc: Vec<TokenStream>,
) -> TypegenResult<(TokenStream, Vec<TokenStream>)> {
    let ident = crate::callsite_ident(name);
    let (case, acc) = crate::structgen(val, name, acc)?;
    match case {
        special_cases::Case::Regular => Ok((quote!(#ident), acc)),
        special_cases::Case::FourXs => {
            Ok((quote!(std::collections::HashMap<String, #ident>), acc))
        }
        otherwise => {
            panic!("structgen should not return variant {:?}", otherwise)
        }
    }
}
