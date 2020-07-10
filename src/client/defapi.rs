macro_rules! def_api_method {
    (
     $reqname:ident ( $( $argname:ident : $argtype:ty ),* ) ->
     $respname:ident { $( $fname:ident : $ftype:ty ),* }
    ) => {
        #[derive(Debug, Deserialize, Serialize)]
        #[allow(non_snake_case)]
        pub struct $respname {
            $( pub $fname : $ftype ),*
        }

        impl Client {
            pub async fn $reqname ( &mut self, $( $argname : $argtype ),* ) -> ResponseResult<$respname> {
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
