use std::path::Path;
use std::str::FromStr;
use std::net::SocketAddr;
use std::collections::HashSet;

use regex::Regex;

pub mod cli;

const DEFAULT_BACKLOG: usize = 1000;

#[derive(Debug)]
pub struct Config<'a> {
    pub binary: &'a Path,
    pub config: &'a Path,
    pub pid: &'a Path,
    pub services: HashSet<ServiceSpec>,
}

#[derive(Debug, Hash, Eq, PartialEq)]
pub struct ServiceSpec {
    pub name: String,
    pub addr: SocketAddr,
    pub backlog: usize,
}

#[derive(Debug)]
pub enum ConfigError {
    InvalidPath(String),
}

#[derive(Debug)]
pub enum ServiceSpecError {
    InvalidServiceSpec,
}

pub trait ConfigBuilder {
    fn build(&self) -> Config;
}

impl<'a> Config<'a> {
    pub fn new(binary: &'a str, config: &'a str, pid: &'a str, services: Vec<&'a str>) -> Self {
        Config {
            binary: &Path::new(binary),
            config: &Path::new(config),
            pid: &Path::new(pid),
            services: services.iter()
                .map(|v| ServiceSpec::from_str(v).unwrap())
                .collect(),
        }
    }

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
        })
    }
}
