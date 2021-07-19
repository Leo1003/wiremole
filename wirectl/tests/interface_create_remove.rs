use futures::executor::block_on;
use rand::prelude::*;
use wirectl::interface::WgInterface;

#[test]
#[ignore = "test must be run as root"]
fn interface_create_remove() {
    block_on(async {
        let mut rng = thread_rng();
        // Interface maximum length is 15 bytes
        let ifname = format!("test_{}", hex::encode(rng.next_u32().to_ne_bytes()));
        println!("Creating interface: {}", ifname);

        let wgif = WgInterface::create_interface(&ifname).await.unwrap();
        println!("Created interface: {}", wgif.ifname());

        wgif.remove_interfaces().await.unwrap();
        println!("Removed interface");
    });
}
