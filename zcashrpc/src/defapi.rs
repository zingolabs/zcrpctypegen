macro_rules! def_api_method {
    (
     $reqname:ident ( $( $argname:ident : $argtype:ty ),* ) ->
     $respname:ident { $( $fname:ident : $ftype:ty ),* }
    ) => {
#[derive(Debug, Deserialize, Serialize)]
        pub struct $respname {
            $( $fname : $ftype ),*
        }

        impl Client {
            pub async fn $reqname ( &mut self, $( $argname : $argtype ),* ) -> Result<$respname, Error<$respname>> {
                let args = vec![
                    $(
                            serde_json::to_value($argname)?
                     ),*
                ];

                self.make_request(stringify!($reqname), args).await
            }
        }
    }
}
