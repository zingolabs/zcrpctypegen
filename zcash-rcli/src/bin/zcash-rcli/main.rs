//! Main entry point for ZcashRcli

#![deny(warnings, missing_docs, trivial_casts, unused_qualifications)]
#![forbid(unsafe_code)]

use zcash_rcli::application::APPLICATION;

/// Boot ZcashRcli
fn main() {
    abscissa_core::boot(&APPLICATION);
}
