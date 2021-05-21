use super::*;

use futures::executor::block_on;
use futures::io::Cursor;
use ipnetwork::IpNetwork;

const IPC_GET_TESTDATA1: &str = "private_key=e84b5a6d2717c1003a13b431570353dbaca9146cf150c5f8575680feba52027a
listen_port=12912
public_key=b85996fecc9c7f1fc6d2572a76eda11d59bcd20be8e543b15ce4bd85a8e75a33
preshared_key=188515093e952f5f22e865cef3012e72f8b5f0b598ac0309d5dacce3b70fcf52
allowed_ip=192.168.4.4/32
endpoint=[abcd:23::33%2]:51820
public_key=58402e695ba1772b1cc9309755f043251ea77fdcf10fbe63989ceb7e19321376
tx_bytes=38333
rx_bytes=2224
allowed_ip=192.168.4.6/32
persistent_keepalive_interval=111
endpoint=182.122.22.19:3233
public_key=662e14fd594556f522604703340351258903b64f35553763f19426ab2a515c58
endpoint=5.152.198.39:51820
allowed_ip=192.168.4.10/32
allowed_ip=192.168.4.11/32
tx_bytes=1212111
rx_bytes=1929999999
protocol_version=1
errno=0

";

fn cidr(a: u8, b: u8, c: u8, d: u8, mask: u8) -> IpNetwork {
    IpNetwork::new([a, b, c, d].into(), mask).unwrap()
}

#[test]
fn parse_ipc_get() {
    block_on(async {
        let mut stream = Cursor::new(IPC_GET_TESTDATA1.as_bytes());

        let device = read_device_info(&mut stream, "test").await.unwrap();

        assert_eq!(device.private_key().unwrap().to_hex(), "e84b5a6d2717c1003a13b431570353dbaca9146cf150c5f8575680feba52027a");
        assert_eq!(device.listen_port(), 12912);
        assert_eq!(device.peers()[0].public_key().to_hex(), "b85996fecc9c7f1fc6d2572a76eda11d59bcd20be8e543b15ce4bd85a8e75a33");
        assert_eq!(device.peers()[0].preshared_key().unwrap().to_hex(), "188515093e952f5f22e865cef3012e72f8b5f0b598ac0309d5dacce3b70fcf52");
        assert_eq!(device.peers()[0].allow_ips(), &[cidr(192, 168, 4, 4, 32)]);
        assert_eq!(device.peers()[0].endpoint(), "[abcd:23::33%2]:51820".parse().unwrap());

        assert_eq!(device.peers()[1].public_key().to_hex(), "58402e695ba1772b1cc9309755f043251ea77fdcf10fbe63989ceb7e19321376");
        assert_eq!(device.peers()[1].tx_bytes(), 38333);
        assert_eq!(device.peers()[1].rx_bytes(), 2224);
        assert_eq!(device.peers()[1].allow_ips(), &[cidr(192, 168, 4, 6, 32)]);
        assert_eq!(device.peers()[1].persistent_keepalive(), 111);
        assert_eq!(device.peers()[1].endpoint(), "182.122.22.19:3233".parse().unwrap());

        assert_eq!(device.peers()[2].public_key().to_hex(), "662e14fd594556f522604703340351258903b64f35553763f19426ab2a515c58");
        assert_eq!(device.peers()[2].tx_bytes(), 1212111);
        assert_eq!(device.peers()[2].rx_bytes(), 1929999999);
        assert_eq!(device.peers()[2].allow_ips(), &[cidr(192, 168, 4, 10, 32), cidr(192, 168, 4, 11, 32)]);
        assert_eq!(device.peers()[2].endpoint(), "5.152.198.39:51820".parse().unwrap());
    })
}
