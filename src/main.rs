extern crate libc;
extern crate clap;
extern crate nix;
extern crate regex;

mod config;
mod haproxy;

use std::panic;
use std::path::Path;
use std::sync::Mutex;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;
use std::fs::remove_file;
use std::collections::HashSet;

use nix::errno;
use clap::{App, Arg};

use haproxy::HAProxy;
use config::{Config, ServiceSpec, ConfigError};

fn path_validator(v: String) -> Result<(), String> {
    if let Err(e) = Config::validate_path(v) {
        return Err(match e {
            ConfigError::InvalidPath(err) => err,
        });
    }
    Ok(())
}


fn main() {
    let service_names = Mutex::new(HashSet::new());
    let matches = App::new("RAProxy")
        .version("0.1.0")
        .about("Reloadable HAProxy utility.")
        .arg(Arg::with_name("binary")
            .help("The path to HAProxy binary.")
            .long("binary")
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
        .arg(Arg::with_name("pid")
            .help("The path to HAProxy process pid.")
            .long("pid")
            .short("p")
            .validator(|v| {
                let p = Path::new(&v);
                if p.exists() {
                    try!(remove_file(p).map_err(|e| format!("Remove file `{}` failed: {}", v, e)));
                }
                Ok(())
            })
            .required(false)
            .default_value("/tmp/haproxy.pid"))
        .arg(Arg::with_name("service")
            .help("The service specification.")
            .long("service")
            .short("s")
            .validator(move |v| {
                let s = try!(ServiceSpec::from_str(&v)
                    .map_err(|_| format!("The specification `{}` is an invalid service spec", v)));
                let mut service_names = service_names.lock().unwrap();
                if service_names.contains(&s.name) {
                    return Err(format!("Service `{}` already exist", s.name));
                }
                service_names.insert(s.name);
                Ok(())
            })
            .takes_value(true)
            .required(true)
            .multiple(true))
        .get_matches();
    let config = Config {
        binary: &Path::new(matches.value_of("binary").unwrap()),
        config: &Path::new(matches.value_of("cfg").unwrap()),
        pid: &Path::new(matches.value_of("pid").unwrap()),
        services: matches.values_of("service")
            .unwrap()
            .map(|v| ServiceSpec::from_str(v).unwrap())
            .collect(),
    };

    let mut haproxy = HAProxy::from_config(&config);
    let child = haproxy.start_process().expect("Start HAProxy process failed");
    if let &mut Some(ref mut child) = child {
        println!("HAProxy process started: PID {}", child.id());
    } else {
        panic!("HAProxy process not exist");
    }

    loop {
        let wait_status = nix::sys::wait::wait();
        if wait_status.is_ok() {
            println!("Wait status: {:?}", wait_status);
            continue;
        }
        sleep(Duration::new(10, 0))
    }
}
