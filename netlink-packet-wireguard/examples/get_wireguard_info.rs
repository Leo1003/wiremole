use anyhow::{bail, Error};
use futures::StreamExt;
use futures::{channel::mpsc::UnboundedReceiver, lock::Mutex};
use netlink_packet_core::{NetlinkMessage, NetlinkPayload, NLM_F_DUMP, NLM_F_REQUEST};
use netlink_packet_generic::GenlFamily;
use netlink_packet_generic::{
    ctrl::{nlas::GenlCtrlAttrs, GenlCtrl, GenlCtrlCmd},
    GenlHeader, GenlMessage,
};
use netlink_packet_utils::Emitable;
use netlink_packet_wireguard::nlas::{WgAllowedIpAttrs, WgPeerAttrs};
use netlink_packet_wireguard::{nlas::WgDeviceAttrs, Wireguard, WireguardCmd};
use netlink_proto::sys::protocols::NETLINK_GENERIC;
use netlink_proto::sys::SocketAddr;
use netlink_proto::{Connection, ConnectionHandle};
use once_cell::sync::Lazy;
use std::{
    env::args,
    io,
    sync::{Arc, RwLock},
};

static CACHE: Lazy<Arc<RwLock<u16>>> = Lazy::new(|| Arc::new(RwLock::new(0)));

#[tokio::main]
async fn main() {
    env_logger::init();

    let argv: Vec<String> = args().collect();
    if argv.len() < 2 {
        eprintln!("Usage: get_wireguard_info <ifname>");
        return;
    }

    let mut genlmsg: GenlMessage<Wireguard> = GenlMessage::from_payload(Wireguard {
        cmd: WireguardCmd::GetDevice,
        nlas: vec![WgDeviceAttrs::IfName(argv[1].clone())],
    });
    genlmsg.set_resolved_family_id(resolve_wireguard_id().await.unwrap());
    let mut nlmsg = NetlinkMessage::from(genlmsg);
    nlmsg.header.flags = NLM_F_REQUEST | NLM_F_DUMP;
    nlmsg.finalize();

    let (connection, mut handle, _) = new_wireguard_connection().unwrap();
    tokio::spawn(connection);

    let mut res = handle.request(nlmsg, SocketAddr::new(0, 0)).unwrap();

    while let Some(rx_packet) = res.next().await {
        match rx_packet.payload {
            NetlinkPayload::InnerMessage(genlmsg) => {
                print_wg_payload(genlmsg.payload);
            }
            NetlinkPayload::Error(e) => {
                eprintln!("Error: {:?}", e.to_io());
            }
            _ => (),
        };
    }
}

fn print_wg_payload(wg: Wireguard) {
    for nla in &wg.nlas {
        match nla {
            WgDeviceAttrs::IfIndex(v) => println!("IfIndex: {}", v),
            WgDeviceAttrs::IfName(v) => println!("IfName: {}", v),
            WgDeviceAttrs::PrivateKey(_) => println!("PrivateKey: (hidden)"),
            WgDeviceAttrs::PublicKey(v) => println!("PublicKey: {}", base64::encode(v)),
            WgDeviceAttrs::ListenPort(v) => println!("ListenPort: {}", v),
            WgDeviceAttrs::Fwmark(v) => println!("Fwmark: {}", v),
            WgDeviceAttrs::Peers(nlas) => {
                for peer in nlas {
                    println!("Peer: ");
                    print_wg_peer(peer);
                }
            }
            _ => (),
        }
    }
}

fn print_wg_peer(nlas: &[WgPeerAttrs]) {
    for nla in nlas {
        match nla {
            WgPeerAttrs::PublicKey(v) => println!("  PublicKey: {}", base64::encode(v)),
            WgPeerAttrs::PresharedKey(_) => println!("  PresharedKey: (hidden)"),
            WgPeerAttrs::Endpoint(v) => println!("  Endpoint: {}", v),
            WgPeerAttrs::PersistentKeepalive(v) => println!("  PersistentKeepalive: {}", v),
            WgPeerAttrs::LastHandshake(v) => println!("  LastHandshake: {:?}", v),
            WgPeerAttrs::RxBytes(v) => println!("  RxBytes: {}", v),
            WgPeerAttrs::TxBytes(v) => println!("  TxBytes: {}", v),
            WgPeerAttrs::AllowedIps(nlas) => {
                for ip in nlas {
                    print_wg_allowedip(ip);
                }
            }
            _ => (),
        }
    }
}

fn print_wg_allowedip(nlas: &[WgAllowedIpAttrs]) -> Option<()> {
    let ipaddr = nlas
        .iter()
        .find_map(|nla| {
            if let WgAllowedIpAttrs::IpAddr(ipaddr) = nla {
                Some(*ipaddr)
            } else {
                None
            }
        })?;
    let cidr = nlas
        .iter()
        .find_map(|nla| {
            if let WgAllowedIpAttrs::Cidr(cidr) = nla {
                Some(*cidr)
            } else {
                None
            }
        })?;
    println!("  AllowedIp: {}/{}", ipaddr, cidr);
    Some(())
}

pub fn new_wireguard_connection() -> io::Result<(
    Connection<GenlMessage<Wireguard>>,
    ConnectionHandle<GenlMessage<Wireguard>>,
    UnboundedReceiver<(NetlinkMessage<GenlMessage<Wireguard>>, SocketAddr)>,
)> {
    let (conn, handle, messages) = netlink_proto::new_connection(NETLINK_GENERIC)?;
    Ok((conn, handle, messages))
}

async fn resolve_wireguard_id() -> Result<u16, Error> {
    if *CACHE.read().unwrap() == 0 {
        let mut guard = CACHE.write().unwrap();
        if *guard != 0 {
            // Updated by others first
            return Ok(*guard);
        }

        let mut genlmsg: GenlMessage<GenlCtrl> = GenlMessage::from_payload(GenlCtrl {
            cmd: GenlCtrlCmd::GetFamily,
            nlas: vec![GenlCtrlAttrs::FamilyName(
                Wireguard::family_name().to_owned(),
            )],
        });
        genlmsg.finalize();
        let mut nlmsg = NetlinkMessage::from(genlmsg);
        nlmsg.header.flags = NLM_F_REQUEST;
        nlmsg.finalize();

        let (connection, mut handle, _) = new_nlctrl_connection()?;
        tokio::spawn(connection);

        let mut res = handle.request(nlmsg, SocketAddr::new(0, 0))?;

        while let Some(rx_packet) = res.next().await {
            if let NetlinkPayload::InnerMessage(genlmsg) = rx_packet.payload {
                for nla in genlmsg.payload.nlas {
                    if let GenlCtrlAttrs::FamilyId(family_id) = nla {
                        *guard = family_id;
                        return Ok(*guard);
                    }
                }
            }
        }
        bail!("No response from netlink")
    } else {
        Ok(*CACHE.read().unwrap())
    }
}

pub fn new_nlctrl_connection() -> io::Result<(
    Connection<GenlMessage<GenlCtrl>>,
    ConnectionHandle<GenlMessage<GenlCtrl>>,
    UnboundedReceiver<(NetlinkMessage<GenlMessage<GenlCtrl>>, SocketAddr)>,
)> {
    let (conn, handle, messages) = netlink_proto::new_connection(NETLINK_GENERIC)?;
    Ok((conn, handle, messages))
}
