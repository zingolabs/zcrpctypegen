use std::str::FromStr;
pub fn declare_rpc_response_types() -> proc_macro2::TokenStream {
    let mut str_literals = Vec::new();
    for file in std::fs::read_dir("json_data").expect("no json_data dir") {
        str_literals.push(
            //A change to the build script could easily break this. We're
            //eschewing a lot of rust's type-safety by building this from a
            //String, instead of creating stongly-typed Tokens and collecting
            //them. Expect a rework in the non-immediate future.
            proc_macro2::TokenStream::from_str(
                &format!(
                    "json_typegen::json_typegen!(\"pub {x}\", \"json_data/{x}.json\");",
                    x = file
                        .expect("failed to unwrap file")
                        .file_name()
                        .to_str()
                        .expect("invalid os string??")
                        .strip_suffix(".json")
                        .expect("non .json file in json_data")
                )
            )
                .expect("Failed to parse into TokenStream"),
        );
    }
    str_literals.into_iter().collect()
}
