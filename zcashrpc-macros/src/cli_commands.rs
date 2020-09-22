pub fn make_command(
    input: (proc_macro2::Ident, proc_macro2::Group),
) -> proc_macro2::TokenStream {
    let (call_ident, command_ident, param_stream, arg_id_stream) =
        crate::utils::format_input("Cmd", input);
    let formatted_param_vec: proc_macro2::TokenStream =
        if !param_stream.is_empty() {
            group_up(param_stream)
                .into_iter()
                .map(|ts| {
                    quote::quote!(
                        #[options(free)]
                        #ts,
                    )
                })
                .collect()
        } else {
            param_stream
        };
    let arg_id_vec: Vec<proc_macro2::TokenStream> =
        arg_id_stream.into_iter().map(|tt| tt.into()).collect();
    quote::quote!(
        ///Macro-generated rpc method
        #[derive(Command, Debug, abscissa_core::Options)]
        pub struct #command_ident {

            #formatted_param_vec

            #[options(help = "command-specific help")]
            help: bool,
        }

        impl Runnable for #command_ident {
            fn run(&self) {
                abscissa_tokio::run(&crate::application::APPLICATION, async {
                    let response =
                        zcashrpc::client::utils::make_client(true).#call_ident(
                            #(self.#arg_id_vec),*
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
