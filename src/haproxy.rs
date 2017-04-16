use std::{io, fs, num};
use std::io::Read;
use std::str::FromStr;
use std::os::unix::io::RawFd;
use std::collections::HashMap;
use std::process::{Command, Child};

use nix;
use nix::sys::socket;
use nix::sys::socket::SetSockOpt;

use config::{Config, ServiceSpec};

struct Listener;

#[derive(Debug)]
pub struct HAProxy<'a> {
    config: &'a Config<'a>,
    services: HashMap<&'a ServiceSpec, RawFd>,
    pub process: Option<Child>,
}

#[derive(Debug)]
pub enum ListenerError {
    ListenFailed(nix::Error),
}

#[derive(Debug)]
pub enum HAProxyProcessError {
    CreateCommandFailed(ListenerError),
    StartCommandFailed(io::Error),
    ReadWorkerPIDFailed(io::Error),
    InvalidPID(num::ParseIntError),
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
            try!(socket::bind(fd, &sock_addr).map_err(ListenerError::ListenFailed));
            try!(socket::listen(fd, service_spec.backlog).map_err(ListenerError::ListenFailed));
        }

        Ok(fd)
    }
}

impl<'a> HAProxy<'a> {
    pub fn init_from_config(config: &'a Config) -> Self {
        HAProxy {
            config: config,
            services: Default::default(),
            process: None,
        }
    }

    pub fn worker_pid(&self) -> Result<u32, HAProxyProcessError> {
        let mut pid_data = String::new();
        let mut fp = try!(fs::File::open(self.config.pid)
            .map_err(HAProxyProcessError::ReadWorkerPIDFailed));
        try!(fp.read_to_string(&mut pid_data).map_err(HAProxyProcessError::ReadWorkerPIDFailed));
        u32::from_str(&pid_data.trim()).map_err(HAProxyProcessError::InvalidPID)
    }

    fn create_command(&mut self) -> Result<Command, HAProxyProcessError> {
        let mut haproxy = Command::new(self.config.binary.as_os_str());
        haproxy.arg("-f").arg(self.config.config.as_os_str());
        haproxy.arg("-p").arg(self.config.pid.as_os_str());
        haproxy.arg("-Ds");
        if self.process.is_some() {
            let worker_pid = try!(self.worker_pid());
            haproxy.arg("-sf").arg(format!("{}", worker_pid));
        }

        haproxy.env_clear();
        for spec in self.config.services.iter() {
            let fd;
            if !self.services.contains_key(spec) {
                fd = try!(Listener::listen(spec).map_err(HAProxyProcessError::CreateCommandFailed));
                self.services.insert(spec, fd);
            } else {
                fd = self.services[spec];
            }
            haproxy.env(spec.name.clone(), format!("{}", fd));
        }
        Ok(haproxy)
    }

    pub fn start_process(&mut self) -> Result<&mut Option<Child>, HAProxyProcessError> {
        let mut command = try!(self.create_command());
        self.process = Some(try!(command.spawn().map_err(HAProxyProcessError::StartCommandFailed)));
        Ok(&mut self.process)
    }
}
