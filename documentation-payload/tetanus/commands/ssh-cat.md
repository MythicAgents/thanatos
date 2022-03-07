+++
title = "ssh-cat"
chapter = false
weight = 103
hidden = true
+++

## Description
Cat a file from a remote system using SSH.

### Parameters
Enter parameters through the Mythic UI with the `Cat` parameter group.
```
ssh -cat
```
![ssh-cat_popup](../images/ssh-cat_popup.png)

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

#### file
 - Absolute path of the remote file to cat

### Browser Script
Cat tasking can be issued through the [ssh -ls](/agents/tetanus/commands/ssh-ls/) browser
script. A `CAT` button next to file entries will issue a new [ssh -cat](/agents/tetanus/commands/ssh-cat/)
task using the path to that file and the credentials/host from that [ssh -ls](/agents/tetanus/commands/ssh-ls/) task.

![ssh-cat_browserscript](../images/ssh-cat_browserscript.png)

### Notes
 - Key-based authentication only works when the agent is on a Linux host.
 - Will not spawn any shell commands for ssh connection.

## OPSEC Considerations
 - Agent will make an ssh connection to the target system.
 - Will log into a target system using SSH. OpenSSH server will log connections by
   default.

## MITRE ATT&CK Mapping
  - T1132
  - T1030
  - T1105
