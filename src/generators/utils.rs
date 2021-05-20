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
    pub(crate) outerattr_or_identandtype: Vec<TokenStream>,
    pub(crate) inner_structs: Vec<TokenStream>,
}
pub(crate) fn handle_named_fields(
    struct_name: &str,
    inner_nodes: serde_json::Map<String, serde_json::Value>,
) -> TypegenResult<FieldsInfo> {
    let mut outerattr_or_identandtype: Vec<TokenStream> = Vec::new();
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
        outerattr_or_identandtype.push(quote!(#serde_rename));
        outerattr_or_identandtype.push(quote!(#token_ident: #tokenized_val,));
    }
    Ok(FieldsInfo {
        case,
        inner_structs,
        outerattr_or_identandtype,
    })
}
