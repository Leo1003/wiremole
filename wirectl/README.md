# Wirectl
![minimum rustc version](https://img.shields.io/badge/rustc-1.52+-blue.svg)
A library providing anything that you will need to configure Wireguard interfaces.

## Motivation
Wireguard has become a widely-used tunnel solution in these years. Being a fast,
simple, and secure protocol, it became my favorite VPN protocol. However, there
is no well-maintained server to dynamically modify the configurations. 
Thus, this limits the usage of Wireguard in some circumstances.

In order to manage wireguard configurations via REST api and beautiful interface.
I decide to create a web server in Rust. But I found that there is also no 
well-maintained library to control Wireguard in the Rust ecosystem. So, my project
became writing one library and one web server of Wireguard. :/

## Current Status
This library still under development, it lacks of some core features and isn't
properly tested.

## Roadmap
- Support for different implementation
    - [ ] Linux
    - [ ] FreeBSD
    - [ ] OpenBSD
    - [x] Userspace (Unix)
    - [ ] Userspace (Windows)
- Async runtime support
    - [x] Smol
    - [ ] Tokio
    - Be async runtime agnostic if there is a proper way to do that
- Extension features
    - [ ] wg-quick configuration
        - [ ] Parsing
        - [ ] Generating
    - [ ] Embbeding [boringtun](https://github.com/cloudflare/boringtun) library
