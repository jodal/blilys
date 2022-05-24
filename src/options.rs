use hueclient::CommandLight;
use structopt::StructOpt;

use std::net::IpAddr;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "blilys",
    about = "Control Philips Hue lights from the command line."
)]
pub struct Opt {
    /// IP address. If not provided, auto discovery is attempted.
    #[structopt(short, long)]
    pub bridge: Option<IpAddr>,
    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// Pair with bridge to get a username.
    Pair,
    /// Show config.
    Config,
    /// List available groups.
    Groups,
    // Control a group.
    Group {
        group: usize,
        #[structopt(subcommand)]
        op: LightOperation,
    },
    /// List available lights.
    Lights,
    /// Control a light.
    Light {
        light: usize,
        #[structopt(subcommand)]
        op: LightOperation,
    },
}

#[derive(Debug, StructOpt)]
pub enum LightOperation {
    /// Turn light on.
    On {
        #[structopt(short, long, help = "Brightness")]
        bri: Option<u8>,
    },
    /// Turn light off.
    Off,
    /// Enable special mode.
    Mode {
        #[structopt(subcommand)]
        mode: LightMode,
    },
}

#[derive(Debug, StructOpt)]
pub enum LightMode {
    /// Halloween mode with scary blinking lights.
    Halloween,
}

impl LightOperation {
    pub fn to_command(&self) -> CommandLight {
        match self {
            LightOperation::On { bri } => {
                let mut command = CommandLight::default().on();
                if let Some(bri) = bri {
                    command = command.with_bri(*bri);
                }
                command
            }
            LightOperation::Off => CommandLight::default().off(),
            LightOperation::Mode { mode: _ } => CommandLight::default(),
        }
    }
}
