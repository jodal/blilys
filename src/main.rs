use directories::ProjectDirs;
use eyre::Result;
use hueclient::bridge::CommandLight;
use serde::{Deserialize, Serialize};
use std::io;
use std::net::IpAddr;
use std::{fs, path::Path};
use structopt::StructOpt;

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    bridge: Bridge,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            bridge: Bridge {
                ip: None,
                username: None,
            },
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
struct Bridge {
    ip: Option<IpAddr>,
    username: Option<String>,
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "blilys",
    about = "Control Philips Hue lights from the command line."
)]
struct Opt {
    /// IP address. If not provided, auto discovery is attempted.
    #[structopt(short, long)]
    bridge: Option<IpAddr>,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Pair with bridge to get a username.
    Pair,
    /// Show config.
    Config,
    /// List available lights.
    List,
    /// Turn light on.
    On { light: usize },
    /// Turn light off.
    Off { light: usize },
    /// Set brightness.
    Bri { light: usize, bri: u8 },
}

fn main() -> Result<()> {
    let project_dirs = ProjectDirs::from("", "", "blilys").expect("Config dir not readable");
    let config_dir = project_dirs.config_dir();
    if !config_dir.is_dir() {
        fs::create_dir_all(config_dir)?;
    }
    let config_path = config_dir.join("config.toml");
    let mut config: Config = match config_path.is_file() {
        true => toml::from_str(String::from_utf8(fs::read(&config_path)?)?.as_ref())?,
        false => Default::default(),
    };

    let opt = Opt::from_args();
    let command = Command::from_args();

    let mut bridge = match opt.bridge {
        Some(ip) => hueclient::bridge::Bridge::for_ip(ip),
        None => match config.bridge.ip {
            Some(ip) => hueclient::bridge::Bridge::for_ip(ip),
            None => hueclient::bridge::Bridge::discover_required(),
        },
    };

    match config.bridge.username.to_owned() {
        Some(username) => {
            bridge = bridge.with_user(username);
        }
        None => {}
    }

    match command {
        Command::Pair => {
            eprintln!("Discovered Philips Hue bridge at {}.", bridge.ip);
            eprintln!("To pair, press the button on your bridge now.");
            eprintln!("Then, press any key to continue pairing ...");

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();

            eprintln!("Registering user ...");
            let username = bridge.register_user("blilys")?;
            eprintln!("Pairing complete.");

            eprintln!("Writing configuration ...");
            config = Config {
                bridge: Bridge {
                    ip: Some(bridge.ip),
                    username: Some(username),
                    ..config.bridge
                },
                ..config
            };
            fs::write(&config_path, toml::to_string(&config)?)?;
            print_config(&config_path, &config)?;
        }
        Command::Config => {
            print_config(&config_path, &config)?;
        }
        Command::List => {
            for il in bridge.get_all_lights()? {
                println!("{id:2}: {name}", id = il.id, name = il.light.name,);
                println!(
                    "    {on}, brightness: {bri}",
                    on = if il.light.state.on { "On" } else { "Off" },
                    bri = il.light.state.bri.unwrap_or(0).to_string()
                );
            }
        }
        Command::On { light } => {
            let command: CommandLight = Default::default();
            bridge.set_light_state(light, &command.on())?;
        }
        Command::Off { light } => {
            let command: CommandLight = Default::default();
            bridge.set_light_state(light, &command.off())?;
        }
        Command::Bri { light, bri } => {
            let command: CommandLight = Default::default();
            bridge.set_light_state(light, &command.with_bri(bri))?;
        }
    }

    Ok(())
}

fn print_config(config_path: &Path, config: &Config) -> Result<()> {
    eprintln!("# {}", config_path.to_string_lossy());
    print!("{}", toml::to_string(&config)?);
    Ok(())
}
