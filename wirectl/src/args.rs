use clap::{Arg, Args, Error, ErrorKind, FromArgMatches, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Opts {
    #[clap(subcommand)]
    pub subcmd: SubCommands,
}

#[derive(Debug, Subcommand)]
pub enum SubCommands {
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ShowCmd {
    /// Interface name to show, or specify "all" to print every interface found
    pub interface: Option<String>,
    pub fields: Option<ShowFields>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ShowFields {
    pub public_key: bool,
    pub private_key: bool,
    pub listen_port: bool,
    pub fwmark: bool,
    pub peers: bool,
    pub preshared_keys: bool,
    pub endpoints: bool,
    pub allowed_ips: bool,
    pub latest_handshakes: bool,
    pub persistent_keepalive: bool,
    pub transfer: bool,
}

impl ShowFields {
    fn enable_all_fields(&mut self) {
        self.public_key = true;
        self.private_key = true;
        self.listen_port = true;
        self.fwmark = true;
        self.peers = true;
        self.preshared_keys = true;
        self.endpoints = true;
        self.allowed_ips = true;
        self.latest_handshakes = true;
        self.persistent_keepalive = true;
        self.transfer = true;
    }
}

impl Args for ShowCmd {
    fn augment_args(app: clap::App<'_>) -> clap::App<'_> {
        app.arg(
            Arg::new("interface")
                .help("Interface name to show, or specify \"all\" to print every interface found")
                .index(1)
                .default_value("all"),
        )
        .arg(
            Arg::new("fields")
                .help("Specifying the fields to print")
                .index(2)
                .takes_value(true)
                .multiple_values(true)
                .possible_values(&[
                    "public-key",
                    "private-key",
                    "listen-port",
                    "fwmark",
                    "peers",
                    "preshared-keys",
                    "endpoints",
                    "allowed-ips",
                    "latest-handshakes",
                    "persistent-keepalive",
                    "transfer",
                    "dump",
                ]),
        )
    }

    fn augment_args_for_update(app: clap::App<'_>) -> clap::App<'_> {
        Self::augment_args(app)
    }
}

impl FromArgMatches for ShowCmd {
    fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, Error> {
        let mut arg = Self::default();
        arg.update_from_arg_matches(matches)?;
        Ok(arg)
    }

    fn update_from_arg_matches(&mut self, matches: &clap::ArgMatches) -> Result<(), Error> {
        if let Some(ifname) = matches.value_of("interface") {
            self.interface = if ifname != "all" {
                Some(ifname.to_owned())
            } else {
                None
            };
        } else {
            self.interface = None;
        }

        if let Some(match_fields) = matches.values_of("fields") {
            let mut fields = ShowFields::default();

            for f in match_fields {
                match f {
                    "public-key" => fields.public_key = true,
                    "private-key" => fields.private_key = true,
                    "listen-port" => fields.listen_port = true,
                    "fwmark" => fields.fwmark = true,
                    "peers" => fields.peers = true,
                    "preshared-keys" => fields.preshared_keys = true,
                    "endpoints" => fields.endpoints = true,
                    "allowed-ips" => fields.allowed_ips = true,
                    "latest-handshakes" => fields.latest_handshakes = true,
                    "persistent-keepalive" => fields.persistent_keepalive = true,
                    "transfer" => fields.transfer = true,
                    "dump" => fields.enable_all_fields(),
                    _ => return Err(Error::raw(ErrorKind::InvalidValue, "Invalid field name")),
                }
            }
            self.fields = Some(fields);
        } else {
            self.fields = None;
        }

        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct SetCmd {
    interface: String,
}
