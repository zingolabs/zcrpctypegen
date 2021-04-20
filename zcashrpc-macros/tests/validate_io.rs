#[test]
fn check_api_existence() {
    let pathstr = &format!(
        "{}/zcashrpc-api/src/lib.rs",
        &std::env::var("LD_LIBRARY_PATH").unwrap()
    );
    dbg!(pathstr);
    /*for i in std::env::vars_os() {
        dbg!(&i);
    }*/
}
