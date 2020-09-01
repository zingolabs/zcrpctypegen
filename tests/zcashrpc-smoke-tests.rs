#[derive(std::fmt::Debug)]
struct TestsFailed;

#[tokio::test]
async fn main() -> Result<(), TestsFailed> {
    let mut runner = Runner::new();
    macro_rules! run_rpc_test {
        ($x:ident) => {
            runner.run(stringify!($x), || make_client().$x()).await
        };
    }

    run_rpc_test!(getinfo);
    run_rpc_test!(getblockchaininfo);
    run_rpc_test!(z_getnewaddress);

    runner.finish()
}

struct Runner {
    tests: u64,
    failures: u64,
}

impl Runner {
    fn new() -> Runner {
        Runner {
            tests: 0,
            failures: 0,
        }
    }

    async fn run<F, Fut, R, E>(&mut self, name: &str, test: F)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<R, E>>,
        R: std::fmt::Debug,
        E: std::fmt::Debug,
    {
        self.tests += 1;
        println!("=== smoke test {}... ", name);
        match test().await {
            Ok(r) => println!("ok.\n{:#?}\n", r),
            Err(e) => {
                self.failures += 1;
                println!("FAIL:\n  {:#?}\n", e);
            }
        }
    }

    fn finish(self) -> Result<(), TestsFailed> {
        println!("=== {} tests with {} failures.", self.tests, self.failures);
        if self.failures == 0 {
            Ok(())
        } else {
            Err(TestsFailed)
        }
    }
}

fn make_client() -> zcashrpc::Client {
    let host = std::env::var("ZCASHRPC_TEST_HOST")
        .unwrap_or(String::from("127.0.0.1:18232".to_string()));
    let auth = std::env::var("ZCASHRPC_TEST_AUTH").unwrap_or_else(|_| {
        let cookie_path = match std::env::var("REGTEST") {
            Ok(_) => "~/.zcash/regtest/.cookie",
            Err(_) => "~/.zcash/.cookie",
        };
        let mut cookie_file = std::fs::File::open(cookie_path)
            .expect(&format!("no cookie found in {}", cookie_path));
        let mut cookie_string = String::new();
        use std::io::Read;
        cookie_file
            .read_to_string(&mut cookie_string)
            .expect("Failed to read cookie");
        cookie_string
    });
    zcashrpc::Client::new(host, auth)
}
