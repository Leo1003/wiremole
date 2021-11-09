use std::io::{Stdin, stdin};
use std::process::exit;

use clap::{AppSettings, Parser};
use wirectl::WireCtlError;
use wirectl::interface::WgInterface;
use wirectl::types::{PresharedKey, PrivateKey, PublicKey, WG_KEY_BASE64_LEN};
use zeroize::Zeroizing;

#[derive(Debug, Parser)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommands,
}

#[derive(Debug, Parser)]
enum SubCommands {
    /// List the current interfaces
    List,
    /// Shows the current configuration and device information
    Show(ShowCmd),
    /// Change the current configuration, add peers, remove peers, or change peers
    Set(SetCmd),
    /// Generates a new private key and writes it to stdout
    Genkey,
    /// Generates a new preshared key and writes it to stdout
    Genpsk,
    /// Reads a private key from stdin and writes a public key to stdout
    Pubkey,
}

#[derive(Debug, Parser)]
struct ShowCmd {
    interface: Option<String>,
}

#[derive(Debug, Parser)]
struct SetCmd {
    interface: String,
}

fn main() {
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommands::List => todo!(),
        SubCommands::Show(opt) => todo!(),
        SubCommands::Set(opt) => todo!(),
        SubCommands::Genkey => cmd_genkey(),
        SubCommands::Genpsk => cmd_genpsk(),
        SubCommands::Pubkey => if let Err(e) = cmd_pubkey() {
            eprintln!("{}", e);
            exit(1);
        },
    }
}

fn cmd_genkey() {
    let mut rng = rand::thread_rng();
    let key = PrivateKey::generate(&mut rng);
    println!("{}", key.to_base64());
}

fn cmd_genpsk() {
    let mut rng = rand::thread_rng();
    let key = PresharedKey::generate(&mut rng);
    println!("{}", key.to_base64());
}

fn cmd_pubkey() -> Result<(), WireCtlError> {
    let mut keystr = Zeroizing::new(String::new());
    stdin().read_line(&mut keystr).unwrap();
    let key = PrivateKey::from_base64(keystr.trim())?;
    println!("{}", key.public_key().to_base64());
    Ok(())
}
