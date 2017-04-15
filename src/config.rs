use std::path::Path;
use std::str::FromStr;
use std::net::SocketAddr;
use std::os::unix::io::RawFd;
use std::collections::HashMap;

use regex::Regex;

const DEFAULT_BACKLOG: usize = 1000;

#[derive(Debug)]
pub struct Config<'a> {
    pub binary: &'a Path,
    pub config: &'a Path,
    pub pid: &'a Path,
    pub services: HashMap<String, ServiceSpec>,
}

#[derive(Debug)]
pub struct ServiceSpec {
    pub name: String,
    pub addr: SocketAddr,
    pub backlog: usize,
    pub fd: Option<RawFd>,
}

#[derive(Debug)]
pub enum ConfigError {
    InvalidPath(String),
}

#[derive(Debug)]
pub enum ServiceSpecError {
    InvalidServiceSpec,
}

impl<'a> Config<'a> {
    pub fn validate_path(v: String) -> Result<(), ConfigError> {
        let path = Path::new(&v);
        if !path.is_file() {
            return Err(ConfigError::InvalidPath(format!("Path `{}` is not a regular file", v)));
        }
        if !path.exists() {
            return Err(ConfigError::InvalidPath(format!("Path `{}` is not exist", v)));
        }
        Ok(())
    }
}

impl FromStr for ServiceSpec {
    type Err = ServiceSpecError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pattern = Regex::new(r"^(?P<name>\w+)=(?P<addr>\S[^@]+)(?:@(?P<backlog>\d+))?$")
            .unwrap();
        let caps = pattern.captures(s);
        if caps.is_none() {
            return Err(ServiceSpecError::InvalidServiceSpec);
        }
        let caps = caps.unwrap();
        let mut backlog = DEFAULT_BACKLOG;
        if caps.name("backlog").is_some() {
            backlog = try!(usize::from_str(&caps["backlog"])
                .map_err(|_| ServiceSpecError::InvalidServiceSpec));
        }
        Ok(ServiceSpec {
            name: caps["name"].to_string(),
            addr: try!(SocketAddr::from_str(&caps["addr"])
                .map_err(|_| ServiceSpecError::InvalidServiceSpec)),
            backlog: backlog,
            fd: None,
        })
    }
}