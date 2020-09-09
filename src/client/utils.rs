#[cfg(feature = "cookie-finder")]
pub fn get_cookie(regtest: bool) -> std::io::Result<String> {
    let mut cookie_path = match dirs::home_dir() {
        Some(x) => x,
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "{} {} {}",
                    "Could not find your home directory.",
                    "Please pass the contents of ~/.zcash/.cookie to the",
                    "enviroment variable ZCASH_TEST_AUTH."
                ),
            ))
        }
    };

    cookie_path.push(".zcash");
    if regtest {
        cookie_path.push("regtest");
    }
    cookie_path.push(".cookie");

    let mut cookie_file = std::fs::File::open(cookie_path)?;
    let mut cookie_string = String::new();
    use std::io::Read as _;
    cookie_file.read_to_string(&mut cookie_string)?;
    Ok(cookie_string)
}

pub fn get_zcashd_port() -> String {
    //This could theoretically be expanded to do some sort of
    //automatic port lookup
    String::from("127.0.0.1:18232")
}
