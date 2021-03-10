//! In order to leverage all of Rust's type safety, this crate produces
//! a set of concrete Rust types for responses from the zcashd-RPC interface.

mod error;
mod special_cases;
use error::TypegenResult;
use proc_macro2::TokenStream;
use quote::quote;

/// Process quizface-formatted response specifications from files, producing
/// Rust types, in the `rpc_response_types.rs` file.
fn main() {
    let initial_comment = r#"//procedurally generated response types, note that zcashrpc-typegen
           //is in early alpha, and output is subject to change at any time.
"#;
    use std::io::Write as _;
    std::fs::write(output_path(), initial_comment).unwrap();
    for filenode in std::fs::read_dir(&std::path::Path::new(
        &std::env::args()
            .nth(1)
            .unwrap_or("./example_dir".to_string()),
    ))
    .unwrap()
    {
        if let Ok(code) = process_response(
            &filenode.expect("Problem getting direntry!").path(),
        ) {
            let mut outfile = std::fs::OpenOptions::new()
                .append(true)
                .open(output_path())
                .unwrap();
            outfile.write_all(code.to_string().as_bytes()).unwrap();
            assert!(std::process::Command::new("rustfmt")
                .arg(output_path())
                .output()
                .unwrap()
                .status
                .success());
        } else {
            todo!("Holy moly something is messed up!");
        }
    }
}

