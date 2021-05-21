//procedurally generated response types, note that zcashrpc-typegen
//is in early alpha, and output is subject to change at any time.
pub mod boolAlias {
    #[derive(Debug, serde :: Deserialize, serde :: Serialize)]
    pub struct BoolAliasArguments;
    pub type BoolAliasResponse = bool;
}
pub mod numberAlias {
    #[derive(Debug, serde :: Deserialize, serde :: Serialize)]
    pub struct NumberAliasArguments;
    pub type NumberAliasResponse = rust_decimal::Decimal;
}
pub mod stringAlias {
    #[derive(Debug, serde :: Deserialize, serde :: Serialize)]
    pub struct StringAliasArguments;
    pub type StringAliasResponse = String;
}
