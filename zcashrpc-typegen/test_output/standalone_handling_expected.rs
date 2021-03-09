//procedurally generated response types, note that zcashrpc-typegen
//is in early alpha, and output is subject to change at any time.
#[derive(Debug, serde :: Deserialize, serde :: Serialize)]
pub struct Deltas {
    pub address: String,
    pub height: rust_decimal::Decimal,
    pub index: rust_decimal::Decimal,
    pub satoshis: rust_decimal::Decimal,
    pub txid: String,
}
#[derive(Debug, serde :: Deserialize, serde :: Serialize)]
pub struct End {
    pub hash: String,
    pub height: rust_decimal::Decimal,
}
#[derive(Debug, serde :: Deserialize, serde :: Serialize)]
pub struct Start {
    pub hash: String,
    pub height: rust_decimal::Decimal,
}
#[derive(Debug, serde :: Deserialize, serde :: Serialize)]
pub enum GetaddressdeltasResponse {
    ChainInfoFalse(Vec<Deltas>),
    ChainInfoTrue {
        pub deltas: Vec<Deltas>,
        pub end: End,
        pub start: Start,
    },
}