fn process_response(file: &std::path::Path) -> TypegenResult<TokenStream> {
    let acc = Vec::new();
    let (name, file_body) = get_data(file);
    let mod_name = callsite_ident(&match file
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .strip_suffix(".json")
        .unwrap()
    {
        name if special_cases::RESERVED_KEYWORDS.contains(&name) => {
            format!("{}_mod", name)
        }
        name => name.to_string(),
    });
    let mut output = match file_body {
        serde_json::Value::Object(obj) => {
            structgen(obj, &name, acc)
                .expect(&format!(
                    "file_body of {} struct failed to match",
                    file.to_str().unwrap()
                ))
                .1
        }
        val => alias(val, &name, acc).expect(&format!(
            "file_body of {} alias failed to match",
            file.to_str().unwrap()
        )),
    };

    output.sort_by(|ts1, ts2| ts1.to_string().cmp(&ts2.to_string()));
    output.dedup_by(|ts1, ts2| ts1.to_string() == ts2.to_string());
    Ok(quote::quote!(pub mod #mod_name { #(#output)* }))
}

fn get_data(file: &std::path::Path) -> (String, serde_json::Value) {
    let file_body =
        from_file_deserialize(&file).expect("Couldn't unpack file!");
    let mut name = capitalize_first_char(
        file.file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .strip_suffix(".json")
            .unwrap(),
    );
    name.push_str("Response");
    (name, file_body)
}

/// This function provides input for the OS interface that we access via
/// std::process, and std::fs.
fn output_path() -> std::ffi::OsString {
    std::ffi::OsString::from(
        std::env::args()
            .nth(2)
            .unwrap_or("./../src/client/rpc_response_types.rs".to_string()),
    )
}

/// Handles data access from fs location through deserialization
fn from_file_deserialize(
    file_path: &std::path::Path,
) -> TypegenResult<serde_json::Value> {
    let from_io_to_fs = error::FSError::from_io_error(file_path);
    let mut file = std::fs::File::open(file_path).map_err(&from_io_to_fs)?;
    let mut file_body = String::new();
    use std::io::Read as _;
    file.read_to_string(&mut file_body)
        .map_err(&from_io_to_fs)?;
    let file_body_json =
        serde_json::de::from_str(&file_body).map_err(|err| {
            error::JsonError::from_serde_json_error(err, file_body)
        })?;
    Ok(file_body_json)
}

/// Simple wrapper that always generates Idents with "call_site" spans.
fn callsite_ident(name: &str) -> proc_macro2::Ident {
    proc_macro2::Ident::new(name, proc_macro2::Span::call_site())
}

fn handle_options_standalones_and_keywords(
    rename: &mut TokenStream,
    field_name: &mut String,
    atomic_response: &mut bool,
    option: &mut bool,
) -> () {
    if special_cases::RESERVED_KEYWORDS.contains(&field_name.as_str()) {
        *rename = format!("#[serde(rename = \"{}\")]", &field_name)
            .parse()
            .unwrap();
        field_name.push_str("_field");
    }

    if field_name.starts_with("alsoStandalone<") {
        *field_name = field_name
            .trim_end_matches(">")
            .trim_start_matches("alsoStandalone<")
            .to_string();
        *atomic_response = false;
    } else if field_name.starts_with("Option<") {
        *field_name = field_name
            .trim_end_matches(">")
            .trim_start_matches("Option<")
            .to_string();
        *option = true;
    }
}
fn structgen(
    inner_nodes: serde_json::Map<String, serde_json::Value>,
    struct_name: &str,
    mut acc: Vec<TokenStream>,
) -> TypegenResult<(Option<special_cases::Case>, Vec<TokenStream>)> {
    let mut ident_val_tokens: Vec<TokenStream> = Vec::new();
    let mut atomic_response = true;
    let mut chaininfofalse_tokens = TokenStream::new();
    let mut rename = TokenStream::new();
    // The default collection behind a serde_json_map is a BTreeMap
    // and being the predicate of "in" causes into_iter to be called.
    // See: https://docs.serde.rs/src/serde_json/map.rs.html#3
    for (mut field_name, val) in inner_nodes {
        let mut option = false;
        dbg!(&field_name);
        //special case handling
        if &field_name == "xxxx" {
            acc = tokenize_value(struct_name, val, acc)?.1; // .0 unused
            return Ok((Some(special_cases::Case::FourXs), acc));
        }

        handle_options_standalones_and_keywords(
            &mut rename,
            &mut field_name,
            &mut atomic_response,
            &mut option,
        );

        let (mut tokenized_val, temp_acc) =
            tokenize_value(&capitalize_first_char(&field_name), val, acc)?;
        acc = temp_acc;
        if option {
            use std::str::FromStr as _;
            tokenized_val =
                TokenStream::from_str(&format!("Option<{}>", tokenized_val))
                    .unwrap();
        }

        if chaininfofalse_tokens.is_empty() {
            chaininfofalse_tokens = tokenized_val.clone();
        }

        let token_ident = callsite_ident(&field_name);
        ident_val_tokens.push(quote!(#rename));
        ident_val_tokens.push(quote!(#token_ident: #tokenized_val,));
    }

    if atomic_response {
        ident_val_tokens = ident_val_tokens
            .into_iter()
            .map(|ts| match ts.clone().into_iter().next() {
                None => ts,
                Some(proc_macro2::TokenTree::Punct(_)) => ts,
                _ => quote!(pub #ts),
            })
            .collect();
    }

    let ident = callsite_ident(struct_name);
    let body = if atomic_response {
        quote!(
            pub struct #ident {
                #(#ident_val_tokens)*
            }
        )
    } else {
        // getaddressdeltas and getaddressutxos "(or, if chainInfo is true)"
        quote!(
            pub enum #ident {
                ChainInfoFalse(#chaininfofalse_tokens),
                ChainInfoTrue {
                    #(#ident_val_tokens)*
                },
            }
        )
    };

    acc.push(quote!(
        #[derive(Debug, serde::Deserialize, serde::Serialize)]
        #body
    ));
    Ok((None, acc))
}

fn alias(
    data: serde_json::Value,
    name: &str,
    acc: Vec<TokenStream>,
) -> TypegenResult<Vec<TokenStream>> {
    let ident = callsite_ident(&name);
    let (type_body, mut acc) = tokenize_value(
        &capitalize_first_char(name.trim_end_matches("Response")),
        data,
        acc,
    )?;
    let aliased = quote!(
        pub type #ident = #type_body;
    );
    acc.push(aliased);
    Ok(acc)
}

fn tokenize_value(
    name: &str,
    val: serde_json::Value,
    acc: Vec<TokenStream>,
) -> TypegenResult<(TokenStream, Vec<TokenStream>)> {
    match val {
        serde_json::Value::String(label) => {
            tokenize_terminal(name, label.as_str()).map(|x| (x, acc))
        }
        serde_json::Value::Array(vec) => tokenize_array(name, vec, acc),
        serde_json::Value::Object(obj) => tokenize_object(name, obj, acc),
        otherwise => Err(error::QuizfaceAnnotationError {
            kind: error::InvalidAnnotationKind::from(otherwise),
            location: name.to_string(),
        })?,
    }
}

fn tokenize_terminal(name: &str, label: &str) -> TypegenResult<TokenStream> {
    Ok(match label {
        "Decimal" => quote!(rust_decimal::Decimal),
        "bool" => quote!(bool),
        "String" => quote!(String),
        "hexadecimal" => quote!(String),
        "INSUFFICIENT" => quote!(compile_error!(
            "Insufficient zcash-cli help output to autogenerate type"
        )),
        otherwise => {
            return Err(error::QuizfaceAnnotationError {
                kind: error::InvalidAnnotationKind::from(
                    serde_json::Value::String(otherwise.to_string()),
                ),
                location: name.to_string(),
            }
            .into())
        }
    })
}

fn tokenize_array(
    name: &str,
    mut array_of: Vec<serde_json::Value>,
    acc: Vec<TokenStream>,
) -> TypegenResult<(TokenStream, Vec<TokenStream>)> {
    let (val, acc) = tokenize_value(
        name,
        array_of.pop().ok_or(error::QuizfaceAnnotationError {
            kind: error::InvalidAnnotationKind::EmptyArray,
            location: name.to_string(),
        })?,
        acc,
    )?;
    Ok((quote!(Vec<#val>), acc))
}

fn tokenize_object(
    name: &str,
    val: serde_json::Map<String, serde_json::Value>,
    acc: Vec<TokenStream>,
) -> TypegenResult<(TokenStream, Vec<TokenStream>)> {
    let ident = callsite_ident(name);
    let (special_case, acc) = structgen(val, name, acc)?;
    if let Some(special_case) = special_case {
        match special_case {
            special_cases::Case::FourXs => {
                Ok((quote!(std::collections::HashMap<String, #ident>), acc))
            }
        }
    } else {
        Ok((quote!(#ident), acc))
    }
}

fn capitalize_first_char(input: &str) -> String {
    let mut ret = input.to_string();
    let ch = ret.remove(0);
    ret.insert(0, ch.to_ascii_uppercase());
    ret
}

#[cfg(test)]
mod unit {
    mod atomic {
        use crate::*;
        #[test]
        fn tokenize_value_string() {
            let quoted_string = tokenize_value(
                "some_field",
                serde_json::json!("String"),
                Vec::new(),
            );
            assert_eq!(
                quote!(String).to_string(),
                quoted_string.unwrap().0.to_string(),
            );
        }
        #[test]
        fn tokenize_value_number() {
            let quoted_number = tokenize_value(
                "some_field",
                serde_json::json!("Decimal"),
                Vec::new(),
            );
            assert_eq!(
                quote!(rust_decimal::Decimal).to_string(),
                quoted_number.unwrap().0.to_string(),
            );
        }
        #[test]
        fn tokenize_value_bool() {
            let quoted_bool = tokenize_value(
                "some_field",
                serde_json::json!("bool"),
                Vec::new(),
            );
            assert_eq!(
                quote!(bool).to_string(),
                quoted_bool.unwrap().0.to_string(),
            );
        }
    }
    mod intermediate {
        use crate::*;
        #[test]
        fn process_response_getinfo() {
            let getinfo_path = std::path::Path::new(
                "./test_data/quizface_output/getinfo.json",
            );
            let output = process_response(getinfo_path);
            assert_eq!(
                output.unwrap().to_string(),
                test_consts::GETINFO_RESPONSE
            );
        }
        #[test]
        fn tokenize_object_simple_unnested() {
            let quoted_object = tokenize_value(
                "somefield",
                serde_json::json!(
                    {
                        "inner_a": "String",
                        "inner_b": "bool",
                        "inner_c": "Decimal",
                    }
                ),
                Vec::new(),
            )
            .unwrap();
            assert_eq!(
                quote!(somefield).to_string(),
                quoted_object.0.to_string(),
            );
            assert_eq!(
                quoted_object.1[0].to_string(),
                test_consts::SIMPLE_UNNESTED_RESPONSE,
            );
        }
    }
}

#[cfg(test)]
mod test_consts {
    pub(super) const GETINFO_RESPONSE: &str = "pub mod getinfo { # [derive \
    (Debug , serde :: Deserialize , serde :: Serialize)] pub struct \
    GetinfoResponse { pub proxy : Option < String > , pub balance : \
    rust_decimal :: Decimal , pub blocks : rust_decimal :: Decimal , pub \
    connections : rust_decimal :: Decimal , pub difficulty : rust_decimal :: \
    Decimal , pub errors : String , pub keypoololdest : rust_decimal :: \
    Decimal , pub keypoolsize : rust_decimal :: Decimal , pub paytxfee : \
    rust_decimal :: Decimal , pub protocolversion : rust_decimal :: Decimal , \
    pub relayfee : rust_decimal :: Decimal , pub testnet : bool , pub \
    timeoffset : rust_decimal :: Decimal , pub unlocked_until : rust_decimal \
    :: Decimal , pub version : rust_decimal :: Decimal , pub walletversion : \
    rust_decimal :: Decimal , } }";
    pub(super) const SIMPLE_UNNESTED_RESPONSE: &str = "# [derive (Debug , \
    serde :: Deserialize , serde :: Serialize)] pub struct somefield { pub \
    inner_a : String , pub inner_b : bool , pub inner_c : rust_decimal :: \
    Decimal , }";
}
