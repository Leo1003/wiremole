table! {
    allowed_ips (id) {
        id -> Integer,
        peer_id -> Nullable<Integer>,
        ipaddress -> Varbinary,
        mask -> Unsigned<Tinyint>,
    }
}

table! {
    interfaces (id) {
        id -> Integer,
        devname -> Varchar,
        mtu -> Nullable<Unsigned<Integer>>,
        privkey -> Nullable<Binary>,
        fwmark -> Unsigned<Integer>,
        listen_port -> Unsigned<Smallint>,
    }
}

table! {
    interface_ips (id) {
        id -> Integer,
        interface_id -> Nullable<Integer>,
        ipaddress -> Varbinary,
        mask -> Unsigned<Tinyint>,
    }
}

table! {
    peers (id) {
        id -> Integer,
        interface_id -> Nullable<Integer>,
        pubkey -> Binary,
        preshared_key -> Nullable<Binary>,
        endpoint_ip -> Nullable<Varbinary>,
        endpoint_port -> Nullable<Unsigned<Smallint>>,
        endpoint_flowinfo -> Nullable<Unsigned<Integer>>,
        persistent_keepalive -> Nullable<Unsigned<Smallint>>,
    }
}

joinable!(allowed_ips -> peers (peer_id));
joinable!(interface_ips -> interfaces (interface_id));
joinable!(peers -> interfaces (interface_id));

allow_tables_to_appear_in_same_query!(
    allowed_ips,
    interfaces,
    interface_ips,
    peers,
);
