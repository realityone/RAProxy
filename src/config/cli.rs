use std::path::Path;
use std::sync::Mutex;
use std::str::FromStr;
use std::fs::remove_file;
use std::collections::HashSet;

use clap::{App, Arg};

use super::{ConfigBuilder, Config, ConfigError, ServiceSpec};

#[derive(Debug)]
pub struct CommandLine {
    binary: String,
    cfg: String,
    pid: String,
    services: Vec<String>,
}

impl ConfigBuilder for CommandLine {
    fn build(&self) -> Config {
        let services: Vec<&str> = self.services.iter().map(|s| s.as_ref()).collect();
        Config::new(&self.binary, &self.cfg, &self.pid, services)
    }
}

impl CommandLine {
    fn new() -> Self {
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
                        try!(remove_file(p)
                            .map_err(|e| format!("Remove file `{}` failed: {}", v, e)));
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
                    let s = try!(ServiceSpec::from_str(&v).map_err(|_| {
                        format!("The specification `{}` is an invalid service spec", v)
                    }));
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
        CommandLine {
            binary: matches.value_of("binary").unwrap().to_string(),
            cfg: matches.value_of("cfg").unwrap().to_string(),
            pid: matches.value_of("pid").unwrap().to_string(),
            services: matches.values_of("service")
                .unwrap()
                .map(|v| v.to_string())
                .collect(),
        }
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

pub fn new() -> CommandLine {
    CommandLine::new()
}