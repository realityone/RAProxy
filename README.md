# RAProxy

[![Build Status](https://travis-ci.org/realityone/RAProxy.svg?branch=master)](https://travis-ci.org/realityone/RAProxy)

Reloadable HAProxy inspired from [multibinder](https://github.com/github/multibinder).

## Requirements

- Rust(only on build)
- HAProxy
- Linux or macOS

## Installation

### Install Rust

Using [rustup](https://www.rustup.rs) is recommended.

### Build from source code

```bash
# git clone https://github.com/realityone/RAProxy.git
# cd Raproxy
# cargo install
```

Then you can run `raproxy` in your terminal.

```bash
# raproxy 
error: The following required arguments were not provided:
    --binary <binary>
    --config <cfg>
    --service <service>...

USAGE:
    raproxy --binary <binary> --config <cfg> --service <service>... --pid <pid>

For more information try --help
```

## Usage

### Install HAProxy

First of all, please make sure you have installed original HAProxy on your machine.

Simply type `haproxy` in terminal for test.

```bash
# haproxy 
HA-Proxy version 1.7.2 2017/01/13
Copyright 2000-2017 Willy Tarreau <willy@haproxy.org>

...
```

If error, you should install it first.

### Config your HAProxy config file

Here is an example.

```conf
global
  maxconn 256
defaults
  mode http
  timeout connect 5000ms
  timeout client 50000ms
  timeout server 50000ms

frontend http-in
  bind fd@${APP_1}
  use_backend sina

backend sina
  server sina www.sina.com:80
```

In frontend `http-in`, use `fd@{__NAME__}` for bind specification. The `__NAME__` should be your service name which will defined in RAProxy, I will explain it later.

### Start RAProxy

- The HAProxy binary is at `/usr/local/bin/haproxy`
- The HAProxy config file is at `/etc/haproxy/haproxy.cfg`
- We want bind frontend `http-in` at address `0.0.0.0:8080` for incoming traffic. And we name it as `APP_1`.

```bash
# raproxy -b /usr/local/bin/haproxy -c /etc/haproxy/haproxy.cfg -s APP_1=0.0.0.0:8080
INFO:raproxy: HAProxy process started: PID 46972
```

Using `curl` for test.

```bash
# curl -H 'Host: sina.com' 127.0.0.1:8080
<html>
<head><title>301 Moved Permanently</title></head>
<body bgcolor="white">
<center><h1>301 Moved Permanently</h1></center>
<hr><center>nginx/1.5.2</center>
</body>
</html>
```

### Reload HAProxy

Find the `raproxy` process, and emit `SIGHUP` signal.

```bash
# ps ax | grep raproxy
46971 s001  S+     0:00.00 raproxy -b /usr/local/bin/haproxy -c /etc/haproxy/haproxy.cfg -s APP_1=0.0.0.0:8080
47083 s004  S+     0:00.00 grep raproxy
# kill -s HUP 46971
```

Then the new HAProxy process will be created, and the last will exit gracefully.

```
# raproxy -b /usr/local/bin/haproxy -c /etc/haproxy/haproxy.cfg -s APP_1=0.0.0.0:8080
INFO:raproxy: HAProxy process started: PID 46972
INFO:raproxy: HAProxy process started: PID 47098
INFO:raproxy: Process exited: Exited(46972, 0)
```

## TODO

* [x] Auto detect HAProxy binary.
* [ ] Reload RAProxy self.
* [ ] Integrate with SystemD.

## Reference

- [GLB part 2: HAProxy zero-downtime, zero-delay reloads with multibinder](https://githubengineering.com/glb-part-2-haproxy-zero-downtime-zero-delay-reloads-with-multibinder/)
- [SO_REUSEADDR option](http://man7.org/linux/man-pages/man7/socket.7.html#SO_REUSEADDR)



