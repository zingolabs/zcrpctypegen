use crate::utils::{callsite_ident, camel_to_under, under_to_camel};
use crate::TypegenResult;
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) enum FourXs {
    False,
    True,
}

pub(crate) fn handle_options_and_keywords(
    serde_rename: &mut Option<TokenStream>,
    field_name: &mut String,
    option: &mut bool,
) -> () {
    if crate::utils::RESERVED_KEYWORDS.contains(&field_name.as_str()) {
        *serde_rename = Some(
            format!("#[serde(rename = \"{}\")]", &field_name)
                .parse()
                .unwrap(),
        );
        field_name.push_str("_field");
    }

    if field_name.starts_with("Option<") {
        *field_name = field_name
            .trim_end_matches(">")
            .trim_start_matches("Option<")
            .to_string();
        *option = true;
    }
}

fn handle_argument_field_name(field_name: String) -> String {
    field_name
        .chars()
        .map(|a_char| {
            match a_char.to_string().as_str() {
                "-" | "_" => "_",
                "<" => "<",
                ">" => ">",
                "|" => "_or_",
                "1" => "one",
                "2" => "two",
                "3" => "three",
                "4" => "four",
                "5" => "five",
                "6" => "six",
                c if c.chars().next().unwrap().is_alphabetic() => c,
                c => {
                    eprintln!(
                        "WARNING: omitting bad char '{}' in field name '{}'",
                        c, &field_name
                    );
                    ""
                }
            }
            .to_string()
        })
        .collect()
}

pub(crate) fn handle_argument_fields_names(
    nodes: serde_json::Map<String, serde_json::Value>,
) -> serde_json::Map<String, serde_json::Value> {
    nodes
        .into_iter()
        .map(|(field_name, val)| {
            let new_field_name = if field_name.starts_with("Option<") {
                format!(
                    "Option<{}>",
                    handle_argument_field_name(
                        field_name
                            .strip_prefix("Option<")
                            .unwrap()
                            .strip_suffix(">")
                            .unwrap()
                            .to_string()
                    )
                )
            } else {
                handle_argument_field_name(field_name)
            };

            (new_field_name, val)
        })
        .collect()
}

pub(crate) fn add_pub_keywords(tokens: &mut Vec<TokenStream>) {
    *tokens = tokens
        .into_iter()
        .map(|ts| match ts.clone().into_iter().next() {
            None | Some(proc_macro2::TokenTree::Punct(_)) => ts.clone(),
            _ => quote!(pub #ts),
        })
        .collect();
}

pub(crate) struct FieldsInfo {
    pub(crate) case: super::utils::FourXs,
    pub(crate) ident_val_tokens: Vec<TokenStream>,
    pub(crate) inner_structs: Vec<TokenStream>,
}
pub(crate) fn handle_fields(
    struct_name: &str,
    inner_nodes: serde_json::Map<String, serde_json::Value>,
) -> TypegenResult<FieldsInfo> {
    let mut ident_val_tokens: Vec<TokenStream> = Vec::new();
    let mut inner_structs = Vec::new();
    let mut case = super::utils::FourXs::False;
    for (mut field_name, val) in inner_nodes {
        //special case handling
        if &field_name == "xxxx" {
            inner_structs = super::tokenize::value(struct_name, val)?.1; // .0 unused
            case = super::utils::FourXs::True;
            break;
        }

        let mut serde_rename = None;
        let mut option = false;
        handle_options_and_keywords(
            &mut serde_rename,
            &mut field_name,
            &mut option,
        );
        field_name = camel_to_under(&field_name);

        //temp_acc needed because destructuring assignments are unstable
        //see https://github.com/rust-lang/rust/issues/71126 for more info
        let (mut tokenized_val, new_struct, _terminal_enum) =
            super::tokenize::value(&under_to_camel(&field_name), val)?;
        inner_structs.extend(new_struct);
        if option {
            use std::str::FromStr as _;
            tokenized_val =
                TokenStream::from_str(&format!("Option<{}>", tokenized_val))
                    .unwrap();
        }

        let token_ident = callsite_ident(&field_name);
        ident_val_tokens.push(quote!(#serde_rename));
        ident_val_tokens.push(quote!(#token_ident: #tokenized_val,));
    }
    Ok(FieldsInfo {
        case,
        inner_structs,
        ident_val_tokens,
    })
}
