#[test]
fn validate_basic_struct_test_output() {
    call_test("basic_struct");
}
fn validate_quizface_output() {
    call_test("quizface_output");
}

fn call_test(test_name: &str) {
    assert!(std::process::Command::new("cargo")
        .args(&[
            "run",
            &format!("./test_data/{}", test_name),
            &format!("test_output/{}.rs", test_name),
        ])
        .output()
        .expect("cargo run failed")
        .status
        .success());

    let output =
        std::fs::read_to_string(format!("./test_output/{}.rs", test_name));
    let expected = std::fs::read_to_string(format!(
        "./test_output/{}_expected.rs",
        test_name
    ));
    assert_eq!(output.unwrap(), expected.unwrap());
    std::fs::remove_file(format!("./test_output/{}.rs", test_name)).unwrap();
}
