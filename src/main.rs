extern crate libc;
extern crate clap;
extern crate nix;

use std::process::Command;
use nix::sys::socket;
use nix::sys::socket::bind;
use nix::sys::socket::SockAddr;
use nix::sys::socket::{InetAddr, UnixAddr, getsockname};
use nix::sys::socket::SetSockOpt;
use std::str::FromStr;
use std::net::SocketAddr;
// use std::os::unix::io::AsRawFd;
// use time::Duration;
// use clap::{App, Arg};

// fn cli_app() {
//     let matches = App::new("RAProxy")
//         .version("0.1")
//         .about("HAProxy reload helper.")
//         .author("realityone.")
//         .arg(Arg::with_name("haproxy")
//             .help("The HAProxy binary path.")
//             .long("haproxy")
//             .short("b")
//             .takes_value(true)
//             .required(true))
//         .arg(Arg::with_name("cfg")
//             .help("The HAProxy config path.")
//             .long("config")
//             .short("c")
//             .takes_value(true)
//             .required(true))
//         .get_matches();
// }

fn main() {
    let raw_fd = socket::socket(socket::AddressFamily::Inet,
                                socket::SockType::Stream,
                                socket::SockFlag::empty(),
                                0)
        .unwrap();

    let opt = socket::sockopt::ReuseAddr {};
    opt.set(raw_fd, &true);

    let actual: SocketAddr = FromStr::from_str("0.0.0.0:7878").unwrap();
    let addr = InetAddr::from_std(&actual);
    let sa = SockAddr::new_inet(addr);

    bind(raw_fd, &sa).unwrap();

    let mut child = Command::new("/usr/local/bin/haproxy")
        .arg("-f")
        .arg("/Users/realityone/Softs/raproxy/hap.cfg")
        .env("APP_FD", format!("{}", raw_fd))
        .spawn()
        .unwrap();
    let ecode = child.wait()
        .expect("failed to wait on child");
    unsafe {
        libc::sleep(30);
    }
}
