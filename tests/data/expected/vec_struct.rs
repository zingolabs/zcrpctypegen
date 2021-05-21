//procedurally generated response types, note that zcashrpc-typegen
//is in early alpha, and output is subject to change at any time.
pub mod testdatavec {
    #[derive(Debug, serde :: Deserialize, serde :: Serialize)]
    pub struct TestdatavecArguments;
    #[derive(Debug, serde :: Deserialize, serde :: Serialize)]
    pub struct TestdatavecElement {
        pub fielda: String,
        pub fieldb: rust_decimal::Decimal,
    }
    pub type TestdatavecResponse = Vec<TestdatavecElement>;
}
