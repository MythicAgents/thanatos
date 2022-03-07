+++
title = "shell"
chapter = false
weight = 103
hidden = true
+++

## Description
Run a shell command using `/bin/bash -c` on Linux or `cmd.exe /c` on Windows and return
the output. This will not block the agent.

### Parameters
`command`
 * Shell command to run

## Usage
```
shell [command]
```
```
shell -command [command]
```

### Examples
```
shell id
```
```
shell -command whoami
```

### Popup
Command supports using the Mythic UI popup for entering parameters.

## OPSEC Considerations
{{% notice info %}}
This will spawn a child process which is visible in a process browser.
{{% /notice %}}

## MITRE ATT&CK Mapping
 - T1059

