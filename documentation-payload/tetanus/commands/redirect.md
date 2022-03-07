+++
title = "redirect"
chapter = false
weight = 103
hidden = true
+++

## Description
Set up a TCP redirector on the machine. The agent will listen for TCP connections on a
specified interface IP address and port and redirect them to another host.

### Parameters
`bindhost`
 * Bind host or IP address

`bindport`
 * Bind port

`connecthost`
 * Host to redirect connections to

`connectport`
 * Port to redirect connections to

## Usage
```
redirect -bindhost [bindhost] -bindport [bindport] -connecthost [connecthost] -connectport [connectport]
```

![redirect_popup](../images/redirect_popup.png)

### Example
Redirect all connections to port 80 on the machine to port 80 on host 192.168.1.36

```
redirect -bindhost 0.0.0.0 -bindport 80 -connecthost 192.168.1.36 -connectport 80
```

![redirect_example](../images/redirect_example.png)

### Example (Mythic pivot listener)
Redirectors can be used to setup pivot listeners for other agents. This example will
setup a pivot listener on the machine at `192.168.122.54 (fedora-server)` and will spawn a new agent on
machine `192.168.122.55 (ubuntu-server)` tunneling the connection through the pivot.  

Initial egress agent on fedora-server (192.168.122.54)

![redirect_pivot_initial](../images/redirect_pivot_initial.png)

Generate a new payload but with the callback host set to the IP of the pivot machine. Port
8080 is the port the pivot listener will be listening on.

![redirect_generate_pivot](../images/redirect_generate_pivot.png)
![pivot_example_built_agents](../images/pivot_example_built_agents.png)

Setup the pivot listener on the egress agent. Redirect all connections to port 8080 on all
interfaces to port 80 of the Mythic server.

```
redirect -bindhost 0.0.0.0 -bindport 8080 -connecthost mythic.local.vm -connectport 80
```

![redirect_start_pivot](../images/redirect_start_pivot.png)
![redirect_pivot_listening](../images/redirect_pivot_listening.png)

Spawn the newly built payload with the configured pivot connection on the machine. This
example will use the [ssh-spawn](/agents/tetanus/commands/ssh-spawn/) command for
launching the agent.

![pivot_spawn_ssh](../images/pivot_spawn_ssh.png)

Mythic will build a new agent using the previously built payload as a template and spawn
it on the machine.

![pivot_uploading_agent](../images/pivot_uploading_agent.png)

The new agent is spawned on the machine

![pivot_new_agent_spawned](../images/pivot_new_agent_spawned.png)

The metadata of the new callback shows that the External IP matches the IP of the pivot
machine

![pivot_callback_metadata](../images/pivot_callback_metadata.png)

Killing the job associated with the redirect will kill the connection

## Notes
 - Machine firewall may block inbound or outbound connections

## OPSEC Considerations
 - Agent will bind to a port

## MITRE ATT&CK Mapping
  - T1090
