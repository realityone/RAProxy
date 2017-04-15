extern crate libc;
extern crate clap;
extern crate nix;
extern crate regex;

mod config;
mod haproxy;

use std::path::Path;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;
use std::fs::remove_file;

use nix::errno;
use clap::{App, Arg};

use haproxy::haproxy_process;
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
            .validator(|v| {
                try!(ServiceSpec::from_str(&v)
                    .map_err(|_| format!("The specification `{}` is an invalid service spec", v)));
                Ok(())
            })
            .takes_value(true)
            .required(true)
            .multiple(true))
        .get_matches();
    let mut config = Config {
        binary: &Path::new(matches.value_of("binary").unwrap()),
        config: &Path::new(matches.value_of("cfg").unwrap()),
        pid: &Path::new(matches.value_of("pid").unwrap()),
        services: matches.values_of("service")
            .unwrap()
            .map(|v| {
                let spec = ServiceSpec::from_str(v).unwrap();
                (spec.name.clone(), spec)
            })
            .collect(),
    };

    let mut initial = true;
    let mut cpid = None;

    let mut process = haproxy_process(&mut config, initial, cpid)
        .expect("Create haproxy process failed");
    let mut child = process.spawn().expect("Spawn haproxy process failed");
    cpid = Some(child.id());
    // let exit_status = child.wait().expect("HAProxy process wasn't running");
    // println!("HAProxy process exit with code: {}", exit_status);
    initial = false;

    loop {
        let wait_status = nix::sys::wait::wait();
        if wait_status.is_ok() {
            println!("Wait status: {:?}", wait_status);
            continue;
        }
        sleep(Duration::new(10, 0));
    }
}
