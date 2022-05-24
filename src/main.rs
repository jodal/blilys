use crate::config::Config;
use eyre::Result;
use hueclient::CommandLight;
use rand::distributions::{Distribution, Uniform};
use std::io;
use std::net::IpAddr;
use std::time::Duration;
use structopt::StructOpt;

mod config;

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
    /// Halloween mode.
    Halloween { light: usize },
}

fn main() -> Result<()> {
    let mut config = Config::from_file()?;
    let opt = Opt::from_args();

    let unauth_bridge = match opt.bridge {
        Some(ip) => hueclient::Bridge::for_ip(ip),
        None => match config.bridge.ip {
            Some(ip) => hueclient::Bridge::for_ip(ip),
            None => hueclient::Bridge::discover_required(),
        },
    };

    let bridge = if let Command::Pair = opt.cmd {
        pair(unauth_bridge, &mut config)?
    } else {
        match config.bridge.username {
            Some(ref username) => unauth_bridge.with_user(username),
            None => pair(unauth_bridge, &mut config)?,
        }
    };

    match opt.cmd {
        Command::Pair => {
            // Pairing is handled above, when creating the authenticated Bridge.
        }
        Command::Config => {
            config.print()?;
        }
        Command::List => {
            for il in bridge.get_all_lights()? {
                println!("{id:2}: {name}", id = il.id, name = il.light.name,);
                println!(
                    "    {on}, brightness: {bri}, hue: {hue}",
                    on = if il.light.state.on { "On" } else { "Off" },
                    bri = il.light.state.bri.unwrap_or(0).to_string(),
                    hue = il.light.state.hue.unwrap_or(0).to_string()
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
        Command::Halloween { light } => loop {
            let command: CommandLight = Default::default();
            bridge.set_light_state(light, &command.with_bri(rand_bri(1, 50)))?;
            sleep_a_bit();

            let command: CommandLight = Default::default();
            bridge.set_light_state(light, &command.with_bri(rand_bri(70, 120)))?;
            sleep_a_bit();
        },
    }

    Ok(())
}

fn pair(unauth_bridge: hueclient::UnauthBridge, config: &mut Config) -> Result<hueclient::Bridge> {
    eprintln!("Discovered Philips Hue bridge at {}.", unauth_bridge.ip);
    eprintln!("To pair, press the button on your bridge now.");
    eprintln!("Then, press any key to continue pairing ...");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    eprintln!("Registering user ...");
    let bridge = unauth_bridge.register_user("blilys")?;
    eprintln!("Pairing complete.");

    eprintln!("Saving configuration ...");
    config.bridge.ip = Some(bridge.ip);
    config.bridge.username = Some(bridge.username.to_owned());
    config.save()?;
    config.print()?;

    Ok(bridge)
}

fn rand_bri(low: u8, high: u8) -> u8 {
    let between = Uniform::from(low..high);
    let mut rng = rand::thread_rng();
    return between.sample(&mut rng);
}

fn sleep_a_bit() {
    let between = Uniform::from(200..1000);
    let mut rng = rand::thread_rng();
    std::thread::sleep(Duration::from_millis(between.sample(&mut rng)));
}
