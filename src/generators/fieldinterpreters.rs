use crate::generators::utils::handle_options_and_keywords;
use crate::utils::{callsite_ident, camel_to_under, under_to_camel};
use crate::TypegenResult;
use proc_macro2::TokenStream;
use quote::quote;
use serde_json::{Map, Value};
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

        let (mut field_type, new_struct, _terminal_enum) =
            super::tokenize::value(&under_to_camel(&field_name), val)?;
        inner_structs.extend(new_struct);
        if option {
            use std::str::FromStr as _;
            field_type =
                TokenStream::from_str(&format!("Option<{}>", field_type))
                    .unwrap();
        }

        let token_ident = callsite_ident(&field_name);
        outerattr_or_identandtype.push(quote!(#serde_rename));
        outerattr_or_identandtype.push(quote!(#token_ident: #field_type,));
    }
    Ok(FieldsInfo {
        case,
        inner_structs,
        outerattr_or_identandtype,
    })
}
//pub(crate) struct ArgumentTuple
pub(crate) fn handle_enumerated_fields(
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

        let (mut field_type, new_struct, _terminal_enum) =
            super::tokenize::value(&under_to_camel(&field_name), val)?;
        inner_structs.extend(new_struct);
        if option {
            use std::str::FromStr as _;
            field_type =
                TokenStream::from_str(&format!("Option<{}>", field_type))
                    .unwrap();
        }

        let token_ident = callsite_ident(&field_name);
        outerattr_or_identandtype.push(quote!(#serde_rename));
        outerattr_or_identandtype.push(quote!(#token_ident: #field_type,));
    }
    Ok(FieldsInfo {
        case,
        inner_structs,
        outerattr_or_identandtype,
    })
}
