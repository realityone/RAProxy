use std::process::Command;
use std::os::unix::io::RawFd;

use nix;
use nix::sys::socket;
use nix::sys::socket::SetSockOpt;

use config::{Config, ServiceSpec};

struct Listener;

#[derive(Debug)]
pub enum ListenerError {
    ListenFailed(nix::Error),
}

#[derive(Debug)]
pub enum HAProxyProcessError {
    CreateHAProxyFailed(ListenerError),
}

impl Listener {
    fn listen(service_spec: &ServiceSpec) -> Result<RawFd, ListenerError> {
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

pub fn haproxy_process(cfg: &Config) -> Result<Command, HAProxyProcessError> {
    let mut haproxy = Command::new(cfg.haproxy.as_os_str());
    haproxy.arg("-f").arg(cfg.config.as_os_str());

    for (_, service_spec) in &cfg.services {
        let fd = try!(Listener::listen(&service_spec)
            .map_err(HAProxyProcessError::CreateHAProxyFailed));
        haproxy.env(service_spec.name.clone(), format!("{}", fd));
    }
    Ok(haproxy)
}