extern crate libc;
extern crate clap;
extern crate nix;
extern crate regex;

use std::path::Path;
use std::str::FromStr;

use clap::{App, Arg};

mod config;
mod haproxy;

use config::{Config, ServiceSpec, ConfigError};
use haproxy::haproxy_process;

fn path_validator(v: String) -> Result<(), String> {
    if let Err(e) = Config::validate_path(v) {
        return Err(match e {
            ConfigError::InvalidPath(err) => err,
        });
    }
    Ok(())
}

fn main() {
    let matches = App::new("RAProxy")
        .version("0.1.0")
        .about("Reloadable HAProxy utility.")
        .arg(Arg::with_name("haproxy")
            .help("The path to HAProxy binary.")
            .long("haproxy")
            .short("b")
            .takes_value(true)
            .validator(path_validator)
            .required(true))
        .arg(Arg::with_name("cfg")
            .help("The path to HAProxy config template.")
            .long("config")
            .short("c")
            .takes_value(true)
            .validator(path_validator)
            .required(true))
        .arg(Arg::with_name("service")
            .help("The service specification.")
            .long("service")
            .short("s")
            .validator(|v| {
                try!(ServiceSpec::from_str(&v)
                    .map_err(|_| format!("The specification `{}` is an invalid service spec", v)));
                Ok(())
            })
            .takes_value(true)
            .required(true)
            .multiple(true))
        .get_matches();
    let config = Config {
        haproxy: &Path::new(matches.value_of("haproxy").unwrap()),
        config: &Path::new(matches.value_of("cfg").unwrap()),
        services: matches.values_of("service")
            .unwrap()
            .map(|v| {
                let spec = ServiceSpec::from_str(v).unwrap();
                (spec.name.clone(), spec)
            })
            .collect(),
    };
    let mut process = haproxy_process(&config).expect("Create haproxy process failed");
    let child = process.spawn().expect("Spawn haproxy process failed");
    println!("Config: {:?}", config);
}
