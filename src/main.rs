extern crate clap;
extern crate nix;
extern crate regex;

mod config;
mod haproxy;

use std::{thread, process};
use std::path::Path;
use std::sync::Mutex;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;
use std::fs::remove_file;
use std::collections::HashSet;

use nix::sys::wait;
use nix::sys::signal;
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

fn config_from_cli() -> (String, String, String, Vec<String>) {
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
    (matches.value_of("binary").unwrap().to_string(),
     matches.value_of("cfg").unwrap().to_string(),
     matches.value_of("pid").unwrap().to_string(),
     matches.values_of("service")
        .unwrap()
        .map(|v| v.to_string())
        .collect())
}

fn cleanup_loop() {
    loop {
        if let Ok(status) = wait::wait() {
            println!("Process exited: {:?}", status);
            continue;
        }
        sleep(Duration::new(10, 0));
    }
}

fn start_haproxy_process(haproxy: &mut HAProxy) {
    let child = haproxy.start_process().expect("Start HAProxy process failed");
    if let &mut Some(ref mut child) = child {
        println!("HAProxy process started: PID {}", child.id());
    } else {
        panic!("HAProxy process not exist");
    }
}

fn main() {
    let (binary, config, pid, services) = config_from_cli();
    let services: Vec<&str> = services.iter().map(|s| s.as_ref()).collect();
    let config = Config::new(&binary, &config, &pid, services.as_slice());

    let mut haproxy = HAProxy::init_from_config(&config);
    start_haproxy_process(&mut haproxy);
    // start clean up loop
    thread::spawn(move || cleanup_loop());

    let mut mask = signal::SigSet::empty();
    mask.add(signal::SIGHUP);
    mask.add(signal::SIGTERM);
    mask.add(signal::SIGINT);
    loop {
        let sig = mask.wait().expect("Wait signal failed");
        match sig {
            signal::SIGHUP => {
                start_haproxy_process(&mut haproxy);
            }
            signal::SIGTERM | signal::SIGINT => {
                if let Ok(pid) = haproxy.worker_pid() {
                    if let Err(e) = signal::kill(pid as i32, signal::SIGUSR1) {
                        println!("Stop HAProxy process failed: {}", e);
                    }
                    if let Err(e) = remove_file(haproxy.config.pid) {
                        println!("Remove HAProxy PID file failed: {}", e);
                    }
                }
                println!("RAProxy exited.");
                process::exit(1);
            }
            _ => unreachable!("Unexpected signal: {:?}", sig),
        }
    }
}
