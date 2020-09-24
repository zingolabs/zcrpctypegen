pub fn declare_rpc_response_types() -> proc_macro2::TokenStream {
    let mut rpc_typegen_calls = Vec::new();
    for file in std::fs::read_dir("json_data").expect("no json_data dir") {
        let file_name = file.expect("issue reading from file").file_name();
        let file_name_str = file_name.to_str().expect(
            "file name contains invalid characters. Error in build.rs?",
        );
        let pub_struct = format!(
            "pub {}",
            file_name_str
                .strip_suffix(".json")
                .expect("non .json file in json_data")
        );
        let relative_pathed_file_name = format!("json_data/{}", file_name_str);
        rpc_typegen_calls.push(quote::quote! {
           json_typegen::json_typegen!(#pub_struct, #relative_pathed_file_name, {
               type_alias_extant_types,
           }
                   );
        });
    }
    rpc_typegen_calls.into_iter().collect()
}
