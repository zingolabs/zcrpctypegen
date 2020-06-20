#[macro_export]
macro_rules! define_request_response {
    {
        $(
            $reqname:ident {
                Request: $reqbody:tt,
                Response: $respbody:tt
            }
        ),*
    } => {
        $(
            pub mod $reqname {
                use serde::{Deserialize, Serialize};

                #[derive(Serialize, Deserialize, Debug)]
                pub struct Request $reqbody

                #[derive(Serialize, Deserialize, Debug)]
                pub struct Response $respbody

                impl crate::msg::Request for Request {
                    type Response = Response;
                    fn name() -> &'static str { stringify!($reqname) }
                }

                impl From<&Request> for reqwest::Body {
                    fn from(r: &Request) -> reqwest::Body {
                        reqwest::Body::from(serde_json::to_string(r).unwrap())
                    }
                }
            }
        )*
    }
}
