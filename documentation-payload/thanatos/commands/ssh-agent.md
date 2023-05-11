+++
title = "ssh-agent"
chapter = false
weight = 103
hidden = true
+++

## Description
Connect to running ssh agents and list identities.

### Parameters
`[-l] list`
 * Option to list identities instead of connecting or disconnecting from ssh agents

`[-c] connect`
 * Path to ssh agent socket (only needed when connecting)

`[-d] disconnect`
 * Option to disconnect from the currently connected ssh agent

## Usage
```
ssh-agent [-l] [-d] [-c <socket path>]
```

### Examples
```
ssh-agent -l
```
```
ssh-agent -c /tmp/ssh-XXXXXXbIjCH9/agent.14
```
```
ssh-agent -d
```

![ssh-agent_example](../images/ssh-agent_example.png)

Once connected to an ssh agent, the identities in that agent can then be used for
authentication through the [ssh](/agents/thanatos/commands/ssh/) and
[ssh-spawn](/agents/thanatos/commands/ssh-spawn/) commands by setting the `use ssh agent`
option to true in the task parameters.

![ssh-agent_useagent](../images/ssh-agent_useagent.png)

### Popup
Command supports using the Mythic UI popup for entering parameters. (cli is recommended over the popup)

## MITRE ATT&CK Mapping
 - T1563.001
