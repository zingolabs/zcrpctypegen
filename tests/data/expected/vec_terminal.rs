//procedurally generated response types, note that zcashrpc-typegen
//is in early alpha, and output is subject to change at any time.
pub mod VecOfNumber {
    #[derive(Debug, serde :: Deserialize, serde :: Serialize)]
    pub struct VecOfNumberArguments;
    pub type VecOfNumberResponse = Vec<rust_decimal::Decimal>;
}
pub mod VecOfString {
    #[derive(Debug, serde :: Deserialize, serde :: Serialize)]
    pub struct VecOfStringArguments;
    pub type VecOfStringResponse = Vec<String>;
}
