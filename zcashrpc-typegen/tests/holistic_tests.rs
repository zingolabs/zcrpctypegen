#[test]
fn basic_struct() {
    call_test("basic_struct");
}

#[test]
fn quizface_getinfo_getblockchaininfo() {
    call_test("quizface_output");
}

#[test]
fn standalone_handling() {
    call_test("standalone_handling");
}

#[test]
fn simple_terminal_aliases() {
    call_test("terminal_alias");
}

#[test]
fn alias_vec_of_terminal() {
    call_test("vec_terminal");
}

#[test]
fn alias_vec_of_struct() {
    call_test("vec_struct");
}

#[test]
fn code_dedup() {
    call_test("deduplication");
}

fn call_test(test_name: &str) {
    let output = std::process::Command::new("cargo")
        .args(&[
            "run",
            &format!("./test_data/{}", test_name),
            &format!("test_output/{}.rs", test_name),
        ])
        .output()
        .expect("cargo run failed");
    dbg!(&output);
    assert!(output.status.success());

    let output =
        std::fs::read_to_string(format!("./test_output/{}.rs", test_name));
    let expected = std::fs::read_to_string(format!(
        "./test_output/{}_expected.rs",
        test_name
    ));
    assert_eq!(output.unwrap(), expected.unwrap());
    std::fs::remove_file(format!("./test_output/{}.rs", test_name)).unwrap();
}
