use proc_macro2::TokenStream;
use quote::quote;

pub(crate) enum FourXs {
    False,
    True,
}

pub(crate) fn handle_options_and_keywords(
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
    if crate::utils::RESERVED_KEYWORDS.contains(&field_name.as_str()) {
        *serde_rename = Some(
            format!("#[serde(rename = \"{}\")]", &field_name)
                .parse()
                .unwrap(),
        );
        field_name.push_str("_field");
    }
}

pub(crate) fn handle_option(name_hint: &mut String, optional: &mut bool) -> () {
    if name_hint.starts_with("Option<") {
        *name_hint = name_hint
            .trim_end_matches(">")
            .trim_start_matches("Option<")
            .to_string();
        *optional = true;
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

#[cfg(test)]
mod test {
    use super::*;
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
    #[test]
    fn add_pub_keywords_and_not_to_attrs() {
        let mut startcode = vec![
            quote!(field_one: foo,),
            quote!(field_two: bar,),
            quote!(#[some_attribute]),
            quote!(attributed_field: squelch,),
        ];
        add_pub_keywords(&mut startcode);
        let expected_code = quote!(
            pub field_one: foo,
            pub field_two: bar,
            #[some_attribute]
            pub attributed_field: squelch,
        );
        assert_eq!(
            startcode.into_iter().collect::<TokenStream>().to_string(),
            expected_code.to_string()
        );
    }
}
