use clap::Parser;
use smol::block_on;
use std::env;
use std::io::stdin;
use std::process::exit;
use std::time::{SystemTime, UNIX_EPOCH};
use time::OffsetDateTime;
use wirectl::interface::WgInterface;
use wirectl::types::{PresharedKey, PrivateKey, PublicKey, WgDevice, WG_KEY_BASE64_LEN};
use wirectl::WireCtlError;
use zeroize::Zeroizing;

mod args;
use args::*;

fn main() {
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommands::List => {
            if let Err(e) = block_on(cmd_list()) {
                eprintln!("{}", e);
                exit(1);
            }
        }
        SubCommands::Show(opt) => {
            if let Err(e) = block_on(cmd_show(&opt)) {
                eprintln!("{}", e);
                exit(1);
            }
        }
        SubCommands::Set(opt) => todo!(),
        SubCommands::Genkey => cmd_genkey(),
        SubCommands::Genpsk => cmd_genpsk(),
        SubCommands::Pubkey => {
            if let Err(e) = cmd_pubkey() {
                eprintln!("{}", e);
                exit(1);
            }
        }
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

async fn cmd_list() -> Result<(), WireCtlError> {
    let list = WgInterface::list_interfaces().await?;
    for i in &list {
        println!("{}", i);
    }
    Ok(())
}

async fn cmd_show(opt: &ShowCmd) -> Result<(), WireCtlError> {
    if let Some(ifname) = &opt.interface {
        let wgif = WgInterface::get_interface(ifname).await?;
        show_interface(&wgif, opt, false).await?;
    } else {
        let list = WgInterface::get_interfaces().await?;
        for wgif in &list {
            show_interface(wgif, opt, true).await?;
        }
    }
    Ok(())
}

async fn show_interface(
    wgif: &WgInterface,
    opt: &ShowCmd,
    print_ifname: bool,
) -> Result<(), WireCtlError> {
    let config = wgif.get_config().await?;

    if let Some(fields) = &opt.fields {
        dump_interface(&config, fields, print_ifname);
    } else {
        print_interface_pretty(&config);
    }
    Ok(())
}

fn dump_interface(config: &WgDevice, fields: &ShowFields, print_ifname: bool) {
    // nff: non first field
    let mut nff = false;

    if print_ifname {
        print!("{}", config.device_name);
        nff = true;
    }
    if fields.private_key {
        if nff {
            print!("\t");
        }
        if let Some(private_key) = &config.private_key {
            print!("{}", private_key.to_base64());
        } else {
            print!("(none)");
        }
        nff = true;
    }
    if fields.public_key {
        if nff {
            print!("\t");
        }
        if let Some(public_key) = &config.public_key {
            print!("{}", public_key.to_base64());
        } else {
            print!("(none)");
        }
        nff = true;
    }
    if fields.listen_port {
        if nff {
            print!("\t");
        }
        print!("{}", config.listen_port);
        nff = true;
    }
    if fields.fwmark {
        if nff {
            print!("\t");
        }
        if config.has_fwmark() {
            print!("{}", config.fwmark);
        } else {
            print!("off");
        }
        //nff = true;
    }
    println!();

    if fields.contains_peer_fields() {
        // Peers
        for peer in &config.peers {
            if print_ifname {
                print!("{}\t", config.device_name);
            }
            // If any peer-related fields are specified, always print peer public key
            // Including `field.peers`
            print!("{}", peer.public_key.to_base64());

            if fields.preshared_keys {
                if peer.has_preshared_key() {
                    print!("\t{}", peer.preshared_key.to_base64());
                } else {
                    print!("\t(none)");
                }
            }
            if fields.endpoints {
                if peer.has_endpoint() {
                    print!("\t{}", peer.endpoint);
                } else {
                    print!("\t(none)");
                }
            }
            if fields.allowed_ips {
                if peer.allow_ips.is_empty() {
                    print!("\t(none)");
                } else {
                    let ips_str = peer.allow_ips.iter().map(|ips| format!("{}", ips)).fold(
                        String::new(),
                        |mut s, a| {
                            if !s.is_empty() {
                                s.push(',');
                            }
                            s.push_str(&a);
                            s
                        },
                    );
                    print!("\t{}", &ips_str);
                }
            }
            if fields.latest_handshakes {

                print!("\t{}", peer.last_handshake.duration_since(UNIX_EPOCH).unwrap().as_secs());
            }
            if fields.transfer {
                print!("\t{}\t{}", peer.rx_bytes, peer.tx_bytes);
            }
            if fields.persistent_keepalive {
                if peer.has_persistent_keepalive() {
                    print!("\t{}", peer.persistent_keepalive);
                } else {
                    print!("\toff");
                }
            }


            println!();
        }
    }
}

fn print_interface_pretty(config: &WgDevice) {
    println!("interface: {}", config.device_name);
    if let Some(public_key) = &config.public_key {
        println!("  public key: {}", public_key.to_base64());
    }
    if let Some(private_key) = &config.private_key {
        if let Ok("never") = env::var("WG_HIDE_KEYS").as_deref() {
            println!("  private key: {}", private_key.to_base64());
        } else {
            println!("  private key: (hidden)");
        }
    }
    if config.has_listen_port() {
        println!("  listening port: {}", config.listen_port);
    }
    if config.has_fwmark() {
        println!("  public key: {}", config.fwmark);
    }
    println!();

    for peer in &config.peers {
        println!("peer: {}", peer.public_key.to_base64());
        if peer.has_preshared_key() {
            if let Ok("never") = env::var("WG_HIDE_KEYS").as_deref() {
                println!("  preshared key: {}", peer.preshared_key.to_base64());
            } else {
                println!("  preshared key: (hidden)");
            }
        }
        if peer.has_endpoint() {
            println!("  endpoint: {}", peer.endpoint);
        }
        print!("  allowed ips: ");
        if peer.allow_ips.is_empty() {
            println!("(none)");
        } else {
            for ip in &peer.allow_ips {
                print!("{}, ", ip);
            }
            println!();
        }

        if peer.last_handshake != SystemTime::UNIX_EPOCH {
            print!("  latest handshake: ");
            let time = OffsetDateTime::from(peer.last_handshake);
            let dura = OffsetDateTime::now_utc() - time;
            if dura.is_positive() {
                if dura.whole_days() > 0 {
                    print!("{} ", format_pluralize(dura.whole_days(), "day", "days"));
                }
                if dura.whole_hours() > 0 {
                    print!("{} ", format_pluralize(dura.whole_hours(), "hour", "hours"));
                }
                if dura.whole_minutes() > 0 {
                    print!(
                        "{} ",
                        format_pluralize(dura.whole_minutes(), "minute", "minutes")
                    );
                }
                if dura.whole_seconds() > 0 {
                    print!(
                        "{} ",
                        format_pluralize(dura.whole_seconds(), "second", "seconds")
                    );
                }
                println!("ago");
            } else {
                println!("System clock runs backward. Cannot determine actual handshake time!");
            }
        }

        println!(
            "  transfer: {} received, {} sent",
            format_bytes(peer.rx_bytes),
            format_bytes(peer.tx_bytes)
        );
        if peer.has_persistent_keepalive() {
            print!(
                "  persistent keepalive: every {} seconds",
                peer.persistent_keepalive
            );
        }

        println!();
    }
}

fn format_pluralize(cnt: i64, unit: &str, plural: &str) -> String {
    if cnt > 1 {
        format!("{} {}", cnt, plural)
    } else {
        format!("{} {}", cnt, unit)
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.2} KiB", bytes as f64 / 1024_f64)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.2} MiB", bytes as f64 / (1024_u64 * 1024) as f64)
    } else if bytes < 1024 * 1024 * 1024 * 1024 {
        format!("{:.2} GiB", bytes as f64 / (1024_u64 * 1024 * 1024) as f64)
    } else {
        format!(
            "{:.2} TiB",
            bytes as f64 / (1024_u64 * 1024 * 1024 * 1024) as f64
        )
    }
}
