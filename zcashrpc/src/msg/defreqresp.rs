#[macro_export]
macro_rules! define_request_response {
    {
        $(
            $reqname:ident {
                Request: {
                    $(
                        $fname:ident : $ftype:ty,
                    ),*
                }
                Response: $respbody:tt
            }
        ),*
    } => {
        $(
            pub mod $reqname {
                use serde::{Deserialize, Serialize};

                #[derive(Serialize, Deserialize, Debug)]
                pub struct Request {
                    $(
                        $fname : $ftype
                    ),*
                }

                #[derive(Serialize, Deserialize, Debug)]
                pub struct Response $respbody

                impl crate::msg::Request for Request {
                    type Response = Response;

                    fn name() -> &'static str { stringify!($reqname) }

                    fn params(&self) -> Vec<serde_json::Value> {
                        vec![
                            $( serde_json::to_value( $fname ) ),*
                        ]
                    }
                }
            }
        )*
    }
}
