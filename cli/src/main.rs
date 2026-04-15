//! `horus` — thin CLI over the gateway runtime.
//!
//! Subcommands (WIP):
//!   horus gateway   --listing <PUBKEY> --key <PATH>     run a query gateway
//!   horus listing   list|update|delist                  manage a listing
//!   horus claim     --escrow <PUBKEY>                    post a settlement claim
//!
//! Flags and wiring are deliberately minimal while the protocol stabilizes.

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("gateway") => {
            // The `--features zk-experimental` build enables ZK paths; the
            // default build returns FeatureDisabled if a ZK route is hit.
            #[cfg(feature = "zk-experimental")]
            eprintln!("horus gateway: zk-experimental ENABLED (unaudited)");
            #[cfg(not(feature = "zk-experimental"))]
            eprintln!("horus gateway: zk disabled (build with --features zk-experimental to enable)");
            eprintln!("not yet wired to an RPC endpoint; see RFC-0001 §End-to-end flow");
            ExitCode::SUCCESS
        }
        Some("listing") => {
            eprintln!("horus listing: see RFC-0009 for the record layout");
            ExitCode::SUCCESS
        }
        Some("claim") => {
            eprintln!("horus claim: settlement claim path, see RFC-0007");
            ExitCode::SUCCESS
        }
        Some("--version") | Some("-V") => {
            println!("horus {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("usage: horus <gateway|listing|claim> [flags]");
            ExitCode::from(2)
        }
    }
}
