+++
title = "netstat"
chapter = false
weight = 103
hidden = true
+++

## Description
Report on all active network connections. Note that this may require the agent to be running in a `root` context for complete enumeration.

## Usage
```
netstat
```

### Examples
```
netstat
```
```
[
    {
        "proto": "TCP",
        "local_addr": "127.0.0.53",
        "local_port": 53,
        "remote_addr": "0.0.0.0",
        "remote_port": 0,
        "associated_pids": "",
        "state": "LISTEN"
    },
    {
        "proto": "TCP",
        "local_addr": "0.0.0.0",
        "local_port": 22,
        "remote_addr": "0.0.0.0",
        "remote_port": 0,
        "associated_pids": "",
        "state": "LISTEN"
    },
    {
        "proto": "TCP",
        "local_addr": "10.0.0.1",
        "local_port": 22,
        "remote_addr": "10.0.0.2",
        "remote_port": 40803,
        "associated_pids": "",
        "state": "ESTABLISHED"
    }
]
```

## MITRE ATT&CK Mapping
 - T1049
