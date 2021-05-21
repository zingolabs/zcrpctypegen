use crate::generators::utils::{handle_option, handle_options_and_keywords};
use crate::utils::{callsite_ident, camel_to_under, under_to_camel};
use crate::TypegenResult;
use proc_macro2::TokenStream;
use quote::quote;
pub(crate) struct NamedFieldsInfo {
    pub(crate) case: super::utils::FourXs,
    pub(crate) outerattr_or_identandtype: Vec<TokenStream>,
    pub(crate) inner_structs: Vec<TokenStream>,
}
pub(crate) fn interpret_named_fields(
    struct_name: &str,
    inner_nodes: serde_json::Map<String, serde_json::Value>,
) -> TypegenResult<NamedFieldsInfo> {
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
        let mut optional = false;
        handle_options_and_keywords(
            &mut serde_rename,
            &mut field_name,
            &mut optional,
        );
        field_name = camel_to_under(&field_name);

        let (mut field_type, new_structs, _terminal_enum) =
            super::tokenize::value(&under_to_camel(&field_name), val)?;
        inner_structs.extend(new_structs);
        if optional {
            use std::str::FromStr as _;
            field_type =
                TokenStream::from_str(&format!("Option<{}>", field_type))
                    .unwrap();
        }

        let token_ident = callsite_ident(&field_name);
        outerattr_or_identandtype.push(quote!(#serde_rename));
        outerattr_or_identandtype.push(quote!(#token_ident: #field_type,));
    }
    Ok(NamedFieldsInfo {
        case,
        inner_structs,
        outerattr_or_identandtype,
    })
}
pub(crate) struct EnumeratedFieldsInfo {
    pub(crate) indexed_type: Vec<TokenStream>,
    pub(crate) inner_structs: Vec<TokenStream>,
}
pub(crate) fn handle_enumerated_fields(
    inner_nodes: serde_json::Map<String, serde_json::Value>,
) -> TypegenResult<EnumeratedFieldsInfo> {
    let mut indexed_type: Vec<TokenStream> = Vec::new();
    let mut inner_structs: Vec<TokenStream> = Vec::new();
    //Bypass unreachable code warning
    if true {
        todo!("inner_nodes has to be ordered by beginning number");
    }
    for (mut name_hint, val) in inner_nodes {
        let mut optional = false;
        handle_option(&mut name_hint, &mut optional);
        name_hint = camel_to_under(&name_hint);

        let (mut field_type, new_struct, _terminal_enum) =
            super::tokenize::value(&under_to_camel(&name_hint), val)?;
        inner_structs.extend(new_struct);
        if optional {
            use std::str::FromStr as _;
            field_type =
                TokenStream::from_str(&format!("Option<{}>", field_type))
                    .unwrap();
        }

        indexed_type.push(quote!(#field_type,));
    }
    Ok(EnumeratedFieldsInfo {
        indexed_type,
        inner_structs,
    })
}
