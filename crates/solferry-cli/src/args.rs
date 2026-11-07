use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "solferry", version, about = "Accountless file transfer between your devices")]
pub struct Cli {
    /// Where the device key, settings and paired devices live.
    #[arg(long, global = true, env = "SOLFERRY_DATA_DIR")]
    pub data_dir: Option<PathBuf>,
    /// Print transfer events as JSON lines, for scripts and headless use.
    #[arg(long, global = true)]
    pub json: bool,
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
        /// Pair with this nearby device by comparing a code, instead of showing a pairing code.
        #[arg(long)]
        with: Option<String>,
        /// Ask before taking files from this device instead of receiving them right away.
        #[arg(long)]
        ask: bool,
    },
    /// Redeem a pairing code from another device.
    Join {
        code: String,
        /// Ask before taking files from this device instead of receiving them right away.
        #[arg(long)]
        ask: bool,
    },
    /// List paired devices.
    Devices,
    /// Remove a paired device.
    Forget { device: String },
    /// Choose whether a paired device needs your approval before sending files.
    Approval { device: String, mode: ApprovalArg },
    /// Refuse every connection from a device.
    Block { device: String },
    /// Allow a blocked device again.
    Unblock { id: String },
    /// List devices on the local network.
    Nearby,
    /// Manage the relays you run, used to reach devices on other networks.
    Relay {
        #[command(subcommand)]
        command: RelayCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum RelayCommand {
    /// Show your relays.
    Show,
    /// Test every relay and explain what to fix.
    Check,
    /// Use your own relay server.
    Add { url: String },
    /// Stop using a relay server.
    Remove { url: String },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ApprovalArg {
    /// Files arrive right away.
    Auto,
    /// Every transfer waits for your answer.
    Ask,
}
