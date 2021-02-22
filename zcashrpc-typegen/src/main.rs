mod special_cases;

type GenericResult<T> = Result<T, Box<dyn std::error::Error>>;

fn main() {
    let mut code = Vec::new();
    for filenode in std::fs::read_dir(&std::path::Path::new(
        &std::env::args()
            .nth(1)
            .unwrap_or("./example_dir".to_string()),
    ))
    .unwrap()
    {
        code.push(process_response(
            &filenode.expect("Problem getting direntry!").path(),
            proc_macro2::TokenStream::new(),
        ));
    }
    use std::io::Write as _;
    std::fs::File::create(output_path())
        .unwrap()
        .write(
            code.into_iter()
                .collect::<proc_macro2::TokenStream>()
                .to_string()
                .as_bytes(),
        )
        .unwrap();

    assert!(std::process::Command::new("rustfmt")
        .arg(output_path().to_string_lossy().to_string())
        .output()
        .unwrap()
        .status
        .success());
}

fn process_response(
    file: &std::path::Path,
    acc: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let file_body = get_data(&file).expect("Couldn't unpack file!");
    let name = capitalize_first_char(
        file.file_name()
            .unwrap()
            .to_string_lossy()
            .strip_suffix(".json")
            .unwrap(),
    );
    match file_body {
        serde_json::Value::Object(obj) => {
            typegen(obj, &name, acc)
                .expect("file_body failed to match")
                .1
        }
        val => alias(val, &name, acc).expect("file_body failed to match"),
    }
}

fn output_path() -> Box<std::path::Path> {
    Box::from(std::path::Path::new(
        &std::env::args().nth(2).unwrap_or("./output.rs".to_string()),
    ))
}

fn get_data(file: &std::path::Path) -> GenericResult<serde_json::Value> {
    let mut file = std::fs::File::open(file)?;
    let mut file_body = String::new();
    use std::io::Read as _;
    file.read_to_string(&mut file_body)?;
    let file_body = serde_json::de::from_str(&file_body)?;
    Ok(file_body)
}

fn typegen(
    data: serde_json::Map<String, serde_json::Value>,
    name: &str,
    mut acc: proc_macro2::TokenStream,
) -> GenericResult<(Option<special_cases::Case>, proc_macro2::TokenStream)> {
    let mut code = Vec::new();
    // The default collection behind a serde_json_map is a BTreeMap
    // and being the predicate of "in" causes into_iter to be called.
    // See: https://docs.serde.rs/src/serde_json/map.rs.html#3
    for (field_name, val) in data {
        dbg!(&field_name);
        //special case handling
        if &field_name == "xxxx" {
            acc = quote_value(name, val, acc)?.1; //We ignore the first field
            return Ok((Some(special_cases::Case::FourXs), acc));
        }

        //println!("Got field: {}, {}", field_name, val);
        let key = proc_macro2::Ident::new(
            &field_name,
            proc_macro2::Span::call_site(),
        );
        let (val, temp_acc) =
            quote_value(&capitalize_first_char(&field_name), val, acc)?;
        acc = temp_acc;
        let added_code = quote::quote!(pub #key: #val,);
        code.push(added_code);
    }

    let ident = proc_macro2::Ident::new(name, proc_macro2::Span::call_site());
    acc.extend(quote::quote!(
        #[derive(Debug, serde::Deserialize, serde::Serialize)]
        pub struct #ident {
            #(#code)*
        }
    ));
    Ok((None, acc))
}

fn alias(
    data: serde_json::Value,
    name: &str,
    acc: proc_macro2::TokenStream,
) -> GenericResult<proc_macro2::TokenStream> {
    if let serde_json::Value::Object(_) = data {
        unimplemented!("We don't want to create struct aliases.")
    }
    let ident = proc_macro2::Ident::new(&name, proc_macro2::Span::call_site());
    let (type_body, mut acc) =
        quote_value(&capitalize_first_char(name), data, acc)?;
    let aliased = quote::quote!(
        pub type #ident = #type_body;
    );
    acc.extend(aliased);
    Ok(acc)
}

fn quote_value(
    name: &str,
    val: serde_json::Value,
    acc: proc_macro2::TokenStream,
) -> GenericResult<(proc_macro2::TokenStream, proc_macro2::TokenStream)> {
    match val {
        serde_json::Value::String(kind) => quote_terminal(kind.as_str(), acc),
        serde_json::Value::Array(vec) => quote_array(name, vec, acc),
        serde_json::Value::Object(obj) => quote_object(name, obj, acc),
        otherwise => {
            Err(format!("Did not expect to recieve: \n {}", otherwise).into())
        }
    }
}

