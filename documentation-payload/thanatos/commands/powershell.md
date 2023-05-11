+++
title = "powershell"
chapter = false
weight = 103
hidden = true
+++

## Description
Run a powershell command using `powershell.exe /c` and return the output. This will not
block the agent.

### Parameters
`command`
 * Powershell command to run

## Usage
```
powershell [command]
```
```
powershell -command [command]
```

### Examples
```
powershell Get-ExecutionPolicy
```
```
powershell -command Get-SmbShare
```

### Popup
Command supports using the Mythic UI popup for entering parameters.

## OPSEC Considerations
{{% notice info %}}
This will spawn a child process which is visible in a process browser.
{{% /notice %}}

## MITRE ATT&CK Mapping
 - T1059
