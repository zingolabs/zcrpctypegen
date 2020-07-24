macro_rules! rpc_call {
    ( $self:ident . $rpcname:ident ( $( $arg:expr ),* ) ) => {
        {
            let args = vec![
                $(
                    serde_json::to_value($arg)?
                ),*
            ];

            $self.make_request(stringify!($rpcname), args)
        }
    }
}
