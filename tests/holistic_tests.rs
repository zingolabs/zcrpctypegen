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

make_tests!(
    basic_struct,
    quizface_output,
    terminal_alias,
    vec_terminal,
    vec_struct,
    deduplication
);

fn call_test(test_name: &str) {
    let output = std::process::Command::new("cargo")
        .args(&[
            "run",
            &format!("./test_data/{}", test_name),
            &format!("test_output/{}.rs", test_name),
        ])
        .output()
        .expect("cargo run failed");
    assert!(output.status.success(), "{:#?}", output);

    let expected = std::fs::read_to_string(format!(
        "./test_output/{}_expected.rs",
        test_name
    ));
    let observed =
        std::fs::read_to_string(format!("./test_output/{}.rs", test_name));
    let expected = expected.unwrap();
    let observed = observed.unwrap();
    assert_eq!(
        expected, observed,
        "\n===Custom Format Follows===\nEXPECTED:\n{}\nOBSERVED:\n{}",
        expected, observed
    );
    std::fs::remove_file(format!("./test_output/{}.rs", test_name)).unwrap();
}
