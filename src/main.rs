use crate::config::Config;
use crate::options::{Command, LightMode, LightOperation, Opt};
use eyre::Result;
use hueclient::CommandLight;
use rand::distributions::{Distribution, Uniform};
use std::io;
use std::time::Duration;
use structopt::StructOpt;

mod config;
mod options;

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
        Command::Groups => {
            for ig in bridge.get_all_groups()? {
                let mut lights = ig.group.lights.to_owned();
                lights.sort_by_key(|l| l.parse::<usize>().expect("Light ID to be a number"));
                println!(
                    "{id:2}: {name:30} [{lights}]",
                    id = ig.id,
                    name = ig.group.name,
                    lights = lights.join(", ")
                );
            }
        }
        Command::Group { group, op } => match op {
            LightOperation::Mode { mode } => match mode {
                LightMode::Halloween => loop {
                    let command = CommandLight::default().with_bri(rand_bri(1, 50));
                    bridge.set_group_state(group, &command)?;
                    sleep_a_bit();

                    let command = CommandLight::default().with_bri(rand_bri(70, 120));
                    bridge.set_group_state(group, &command)?;
                    sleep_a_bit();
                },
            },
            light_operation => {
                bridge.set_group_state(group, &light_operation.to_command())?;
            }
        },
        Command::Lights => {
            for il in bridge.get_all_lights()? {
                println!(
                    "{id:2}: {name:30} [{on:3}] [bri {bri:>3}] [hue {hue:>5}]",
                    id = il.id,
                    name = il.light.name,
                    on = if il.light.state.on { "on" } else { "off" },
                    bri = il.light.state.bri.unwrap_or(0).to_string(),
                    hue = il.light.state.hue.unwrap_or(0).to_string()
                );
            }
        }
        Command::Light { light, op } => match op {
            LightOperation::Mode { mode } => match mode {
                LightMode::Halloween => loop {
                    let command = CommandLight::default().with_bri(rand_bri(1, 50));
                    bridge.set_light_state(light, &command)?;
                    sleep_a_bit();

                    let command = CommandLight::default().with_bri(rand_bri(70, 120));
                    bridge.set_light_state(light, &command)?;
                    sleep_a_bit();
                },
            },
            light_operation => {
                bridge.set_light_state(light, &light_operation.to_command())?;
            }
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
