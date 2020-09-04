use abscissa_core::Command;
#[derive(Command, Debug, abscissa_core::Options)]
pub struct GetBlockchainInfoCmd {}

use crate::zero_arg_run_impl;

zero_arg_run_impl!(GetBlockchainInfoCmd, getblockchaininfo);