fn quote_terminal(
    val: &str,
    acc: proc_macro2::TokenStream,
) -> GenericResult<(proc_macro2::TokenStream, proc_macro2::TokenStream)> {
    Ok((
        match val {
            //Todo: Make this better than manual option variants
            "Decimal" => quote::quote!(rust_decimal::Decimal),
            "Option<Decimal>" => quote::quote!(Option<rust_decimal::Decimal>),
            "bool" => quote::quote!(bool),
            "Option<bool>" => quote::quote!(Option<bool>),
            "String" => quote::quote!(String),
            "Option<String>" => quote::quote!(Option<String>),
            otherwise => {
                return Err(format!(
                    "Unexpected type descriptor: \n {}",
                    otherwise
                )
                .into())
            }
        },
        acc,
    ))
}

fn quote_array(
    name: &str,
    mut array_of: Vec<serde_json::Value>,
    acc: proc_macro2::TokenStream,
) -> GenericResult<(proc_macro2::TokenStream, proc_macro2::TokenStream)> {
    let (val, acc) = quote_value(
        name,
        array_of.pop().ok_or(<Box<dyn std::error::Error>>::from(
            String::from("Cannot determine type of empty array"),
        ))?,
        acc,
    )?;
    Ok((quote::quote!(Vec<#val>), acc))
}

fn quote_object(
    name: &str,
    val: serde_json::Map<String, serde_json::Value>,
    acc: proc_macro2::TokenStream,
) -> GenericResult<(proc_macro2::TokenStream, proc_macro2::TokenStream)> {
    let ident = proc_macro2::Ident::new(name, proc_macro2::Span::call_site());
    let (special_case, acc) = typegen(val, name, acc)?;
    if let Some(special_case) = special_case {
        match special_case {
            special_cases::Case::FourXs => Ok((
                quote::quote!(std::collections::HashMap<String, #ident>),
                acc,
            )),
        }
    } else {
        Ok((quote::quote!(#ident), acc))
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
        fn quote_value_string() {
            let quoted_string = quote_value(
                "some_field",
                serde_json::json!("String"),
                proc_macro2::TokenStream::new(),
            );
            assert_eq!(
                quote::quote!(String).to_string(),
                quoted_string.unwrap().0.to_string(),
            );
        }
        #[test]
        fn quote_value_number() {
            let quoted_number = quote_value(
                "some_field",
                serde_json::json!("Decimal"),
                proc_macro2::TokenStream::new(),
            );
            assert_eq!(
                quote::quote!(rust_decimal::Decimal).to_string(),
                quoted_number.unwrap().0.to_string(),
            );
        }
        #[test]
        fn quote_value_bool() {
            let quoted_bool = quote_value(
                "some_field",
                serde_json::json!("bool"),
                proc_macro2::TokenStream::new(),
            );
            assert_eq!(
                quote::quote!(bool).to_string(),
                quoted_bool.unwrap().0.to_string(),
            );
        }
        #[test]
        fn quote_value_optional_string() {
            let quoted_string = quote_value(
                "some_field",
                serde_json::json!("Option<String>"),
                proc_macro2::TokenStream::new(),
            );
            assert_eq!(
                quote::quote!(Option<String>).to_string(),
                quoted_string.unwrap().0.to_string(),
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
            let output =
                process_response(getinfo_path, proc_macro2::TokenStream::new());
            assert_eq!(output.to_string(), test_consts::GETINFO_RESPONSE);
        }
        #[test]
        fn quote_object_simple_unnested() {
            let quoted_object = quote_value(
                "somefield",
                serde_json::json!(
                    {
                        "inner_a": "String",
                        "inner_b": "bool",
                        "inner_c": "Decimal",
                    }
                ),
                proc_macro2::TokenStream::new(),
            )
            .unwrap();
            assert_eq!(
                quote::quote!(somefield).to_string(),
                quoted_object.0.to_string(),
            );
            assert_eq!(
                quoted_object.1.to_string(),
                test_consts::SIMPLE_UNNESTED_RESPONSE,
            );
        }
    }
}

#[cfg(test)]
mod test_consts {
    pub(super) const GETINFO_RESPONSE: &str = "# [derive (Debug , serde :: \
    Deserialize , serde :: Serialize)] pub struct Getinfo { pub balance : \
    rust_decimal :: Decimal , pub blocks : rust_decimal :: Decimal , pub \
    connections : rust_decimal :: Decimal , pub difficulty : rust_decimal :: \
    Decimal , pub errors : String , pub keypoololdest : rust_decimal :: \
    Decimal , pub keypoolsize : rust_decimal :: Decimal , pub paytxfee : \
    rust_decimal :: Decimal , pub protocolversion : rust_decimal :: Decimal , \
    pub proxy : Option < String > , pub relayfee : rust_decimal :: Decimal , \
    pub testnet : bool , pub timeoffset : rust_decimal :: Decimal , pub \
    unlocked_until : rust_decimal :: Decimal , pub version : rust_decimal :: \
    Decimal , pub walletversion : rust_decimal :: Decimal , }";
    pub(super) const SIMPLE_UNNESTED_RESPONSE: &str = "# [derive (Debug , \
    serde :: Deserialize , serde :: Serialize)] pub struct somefield { pub \
    inner_a : String , pub inner_b : bool , pub inner_c : rust_decimal :: \
    Decimal , }";
}
