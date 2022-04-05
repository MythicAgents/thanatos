+++
title = "socks"
chapter = false
weight = 103
hidden = true
+++

## Summary

This establishes a [SOCKS5 proxy](https://docs.mythic-c2.net/operational-pieces/socks-proxy) through the Tetanus agent, permitting tooling to be proxied through the compromised host.
Currently only outgoing TCP connections are supported.

- Needs Admin: False  
- Version: 1  

### Arguments

#### action

- Description: start/stop the proxy
- Required Value: True
- Default Value: None

#### port

- Description: The port on the Mythic server to open for SOCKS traffic
- Required Value: True
- Default Value: None

## Usage
```
socks start/stop [port]
```

Example
```
socks start 7000
```

## MITRE ATT&CK Mapping

- T1090
