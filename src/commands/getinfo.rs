use abscissa_core::{Command, FrameworkError, Options, Runnable};
use zcashrpc::client::utils;

/// The `Options` proc macro generates an option parser based on the struct
/// definition, and is defined in the `gumdrop` crate. See their documentation
/// for a more comprehensive example:
///
/// <https://docs.rs/gumdrop/>
#[derive(Command, Debug, Options)]
pub struct GetInfoCmd {
    /// To whom are we saying hello?
    #[options(free)]
    recipient: Vec<String>,
}

impl Runnable for GetInfoCmd {
    /// Start the application.
    fn run(&self) {
        abscissa_tokio::run(&crate::application::APPLICATION, async {
            let response = zcashrpc::Client::new(
                utils::get_zcashd_port(),
                utils::get_cookie(true).unwrap(),
            )
            .getinfo();
            println!("{:?}", response.await);
        });
    }
}

impl abscissa_core::config::Override<crate::config::ZcashRcliConfig>
    for GetInfoCmd
{
    // Process the given command line options, overriding settings from
    // a configuration file using explicit flags taken from command-line
    // arguments.
    fn override_config(
        &self,
        mut config: crate::config::ZcashRcliConfig,
    ) -> Result<crate::config::ZcashRcliConfig, FrameworkError> {
        if !self.recipient.is_empty() {
            config.hello.recipient = self.recipient.join(" ");
        }

        Ok(config)
    }
}
