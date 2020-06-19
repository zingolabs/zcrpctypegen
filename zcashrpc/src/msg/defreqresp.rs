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
                pub struct Request $reqbody

                pub struct Response $respbody

                impl crate::msg::Request for Request {
                    type Response = Response;
                    fn name() -> &'static str { stringify!($reqname) }
                }
            }
        )*
    }
}
