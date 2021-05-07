# Wiremole
![minimum rustc version](https://img.shields.io/badge/rustc-1.52+-blue.svg)
A Wireguard web interface aims to provide intuitive user interface and a ease to use REST API allowing to dynamically control clients.

## Project Status
This repository is still under development, currently unusable.

## Roadmap
- [ ] Manage wireguard interface
    - Backend
        - [ ] Kernelspace implementation
            - [ ] Linux
            - [ ] OpenBSD/FreeBSD
        - [ ] Userspace implementation
        - [ ] Embedding [boringtun](https://github.com/cloudflare/boringtun) library
    - [ ] Admin authentication by specified wireguard IP
    - [ ] Import existing configuration
    - [ ] Export configuration
    - [ ] Run commands when the interface is up/down
    - [ ] Control routes & NAT
    - Other platform support
        - [ ] Windows
        - [ ] FreeBSD
        - [ ] MacOS
- [ ] REST api
    - [ ] Auth by access token
- [ ] Web UI frontend
    - Powered by [yew](https://github.com/yewstack/yew)
    - [ ] Show realtime statistics
    - [ ] Generate private key on client side
    - [ ] Generate QR code for mobile device
    - [ ] Download client configuration file

