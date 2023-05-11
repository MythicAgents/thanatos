+++
title = "ssh-exec"
chapter = false
weight = 103
hidden = true
+++

## Description
Run a shell command on a target system using SSH.

### Parameters
Enter parameters through the Mythic UI with the `Execute` parameter group.
```
ssh -exec
```
![ssh-exec_popup](../images/ssh-exec_popup.png)

#### credentials
 - Credentials from the Mythic credentials database to use for authentication
   * Uses the `Account Name` from the credentials for the username
   * `plaintext` credential type will use the plain text password specified in `Credential`
   * `key` credential type will use the plain text SSH private key specified in
     `Credential`

#### use ssh agent
 - Option for whether or not to use a connected SSH agent for authentication. Thanatos will use
   the supplied username from the `credentials` parameter but replaces the other fields
   for the ssh agent.

#### host
 - Hostname or IP address of the machine to download the file from

#### port
 - Port for ssh connection

#### exec
 - Command to execute

### Notes
 - Key-based authentication from memory only works when the agent is on a Linux host.

## OPSEC Considerations
{{% notice info %}}
This will log into a target system and run a command. OpenSSH server will log connections by default. Will spawn a process on the target system.
{{% /notice %}}

## MITRE ATT&CK Mapping
  - T1021.004
  - T1059
