use abscissa_core::Command;
#[derive(Command, Debug, abscissa_core::Options)]
pub struct GetBlockchainInfoCmd {}

impl abscissa_core::Runnable for GetBlockchainInfoCmd {
    fn run(&self) {
        println!("Placeholder lol")
    }
}
