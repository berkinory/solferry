use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

use crate::help;

#[derive(Debug, Parser)]
#[command(
    name = "solferry",
    version,
    about = "Accountless file transfer between your devices",
    after_help = help::SHORT,
    after_long_help = help::AGENTS
)]
pub struct Cli {
    /// Where the device key, settings and paired devices live. Each directory is a separate device.
    #[arg(long, global = true, env = "SOLFERRY_DATA_DIR")]
    pub data_dir: Option<PathBuf>,
    /// Print JSON to stdout, one value per line, and errors as JSON to stderr.
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
    Rename {
        /// The new name.
        name: String,
    },
    /// Send files or folders to one or more devices.
    #[command(after_help = help::SEND)]
    Send {
        /// Files or folders to send. A lone - sends stdin as one file.
        #[arg(required_unless_present = "text", conflicts_with = "text")]
        paths: Vec<PathBuf>,
        /// The file name the receiver sees for stdin. Defaults to stdin.txt, or stdin.bin when it isn't text.
        #[arg(long, conflicts_with = "text")]
        name: Option<String>,
        /// Send this text to a paired device's clipboard instead of files. Use - to read stdin.
        #[arg(long)]
        text: Option<String>,
        /// Device name, id prefix or nearby device name. Repeat to send to several.
        #[arg(long = "to", required = true)]
        targets: Vec<String>,
    },
    /// Share files with a browser on this network, or take files from one when no paths are given.
    #[command(after_help = help::LINK)]
    Link {
        /// Files to share. Leave out to take files from a browser instead.
        paths: Vec<PathBuf>,
    },
    /// Stay online and receive files.
    #[command(after_help = help::RECEIVE)]
    Receive {
        /// Save here instead of the default download folder.
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Accept file offers from paired devices without asking. Unpaired devices are still asked.
        #[arg(long)]
        yes: bool,
        /// Exit after the first finished incoming transfer.
        #[arg(long)]
        once: bool,
    },
    /// Create a pairing code for another device.
    #[command(after_help = help::PAIR)]
    Pair {
        /// Pair with this nearby device by comparing a code, instead of showing a pairing code.
        #[arg(long)]
        with: Option<String>,
        /// Pair by comparing a code with the device at this IP address, when it isn't found nearby.
        #[arg(long, conflicts_with = "with")]
        address: Option<String>,
    },
    /// Redeem a pairing code from another device.
    #[command(after_help = help::JOIN)]
    Join {
        /// The code `solferry pair` printed on the other device, starting with sf1.
        code: String,
    },
    /// List paired devices.
    Devices,
    /// Remove a paired device.
    Forget {
        /// Paired device name or id prefix, as `solferry devices` shows.
        device: String,
    },
    /// Choose whether a paired device needs your approval before sending files.
    Approval {
        /// Paired device name or id prefix, as `solferry devices` shows.
        device: String,
        mode: ApprovalArg,
    },
    /// Refuse every connection from a device.
    Block {
        /// Paired device name or id prefix, or the full id of any device.
        device: String,
    },
    /// Allow a blocked device again.
    Unblock {
        /// Blocked device id or id prefix, as `solferry devices` shows.
        id: String,
    },
    /// List devices on the local network.
    #[command(after_help = help::NEARBY)]
    Nearby {
        /// Ask only this IP address, for a device on another subnet or a network that hides it.
        #[arg(long)]
        address: Option<String>,
    },
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
    Add {
        /// The relay link with its token, such as `https://relay.example.com#token`.
        url: String,
    },
    /// Stop using a relay server.
    Remove {
        /// The relay's URL. The token after # may be left out.
        url: String,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ApprovalArg {
    /// Files arrive right away.
    Auto,
    /// Every transfer waits for your answer.
    Ask,
}
