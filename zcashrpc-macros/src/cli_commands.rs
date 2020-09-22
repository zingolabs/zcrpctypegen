pub fn make_command(
    input: (proc_macro2::Ident, proc_macro2::Group),
) -> proc_macro2::TokenStream {
    let (call_ident, command_ident, typed_arg_stream, untyped_arg_stream) =
        crate::utils::format_input("Cmd", input);
    let typed_arg_vec: proc_macro2::TokenStream =
        if !typed_arg_stream.is_empty() {
            group_up(typed_arg_stream)
                .into_iter()
                .map(|ts| {
                    quote::quote!(
                        #[options(free)]
                        #ts,
                    )
                })
                .collect()
        } else {
            typed_arg_stream
        };
    let untyped_arg_vec: Vec<proc_macro2::TokenStream> =
        untyped_arg_stream.into_iter().map(|tt| tt.into()).collect();
    quote::quote!(
        ///Macro-generated rpc method
        #[derive(Command, Debug, abscissa_core::Options)]
        pub struct #command_ident {

            #typed_arg_vec

            #[options(help = "command-specific help")]
            help: bool,
        }

        impl Runnable for #command_ident {
            fn run(&self) {
                abscissa_tokio::run(&crate::application::APPLICATION, async {
                    let response =
                        zcashrpc::client::utils::make_client(true).#call_ident(
                            #(self.#untyped_arg_vec),*
                        );
                    println!("Help flag: {:?}", self.help);
                    println!("{:?}", response.await);
                }).unwrap();
            }
        }
    )
    .into()
}

fn group_up(args: proc_macro2::TokenStream) -> Vec<proc_macro2::TokenStream> {
    args.into_iter()
        .collect::<Vec<proc_macro2::TokenTree>>()
        .split(|tt| {
            if let proc_macro2::TokenTree::Punct(p) = tt {
                p.as_char() == ','
            } else {
                false
            }
        })
        .map(|s| {
            Vec::from(s)
                .into_iter()
                .collect::<proc_macro2::TokenStream>()
        })
        .collect()
}
