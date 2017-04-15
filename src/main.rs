extern crate libc;
extern crate clap;
extern crate nix;

use std::path::Path;
use std::str::FromStr;
use std::net::SocketAddr;
use clap::{App, Arg};

#[derive(Debug)]
struct Config<'a> {
    haproxy: &'a Path,
    config: &'a Path,
    binds: Vec<BindSpec>,
}

#[derive(Debug)]
struct BindSpec {
    addr: SocketAddr,
    backlog: usize,
}

const DEFAULT_BACKLOG: usize = 1000;
#[derive(Debug)]
enum BindSpecError {
    InvalidBindSpec,
}

fn path_validator(v: String) -> Result<(), String> {
    let path = Path::new(&v);
    if !path.is_file() {
        return Err(format!("Path `{}` is not a regular file", v));
    }
    if !path.exists() {
        return Err(format!("Path `{}` is not exist", v));
    }
    Ok(())
}

impl FromStr for BindSpec {
    type Err = BindSpecError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.contains(",") {
            let sa = SocketAddr::from_str(&s);
            if sa.is_err() {
                return Err(BindSpecError::InvalidBindSpec);
            }
            return Ok(BindSpec {
                addr: sa.unwrap(),
                backlog: DEFAULT_BACKLOG,
            });
        }
        let splited: Vec<&str> = s.splitn(2, ",").collect();
        let sa = SocketAddr::from_str(splited[0]);
        let bl = usize::from_str(splited[1]);
        if sa.is_err() || bl.is_err() {
            return Err(BindSpecError::InvalidBindSpec);
        }
        Ok(BindSpec {
            addr: sa.unwrap(),
            backlog: bl.unwrap(),
        })
    }
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
        .arg(Arg::with_name("bind")
            .help("The port bind specification.")
            .long("bind")
            .short("a")
            .validator(|v| {
                if BindSpec::from_str(&v).is_err() {
                    return Err(format!("The specification `{}` is an invalid bind spec", v));
                }
                Ok(())
            })
            .takes_value(true)
            .required(true)
            .multiple(true))
        .get_matches();
    let config = Config {
        haproxy: &Path::new(matches.value_of("haproxy").unwrap()),
        config: &Path::new(matches.value_of("cfg").unwrap()),
        binds: matches.values_of("bind")
            .unwrap()
            .map(|v| BindSpec::from_str(v).unwrap())
            .collect(),
    };
    println!("Config: {:?}", config);
}
