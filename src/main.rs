#[macro_use]
extern crate log;
extern crate env_logger;

extern crate clap;
extern crate nix;
extern crate regex;

mod config;
mod haproxy;

use std::thread::sleep;
use std::time::Duration;
use std::fs::remove_file;
use std::{thread, process, env};

use nix::sys::{wait, signal};

use haproxy::HAProxy;
use config::ConfigBuilder;

fn cleanup_loop() {
    loop {
        if let Ok(status) = wait::wait() {
            info!("Process exited: {:?}", status);
            continue;
        }
        sleep(Duration::new(10, 0));
    }
}

fn start_haproxy_process(haproxy: &mut HAProxy) {
    let child = haproxy.start_process().expect("Start HAProxy process failed");
    if let &mut Some(ref mut child) = child {
        info!("HAProxy process started: PID {}", child.id());
    } else {
        panic!("HAProxy process not exist");
    }
}

fn main() {
    if let Err(_) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init().expect("Init logger failed");

    let builder = config::cli::new();
    let config = builder.build();

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
                        error!("Stop HAProxy process failed: {}", e);
                    }
                    if let Err(e) = remove_file(haproxy.config.pid) {
                        error!("Remove HAProxy PID file failed: {}", e);
                    }
                }
                info!("RAProxy exited.");
                process::exit(1);
            }
            _ => unreachable!("Unexpected signal: {:?}", sig),
        }
    }
}
