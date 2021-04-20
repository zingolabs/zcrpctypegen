fn main() {
    for i in std::env::vars_os() {
        dbg!(&i.0);
    }
    let outdir = &std::env::var("OUT_DIR").unwrap();
    let clone_target = format!("{}/zcashrpc-api", &outdir);
    std::process::Command::new("git")
        .arg("clone")
        .arg("https://github.com/zingolabs/zcashrpc-api.git")
        .arg(&clone_target)
        .output()
        .expect("clone fail");
    std::process::Command::new("mv")
        .arg(format!("{}/src/lib.rs", &clone_target))
        .arg(format!("{}/zcashrpc-api-lib.rs", &outdir))
        .output()
        .expect("mv fail");
    std::process::Command::new("rm")
        .arg("-rf")
        .arg(&clone_target)
        .output()
        .expect("mv fail");
}
