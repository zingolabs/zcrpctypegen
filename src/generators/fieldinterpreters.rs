use crate::utils::{callsite_ident, camel_to_under, under_to_camel};
use crate::TypegenResult;
use proc_macro2::TokenStream;
use quote::quote;
use serde_json::{Map, Value};

fn handle_options_and_keywords(
    serde_rename: &mut Option<TokenStream>,
    field_name: &mut String,
    optional: &mut bool,
) -> () {
    if field_name.starts_with("Option<") {
        *field_name = field_name
            .trim_end_matches(">")
            .trim_start_matches("Option<")
            .to_string();
        *optional = true;
    }
    if crate::utils::RUST_KEYWORDS.contains(&field_name.as_str()) {
        *serde_rename = Some(
            format!("#[serde(rename = \"{}\")]", &field_name)
                .parse()
                .unwrap(),
        );
        field_name.push_str("_field");
    }
}

fn handle_option(name_hint: &mut String, optional: &mut bool) -> () {
    if name_hint.starts_with("Option<") {
        *name_hint = name_hint
            .trim_end_matches(">")
            .trim_start_matches("Option<")
            .to_string();
        *optional = true;
    }
}

fn sort_nodes(nodes: Map<String, Value>) -> Vec<(String, Value)> {
    //! This depends on keys having names like `1_foo 2_spam`...
    let mut nodes_as_vec = nodes.into_iter().collect::<Vec<(String, Value)>>();
    nodes_as_vec.sort_by(|(key1, _), (key2, _)| key1.cmp(&key2));
    nodes_as_vec
}

pub(crate) struct NamedFieldsInfo {
    pub(crate) case: super::FourXs,
    pub(crate) outerattr_or_identandtype: Vec<TokenStream>,
    pub(crate) inner_structs: Vec<TokenStream>,
}
pub(crate) fn interpret_named_fields(
    struct_name: &str,
    inner_nodes: serde_json::Map<String, serde_json::Value>,
) -> TypegenResult<NamedFieldsInfo> {
    let mut outerattr_or_identandtype: Vec<TokenStream> = Vec::new();
    let mut inner_structs = Vec::new();
    let mut case = false;
    for (mut field_name, val) in inner_nodes {
        //special case handling
        if &field_name == "xxxx" {
            inner_structs = super::tokenize::value(struct_name, val)?.1; // .0 unused
            case = true;
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
#[derive(Debug)]
pub(crate) struct EnumeratedFieldsInfo {
    pub(crate) indexed_type: Vec<TokenStream>,
    pub(crate) inner_structs: Vec<TokenStream>,
}
fn process_name_hint(mut name_hint: String) -> String {
    match name_hint.parse::<u8>() {
        Ok(_) => name_hint.insert_str(0, "Arg"),
        Err(_) => name_hint = name_hint[2..].to_string(),
    }
    under_to_camel(&name_hint)
}
pub(crate) fn handle_enumerated_fields(
    inner_nodes: serde_json::Map<String, serde_json::Value>,
) -> TypegenResult<EnumeratedFieldsInfo> {
    //! inner_node Values might be terminal
    let mut indexed_type: Vec<TokenStream> = Vec::new();
    let mut inner_structs: Vec<TokenStream> = Vec::new();
    for (mut raw_name_hint, val) in sort_nodes(inner_nodes) {
        let mut optional = false;
        handle_option(&mut raw_name_hint, &mut optional);
        let name_hint = process_name_hint(raw_name_hint);
        let (mut field_type, new_struct, _terminal_enum) =
            super::tokenize::value(&name_hint, val)?;
        inner_structs.extend(new_struct);
        if optional {
            use std::str::FromStr as _;
            field_type =
                TokenStream::from_str(&format!("Option<{}>", field_type))
                    .unwrap();
            indexed_type.push(
                quote!(#[serde(skip_serializing_if = "Option::is_none")]),
            );
        }

        indexed_type.push(quote!(#field_type,));
    }
    Ok(EnumeratedFieldsInfo {
        indexed_type,
        inner_structs,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn handle_enumerated_fields_() {
        use serde_json::{json, Map, Value};
        let mut input_inner_nodes = Map::<String, Value>::new();
        input_inner_nodes
            .insert("1_anarg".to_string(), json!("String".to_string()));

        let expected_output = EnumeratedFieldsInfo {
            indexed_type: vec![quote!(String,)],
            inner_structs: vec![],
        };
        let observed_output =
            handle_enumerated_fields(input_inner_nodes).unwrap();
        assert_eq!(
            expected_output.indexed_type[0].to_string(),
            observed_output.indexed_type[0].to_string()
        );
    }
    #[test]
    fn handle_options_and_keywords_non_optional_non_keyword() {
        let mut observed_serde_rename = None;
        let mut observed_field_name = "fooople".to_string();
        let mut observed_option = false;
        handle_options_and_keywords(
            &mut observed_serde_rename,
            &mut observed_field_name,
            &mut observed_option,
        );
        assert!(observed_serde_rename.is_none());
        assert_eq!(observed_field_name, "fooople".to_string());
        assert_eq!(observed_option, false);
    }
    #[test]
    fn handle_options_and_keywords_optional_keyword() {
        let mut observed_serde_rename = None;
        let mut observed_field_name = "Option<yield>".to_string();
        let mut observed_option = false;
        handle_options_and_keywords(
            &mut observed_serde_rename,
            &mut observed_field_name,
            &mut observed_option,
        );
        assert!(observed_serde_rename.is_some());
        assert_eq!(observed_field_name, "yield_field".to_string());
        assert_eq!(observed_option, true);
    }
    #[test]
    fn handle_option_true() {
        let mut observed_name_hint = "Option<struct>".to_string();
        let mut observed_option = false;
        handle_option(&mut observed_name_hint, &mut observed_option);
        assert_eq!(observed_name_hint, "struct".to_string());
        assert_eq!(observed_option, true);
    }
    #[test]
    fn handle_option_false() {
        let mut observed_name_hint = "mimblewimble".to_string();
        let mut observed_option = false;
        handle_option(&mut observed_name_hint, &mut observed_option);
        assert_eq!(observed_name_hint, "mimblewimble".to_string());
        assert_eq!(observed_option, false);
    }
}
