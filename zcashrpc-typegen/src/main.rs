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
        serde_json::Value::Array(vec) => match vec.len() {
            0 => emptygen(&name, acc),
            1 => match vec.into_iter().next().unwrap() {
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
            },
            _ => enumgen(vec, &name, acc)?,
        },
        non_array => {
            panic!("Received {}, expected array", non_array.to_string())
        }
    };

    output.sort_by(|ts1, ts2| ts1.to_string().cmp(&ts2.to_string()));
    output.dedup_by(|ts1, ts2| ts1.to_string() == ts2.to_string());
    Ok(quote::quote!(pub mod #mod_name { #(#output)* }))
}

const VARIANT_NAMES: &[&str] = &["Regular", "Verbose", "VeryVerbose"];

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
    serde_rename: &mut Option<TokenStream>,
    field_name: &mut String,
    atomic_response: &mut bool,
    option: &mut bool,
) -> () {
    if special_cases::RESERVED_KEYWORDS.contains(&field_name.as_str()) {
        *serde_rename = Some(
            format!("#[serde(rename = \"{}\")]", &field_name)
                .parse()
                .unwrap(),
        );
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

fn enumgen(
    inner_nodes: Vec<serde_json::Value>,
    //This one layer out from the map that's passed to structgen!
    //Don't let the identical type signatures fool you.
    enum_name: &str,
    mut acc: Vec<TokenStream>,
) -> TypegenResult<Vec<TokenStream>> {
    assert!(inner_nodes.len() <= VARIANT_NAMES.len());
    let ident = callsite_ident(enum_name);
    let enum_code: Vec<TokenStream> = inner_nodes
        .into_iter()
        .zip(VARIANT_NAMES.iter())
        .map(|(value, variant_name)| {
            let variant_name = capitalize_first_char(&variant_name);
            let variant_name_tokens = callsite_ident(&variant_name);
            match value {
                serde_json::Value::Object(obj) => {
                    let field_data = handle_fields(enum_name, obj)?;
                    acc.extend(field_data.new_code);
                    match field_data.case {
                        special_cases::Case::Regular => {
                            let variant_body_tokens =
                                field_data.ident_val_tokens;
                            Ok(quote!(
                            #variant_name_tokens {
                                #(#variant_body_tokens)*
                            },))
                        }
                        other_case => unimplemented!(
                            "Hit special case {:?} in enumgen",
                            other_case
                        ),
                    }
                }
                non_object => {
                    dbg!(&non_object);
                    let (variant_body_tokens, new_acc) =
                        tokenize_value(&variant_name, non_object, acc.clone())?;
                    acc = new_acc;
                    Ok(quote!(#variant_name_tokens(#variant_body_tokens),))
                }
            }
        })
        .collect::<TypegenResult<Vec<TokenStream>>>()?;
    acc.push(quote!(
            #[derive(Debug, serde::Deserialize, serde::Serialize)]
            pub enum #ident {
                #(#enum_code)*
            }
    ));
    Ok(acc)
}

fn structgen(
    inner_nodes: serde_json::Map<String, serde_json::Value>,
    struct_name: &str,
    mut acc: Vec<TokenStream>,
) -> TypegenResult<(special_cases::Case, Vec<TokenStream>)> {
    let ident = callsite_ident(struct_name);
    let field_data = handle_fields(struct_name, inner_nodes)?;
    acc.extend(field_data.new_code);
    let mut ident_val_tokens = field_data.ident_val_tokens;
    let body = match field_data.case {
        special_cases::Case::Regular => {
            add_pub_keywords(&mut ident_val_tokens);
            quote!(
                pub struct #ident {
                    #(#ident_val_tokens)*
                }
            )
        }
        special_cases::Case::AlsoStandaloneEnum(chaininfofalse_tokens) => {
            // getaddressdeltas and getaddressutxos "(or, if chainInfo is true)"
            quote!(
                pub enum #ident {
                    ChainInfoFalse(#chaininfofalse_tokens),
                    ChainInfoTrue {
                        #(#ident_val_tokens)*
                    },
                }
            )
        }
        special_cases::Case::FourXs => {
            return Ok((special_cases::Case::FourXs, acc));
        }
    };

    acc.push(quote!(
        #[derive(Debug, serde::Deserialize, serde::Serialize)]
        #body
    ));
    Ok((special_cases::Case::Regular, acc))
}

fn emptygen(struct_name: &str, mut acc: Vec<TokenStream>) -> Vec<TokenStream> {
    let ident = callsite_ident(struct_name);
    acc.push(quote!(
        #[derive(Debug, serde::Deserialize, serde::Serialize)]
        pub struct #ident;
    ));
    acc
}

fn add_pub_keywords(tokens: &mut Vec<TokenStream>) {
    *tokens = tokens
        .into_iter()
        .map(|ts| match ts.clone().into_iter().next() {
            None | Some(proc_macro2::TokenTree::Punct(_)) => ts.clone(),
            _ => quote!(pub #ts),
        })
        .collect();
}

struct FieldsInfo {
    case: special_cases::Case,
    ident_val_tokens: Vec<TokenStream>,
    new_code: Vec<TokenStream>,
}
fn handle_fields(
    struct_name: &str,
    inner_nodes: serde_json::Map<String, serde_json::Value>,
) -> TypegenResult<FieldsInfo> {
    let mut ident_val_tokens: Vec<TokenStream> = Vec::new();
    let mut new_code = Vec::new();
    let mut atomic_response = true;
    let mut case = special_cases::Case::Regular;
    for (mut field_name, val) in inner_nodes {
        dbg!(&field_name);
        //special case handling
        if &field_name == "xxxx" {
            new_code = tokenize_value(struct_name, val, Vec::new(), false)?.1; // .0 unused
            case = special_cases::Case::FourXs;
            break;
        }

        let mut serde_rename = None;
        let mut option = false;
        handle_options_standalones_and_keywords(
            &mut serde_rename,
            &mut field_name,
            &mut atomic_response,
            &mut option,
        );

        //temp_acc needed because destructuring assignments are unstable
        //see https://github.com/rust-lang/rust/issues/71126 for more info
        let (mut tokenized_val, temp_acc, _terminal_enum) = tokenize_value(
            &capitalize_first_char(&field_name),
            val,
            new_code,
            false,
        )?;
        new_code = temp_acc;
        if option {
            use std::str::FromStr as _;
            tokenized_val =
                TokenStream::from_str(&format!("Option<{}>", tokenized_val))
                    .unwrap();
        }

        if atomic_response == false {
            if let special_cases::Case::AlsoStandaloneEnum(_) = case {
                //noop!
            } else {
                case = special_cases::Case::AlsoStandaloneEnum(
                    tokenized_val.clone(),
                );
            }
        }

        let token_ident = callsite_ident(&field_name);
        ident_val_tokens.push(quote!(#serde_rename));
        ident_val_tokens.push(quote!(#token_ident: #tokenized_val,));
    }
    Ok(FieldsInfo {
        case,
        new_code,
        ident_val_tokens,
    })
}

fn handle_terminal_enum(
    label: &str,
    name: &str,
    called_by_alias: bool,
) -> TokenStream {
    let variants = label
        .strip_prefix("enum:")
        .unwrap()
        .split(',')
        .map(|x| x.trim());
    let variant_idents = variants
        .clone()
        .map(|x| {
            proc_macro2::TokenTree::Ident(callsite_ident(
                &x.split('-')
                    .map(capitalize_first_char)
                    .collect::<Vec<String>>()
                    .join("_"),
            ))
            .into()
        })
        .collect::<Vec<TokenStream>>();
    let variant_idents_renames = variants
        .map(|x| format!("#[serde(rename = \"{}\")]", x).parse().unwrap())
        .collect::<Vec<TokenStream>>();
    let name_tokens = callsite_ident(&if called_by_alias {
        format!("{}Response", name)
    } else {
        name.to_string()
    });
    quote!(
        pub enum #name_tokens {
            #(#variant_idents_renames #variant_idents,)*
        }
    )
}

fn alias(
    data: serde_json::Value,
    name: &str,
    acc: Vec<TokenStream>,
) -> TypegenResult<Vec<TokenStream>> {
    let ident = callsite_ident(&name);
    let (type_body, mut acc, terminal_enum) = tokenize_value(
        &capitalize_first_char(name.trim_end_matches("Response")),
        data,
        acc,
        true,
    )?;
    if !terminal_enum {
        let aliased = quote!(
            pub type #ident = #type_body;
        );
        acc.push(aliased);
    }
    Ok(acc)
}

fn tokenize_value(
    name: &str,
    val: serde_json::Value,
    acc: Vec<TokenStream>,
    called_by_alias: bool,
) -> TypegenResult<(TokenStream, Vec<TokenStream>, bool)> {
    match val {
        serde_json::Value::String(label) => {
            tokenize_terminal(name, label.as_str(), acc, called_by_alias)
        }
        serde_json::Value::Array(vec) => {
            tokenize_array(name, vec, acc).map(|x| (x.0, x.1, false))
        }
        serde_json::Value::Object(obj) => {
            tokenize_object(name, obj, acc).map(|x| (x.0, x.1, false))
        }
        otherwise => Err(error::QuizfaceAnnotationError {
            kind: error::InvalidAnnotationKind::from(otherwise),
            location: name.to_string(),
        })?,
    }
}

fn tokenize_terminal(
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
            enumeration if enumeration.starts_with("enum:") => {
                let ident = callsite_ident(name);
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

fn tokenize_array(
    name: &str,
    mut array_of: Vec<serde_json::Value>,
    acc: Vec<TokenStream>,
) -> TypegenResult<(TokenStream, Vec<TokenStream>)> {
    let (val, acc, _terminal_enum) = tokenize_value(
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

fn tokenize_object(
    name: &str,
    val: serde_json::Map<String, serde_json::Value>,
    acc: Vec<TokenStream>,
) -> TypegenResult<(TokenStream, Vec<TokenStream>)> {
    let ident = callsite_ident(name);
    let (case, acc) = structgen(val, name, acc)?;
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
