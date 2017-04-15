extern crate libc;
extern crate clap;
extern crate nix;
extern crate regex;

use std::path::Path;
use std::str::FromStr;
use std::os::unix::io::RawFd;

use clap::{App, Arg};

use nix::sys::socket;
use nix::sys::socket::SetSockOpt;

mod config;
use config::{Config, ServiceSpec, ConfigError};

struct Listener;

#[derive(Debug)]
enum ListenerError {
    ListenFailed(nix::Error),
}

impl Listener {
    fn listen(service_spec: ServiceSpec) -> Result<RawFd, ListenerError> {
        let fd = try!(socket::socket(match service_spec.addr.is_ipv4() {
                                         true => socket::AddressFamily::Inet,
                                         false => socket::AddressFamily::Inet6,
                                     },
                                     socket::SockType::Stream,
                                     socket::SockFlag::empty(),
                                     0)
            .map_err(ListenerError::ListenFailed));

        // set reuse addr
        {
            let opt = socket::sockopt::ReuseAddr {};
            try!(opt.set(fd, &true).map_err(ListenerError::ListenFailed));
        }

        // listen on specified addr
        {
            let addr = socket::InetAddr::from_std(&service_spec.addr);
            let sock_addr = socket::SockAddr::new_inet(addr);
            socket::bind(fd, &sock_addr).unwrap();
            try!(socket::listen(fd, service_spec.backlog).map_err(ListenerError::ListenFailed));
        }

        Ok(fd)
    }
}

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
    println!("Config: {:?}", config);
}
