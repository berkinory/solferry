use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "beam",
    version,
    about = "Accountless file transfer between your devices"
)]
pub struct Cli {
    /// Where the device key, settings and paired devices live.
    #[arg(long, global = true, env = "BEAM_DATA_DIR")]
    pub data_dir: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Show this device and its settings.
    Status,
    /// Change the name other devices see.
    Rename { name: String },
    /// Send files or folders to one or more devices.
    Send {
        #[arg(required = true)]
        paths: Vec<PathBuf>,
        /// Device name, id prefix or nearby device name. Repeat to send to several.
        #[arg(long = "to", required = true)]
        targets: Vec<String>,
    },
    /// Stay online and receive files.
    Receive {
        /// Save here instead of the default download folder.
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Accept every offer without asking.
        #[arg(long)]
        yes: bool,
        /// Exit after the first finished transfer.
        #[arg(long)]
        once: bool,
    },
    /// Create a pairing code for another device.
    Pair {
        /// The other device belongs to someone else, so its transfers stay confirmed.
        #[arg(long)]
        contact: bool,
    },
    /// Redeem a pairing code from another device.
    Join { code: String },
    /// List paired devices.
    Devices,
    /// Remove a paired device.
    Forget { device: String },
    /// List devices on the local network.
    Nearby,
    /// Control how relays are used.
    Relay {
        #[command(subcommand)]
        command: RelayCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum RelayCommand {
    /// Show the relay policy and your relays.
    Show,
    /// Set how file data may use relays.
    Policy { policy: PolicyArg },
    /// Use your own relay server.
    Add { url: String },
    /// Stop using a relay server.
    Remove { url: String },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum PolicyArg {
    /// Servers only introduce devices. File data stays direct.
    HandshakeOnly,
    /// File data may pass through relays.
    Always,
    /// No servers. Local network and overlay networks only.
    Off,
}
