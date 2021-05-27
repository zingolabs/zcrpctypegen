//! In order to leverage all of Rust's type safety, this crate produces
//! a set of concrete Rust types for responses from the zcashd-RPC interface.

mod error;
mod generators;
mod utils;
use error::TypegenResult;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Process quizface-formatted response specifications from files, producing
/// Rust types, in the `rpc_response_types.rs` file.
fn main() {
    let initial_comment = r#"//procedurally generated response types, note that zcashrpc-typegen
           //is in early alpha, and output is subject to change at any time.
"#;
    let indir_name = &std::env::args()
        .nth(1)
        .expect("Please pass an input directory of quizface interpretations.");
    let indir = &std::path::Path::new(indir_name);
    let input_basename = indir.file_name().unwrap();

    let output_path = output_path(&input_basename.to_string_lossy());
    std::fs::write(&output_path, initial_comment).unwrap();
    let input_files = std::fs::read_dir(indir).unwrap().map(Result::unwrap);
    let mut arguments = std::collections::BTreeMap::new();
    let mut responses = std::collections::BTreeMap::new();
    for filenode in input_files {
        dispatch_to_processors(filenode, &mut arguments, &mut responses);
    }
    for (name, resp) in responses {
        let args = arguments.remove(&name);
        if args.is_none() {
            panic!("WARNING: No arguments found for '{}'", name)
        }
        let mod_name = utils::get_mod_name(&name);
        write_output_to_file(
            quote!(
                pub mod #mod_name {
                    #args
                    #resp
                }
            ),
            &output_path,
        );
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
    let file_name = file_name.to_str().expect("Invalid unicode in RPC name!");
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

fn write_output_to_file(code: TokenStream, output_path: &PathBuf) {
    use std::io::Write as _;
    let mut outfile = std::fs::OpenOptions::new()
        .append(true)
        .open(&output_path)
        .unwrap();
    outfile.write_all(code.to_string().as_bytes()).unwrap();
    assert!(std::process::Command::new("rustfmt")
        .arg(&output_path)
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
                    generators::namedfield_structgen(args, &type_name)
                        .map(|x| x.1)?
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
            0 => Vec::new(),
            1 => match arg_sets.pop().unwrap() {
                serde_json::Value::Object(args) => {
                    generators::argumentgen(args, &type_name)?
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
fn output_path(input_basename: &str) -> std::path::PathBuf {
    let outstring = format!(
        "./output/{}_{}/rpc_response_types.rs",
        input_basename, TYPEGEN_VERSION
    );
    //let outname = std::env::args().nth(2).unwrap_or(outstring);
    let outpath = std::path::Path::new(&outstring);
    std::fs::create_dir_all(outpath.parent().expect("Couldn't create parent."))
        .expect("Couldn't create outdir.");
    outpath.to_path_buf()
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
                "./tests/data/input/test_quizface_output/getinfo_response.json",
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

#[allow(warnings)]
#[cfg(test)]
mod test {
    use super::*;
    use std::collections::BTreeMap;
    const TESTS_DIR: &str = "./tests/data/observed/";
    fn create_direntries_for_dtp(
        file_name: &std::ffi::OsStr,
    ) -> std::fs::DirEntry {
        let test_file = std::fs::File::create(file_name);
        std::fs::read_dir(TESTS_DIR)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
    }
    #[should_panic(expected = "Invalid unicode in RPC name!")]
    #[test]
    #[cfg(target_family = "unix")]
    fn dispatch_to_processors_invalid_utf8_in_fn() {
        //! reference:  https://doc.rust-lang.org/std/ffi/struct.OsString.html#examples-13
        std::fs::remove_dir_all(TESTS_DIR);
        std::fs::create_dir(TESTS_DIR);
        use std::ffi::OsStr;
        use std::path::Path;
        let invalid_utf8_bytes = [
            46, 47, 116, 101, 115, 116, 115, 47, 100, 97, 116, 97, 47, 111, 98,
            115, 101, 114, 118, 101, 100, 47, 0x66, 0x6f, 0x80, 0x6f,
        ];
        let os_str: &std::ffi::OsStr =
            std::os::unix::ffi::OsStrExt::from_bytes(&invalid_utf8_bytes);
        let input_direntry = create_direntries_for_dtp(&os_str);

        dispatch_to_processors(
            input_direntry,
            &mut BTreeMap::new(),
            &mut BTreeMap::new(),
        );
    }
    #[test]
    fn dispatch_to_processors_invalid_fn_end() {
        dbg!(std::fs::remove_dir_all(TESTS_DIR));
        dbg!(std::fs::create_dir(TESTS_DIR));
        let stringy_input_inval_name = 
        let input_invalid_name = std::ffi::OsStr::new(&format!(
            "{}/{}",
            TESTS_DIR, "a_bad_end.json"
        ));
        let input_direntry = create_direntries_for_dtp(&input_invalid_name);
        dbg!(&input_direntry);
        dispatch_to_processors(
            input_direntry,
            &mut BTreeMap::new(),
            &mut BTreeMap::new(),
        );
    }
    #[ignore]
    #[test]
    fn from_file_deserialize_invalid_file_path() {}
    #[ignore]
    #[test]
    fn from_file_deserialize_invalid_file_body() {}
    #[ignore]
    #[test]
    fn get_data_no_json_suffix() {}
    #[ignore]
    #[test]
    #[should_panic(expected = "Received {}, expected array")]
    fn process_response_non_array_body() {
        //! This logic is (or was) duplicated in process_arguments
        let fake_file_path = todo!();
        let observed_pr_result = process_response(fake_file_path);
    }
}
