+++
title = "portscan"
chapter = false
weight = 103
hidden = true
+++

## Description
Scan machines for open ports

### Parameters
Enter parameters through the Mythic UI by entering the command by itself.
```
portscan
```
![portscan_popup](../images/portscan_popup.png)

##### hosts/subnets
The `hosts/subnets` argument is a list of hosts or subnets to run the portscan on.

##### ports/port range
The `posrts/port range` argument is a comma separated list of ports or hyphen separated port ranges to
scan.

![portscan_ports](../images/portscan_ports.png)

##### interval
The interval is used to specify a sleep interval before each scan is made. The
default interval is 1000 milli-seconds (1 second) meaning that every time the agent scans a
port, it will sleep for 1 second before scanning another port.

### Notes
The agent will post results as they come in instead of waiting for the scan to complete
and then posting results.

#### Killable
The port scan runs as a background job. If a kill command is sent using `jobkill` to the
port scan job, it will stop scanning.

## MITRE ATT&CK Mapping
 - T1046
