macro_rules! make_tests {
    ($($test_name:ident),+) => {
        $(
            #[test]
            fn $test_name() {
                call_test(stringify!($test_name));
            }
        )+
    }
}

mod integration {
    make_tests!(
        test_basic_struct,
        test_quizface_output,
        test_terminal_alias,
        test_vec_terminal,
        test_vec_struct,
        test_deduplication
    );
    const TYPEGEN_VERSION: &'static str = env!("CARGO_PKG_VERSION");

    fn call_test(test_name: &str) {
        let test_dir_name = format!("{}_{}", test_name, TYPEGEN_VERSION);
        let output = std::process::Command::new("cargo")
            .args(&["run", &format!("./tests/data/input/{}", &test_dir_name)])
            .output()
            .expect("cargo run failed");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );

        let expected = std::fs::read_to_string(format!(
            "./tests/data/expected/{}.rs",
            test_name
        ));
        let observed = std::fs::read_to_string(format!(
            "./output/{}/rpc_response_types.rs",
            test_dir_name
        ));
        let expected = expected.unwrap();
        let observed = observed.unwrap();
        assert_eq!(
            expected, observed,
            "\n===Custom Format Follows===\nEXPECTED:\n{}\nOBSERVED:\n{}",
            expected, observed
        );
        std::fs::remove_dir_all(format!("./output/{}", test_dir_name)).unwrap();
    }
}
