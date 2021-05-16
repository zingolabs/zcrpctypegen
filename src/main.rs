//! In order to leverage all of Rust's type safety, this crate produces
//! a set of concrete Rust types for responses from the zcashd-RPC interface.

mod error;
mod generators;
mod utils;
use error::TypegenResult;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::BTreeMap;

/// Process quizface-formatted response specifications from files, producing
/// Rust types, in the `rpc_response_types.rs` file.
fn main() {
    let initial_comment = r#"//procedurally generated response types, note that zcashrpc-typegen
           //is in early alpha, and output is subject to change at any time.
"#;
    std::fs::write(output_path(), initial_comment).unwrap();
    let input_files = std::fs::read_dir(&std::path::Path::new(
        &std::env::args()
            .nth(1)
            .unwrap_or("./example_dir".to_string()),
    ))
    .unwrap()
    .map(Result::unwrap);
    let mut arguments = std::collections::BTreeMap::new();
    let mut responses = std::collections::BTreeMap::new();
    for filenode in input_files {
        dbg!(&filenode);
        dispatch_to_processors(filenode, &mut arguments, &mut responses);
    }
    for (name, resp) in responses {
        let mod_name = utils::get_mod_name(&name);
        let args = arguments.remove(&name);
        write_output_to_file(quote!(
            pub mod #mod_name {
                #args
                #resp
            }
        ));
        if args.is_none() {
            eprintln!("WARNING: No arguments found for '{}'", name)
        }
    }
    for (name, _resp) in arguments {
        match name.as_str() {
            "z_getoperationresult"
            | "z_getoperationstatus"
            | "getblocktemplate" => eprintln!(
                "WARNING: Missing response for '{}', this is expected behavior",
                name
            ),
            otherwise => panic!("Missing response for '{}'", otherwise),
        }
    }
}

fn dispatch_to_processors(
    filenode: std::fs::DirEntry,
    arguments: &mut BTreeMap<String, TokenStream>,
    responses: &mut BTreeMap<String, TokenStream>,
) {
    let file_name = filenode.file_name();
    let file_name = file_name.to_string_lossy();
    match file_name {
        name if name.ends_with("_response.json") => {
            match process_response(&filenode.path()) {
                Ok(processed_response) => {
                    responses.insert(
                        name.strip_suffix("_response.json")
                            .unwrap()
                            .to_string(),
                        processed_response,
                    );
                }
                Err(error::TypegenError::Annotation(err))
                    if err.kind
                        == error::InvalidAnnotationKind::Insufficient =>
                {
                    ()
                }
                Err(other_error) => {
                    panic!("Recieved error '{:?}'", other_error)
                }
            }
        }
        name if name.ends_with("_arguments.json") => {
            arguments.insert(
                name.strip_suffix("_arguments.json").unwrap().to_string(),
                process_arguments(&filenode.path()).unwrap(),
            );
        }
        name => panic!("Bad file name: '{}'", name),
    }
}

fn write_output_to_file(code: TokenStream) {
    use std::io::Write as _;
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
}

fn process_response(file: &std::path::Path) -> TypegenResult<TokenStream> {
    let (type_name, file_body) = get_name_and_body_from_file(file);
    let mut output = match file_body {
        serde_json::Value::Array(mut arg_sets) => match arg_sets.len() {
            0 => generators::emptygen(&type_name),
            1 => match arg_sets.pop().unwrap() {
                serde_json::Value::Object(args) => {
                    generators::structgen(args, &type_name).map(|x| x.1)?
                }
                val => generators::alias(val, &type_name)?,
            },
            _ => generators::response_enumgen(arg_sets, &type_name)?,
        },
        non_array => {
            panic!("Received {}, expected array", non_array.to_string())
        }
    };

    output.sort_by(|ts1, ts2| ts1.to_string().cmp(&ts2.to_string()));
    output.dedup_by(|ts1, ts2| ts1.to_string() == ts2.to_string());
    Ok(quote::quote!(#(#output)*))
}

fn process_arguments(file: &std::path::Path) -> TypegenResult<TokenStream> {
    let (type_name, file_body) = get_name_and_body_from_file(file);
    let mut output = match file_body {
        serde_json::Value::Array(mut arg_sets) => match arg_sets.len() {
            0 => generators::emptygen(&type_name),
            1 => match arg_sets.pop().unwrap() {
                serde_json::Value::Object(args) => {
                    generators::argumentgen(args, &type_name).map(|x| x.1)?
                }
                _ => panic!(
                    "Recieved arguments not in object format for file {}",
                    utils::under_to_camel(&type_name)
                ),
            },

            2 => generators::arguments_enumgen(arg_sets, &type_name)?,
            otherwise => {
                eprint!("Error, known RPC help output contains a maximum of two sets of arguments, but we found {} this time.", otherwise);
                generators::arguments_enumgen(arg_sets, &type_name)?
            }
        },
        non_array => {
            panic!("Received {}, expected array", non_array.to_string())
        }
    };

    output.sort_by(|ts1, ts2| ts1.to_string().cmp(&ts2.to_string()));
    output.dedup_by(|ts1, ts2| ts1.to_string() == ts2.to_string());
    Ok(quote::quote!(#(#output)*))
}

fn get_name_and_body_from_file(
    file: &std::path::Path,
) -> (String, serde_json::Value) {
    let (file_name, file_body) = get_data(file);
    let type_name = utils::under_to_camel(&file_name);
    (type_name, file_body)
}

fn get_data(file: &std::path::Path) -> (String, serde_json::Value) {
    let file_body =
        from_file_deserialize(&file).expect("Couldn't unpack file!");
    (
        file.file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .strip_suffix(".json")
            .unwrap()
            .to_string(),
        file_body,
    )
}

/// This function provides input for the OS interface that we access via
/// std::process, and std::fs.
const TYPEGEN_VERSION: &'static str = env!("CARGO_PKG_VERSION");
fn output_path() -> std::ffi::OsString {
    use std::ffi::OsString;
    let input_dirname = "../quizface/output/";
    let in_version = std::fs::read_dir(&input_dirname)
        .expect(&format!("Missing interpretations in {}.", &input_dirname))
        .map(|x| x.unwrap().file_name())
        .collect::<Vec<OsString>>()
        .pop()
        .expect("Can't retrieve input dir name.");
    let outstring = format!(
        "./output/{}_{}/rpc_response_types.rs",
        in_version
            .into_string()
            .expect("Couldn't get String from OsString."),
        TYPEGEN_VERSION
    );
    let outpath = std::path::Path::new(&outstring);
    std::fs::create_dir_all(outpath.parent().expect("Couldn't create parent."))
        .expect("Couldn't create outdir.");
    std::ffi::OsString::from(
        std::env::args()
            .nth(2)
            .unwrap_or(outpath.to_str().unwrap().into()),
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

#[cfg(test)]
mod unit {
    mod intermediate {
        use crate::*;
        #[test]
        fn process_response_getinfo() {
            let getinfo_path = std::path::Path::new(
                "./test_data/quizface_output/getinfo_response.json",
            );
            let output = process_response(getinfo_path);
            assert_eq!(
                output.unwrap().to_string(),
                test_consts::GETINFO_RESPONSE
            );
        }
        mod test_consts {
            pub(super) const GETINFO_RESPONSE: &str = "# [derive \
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
    rust_decimal :: Decimal , }";
        }
    }
}
