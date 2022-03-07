+++
title = "ssh-rm"
chapter = false
weight = 103
hidden = true
+++

## Description
Remove a file or directory from a remote system using SSH.

### Parameters
Enter parameters through the Mythic UI with the `Remove` parameter group.
```
ssh -rm
```
![ssh-rm_popup](../images/ssh-rm_popup.png)

#### credentials
 - Credentials from the Mythic credentials database to use for authentication
   * Uses the `Account Name` from the credentials for the username
   * `plaintext` credential type will use the plain text password specified in `Credential`
   * `key` credential type will use the plain text SSH private key specified in
     `Credential`

#### use ssh agent
 - Option for whether or not to use a connected SSH agent for authentication. Tetanus will use
   the supplied username from the `credentials` parameter but replaces the other fields
   for the ssh agent.

#### host
 - Hostname or IP address of the machine to download the file from

#### port
 - Port for ssh connection

#### path
 - Absolute path of the remote file or directory to remove

### Browser Script
The browser script from `ssh -ls` can be used to task `ssh -rm` commands in the `ACTIONS`
menu through the `Delete` button.

![ssh-rm_browserscript](../images/ssh-rm_browserscript.png)

### Notes
 - Key-based authentication only works when the agent is on a Linux host.
 - Will not spawn any shell commands for ssh connection.

## Important Notes!
{{% notice info %}}
**This command will try to remove whatever path you provide it without warning. This includes
files or directories with all their children. Be cautious when removing items!**
{{% /notice %}}

## OPSEC Considerations
 - Agent will make an SSH connection to the target system.
 - Will log into a target system using SSH. OpenSSH server will log connections by
   default.

## MITRE ATT&CK Mapping
 - T1021.004
 - T1070.004
 - T1565
