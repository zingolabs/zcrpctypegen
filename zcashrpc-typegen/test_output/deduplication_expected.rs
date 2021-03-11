//procedurally generated response types, note that zcashrpc-typegen
//is in early alpha, and output is subject to change at any time.
pub mod z_gettreestate {
    #[derive(Debug, serde :: Deserialize, serde :: Serialize)]
    pub struct Commitments {
        pub finalRoot: String,
        pub finalState: String,
    }
    #[derive(Debug, serde :: Deserialize, serde :: Serialize)]
    pub struct Sapling {
        pub commitments: Commitments,
        pub skipHash: String,
    }
    #[derive(Debug, serde :: Deserialize, serde :: Serialize)]
    pub struct Sprout {
        pub commitments: Commitments,
        pub skipHash: String,
    }
    #[derive(Debug, serde :: Deserialize, serde :: Serialize)]
    pub struct Z_gettreestateResponse {
        pub hash: String,
        pub height: rust_decimal::Decimal,
        pub sapling: Sapling,
        pub sprout: Sprout,
    }
}
