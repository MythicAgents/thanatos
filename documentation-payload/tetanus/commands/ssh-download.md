+++
title = "ssh-download"
chapter = false
weight = 103
hidden = true
+++

## Description
Download a file from a remote system using scp and upload it to Mythic

### Parameters
Enter parameters through the Mythic UI with the `Download` parameter group.
```
ssh -download
```
![ssh-download_popup](../images/ssh-download_popup.png)

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

#### download path
 - Absolute path of the file on the remote system to download

### Example
![ssh-download_example_popup](../images/ssh-download_example_popup.png)

### Browser Script
The browser script from [ssh -ls](/agents/tetanus/commands/ssh-ls/) can be used to issue `ssh -download` tasking. Click the
`Download` button in the `ACTIONS` menu next to the file for download.

![ssh-download_browserscript](../images/ssh-download_browserscript.png)

### Notes
 - Key-based authentication from memory only works when the agent is on a Linux host.
 - File being downloaded never touches the host's disk when being transferred.
 - Will not spawn any shell commands for ssh connection.

## OPSEC Considerations
 - Agent will make an ssh connection to the target system.
 - Will not produce any artifacts on the host system.
 - Egress file chunk size is `512KB`.
 - Will log into a target system using SSH. OpenSSH server will log connections by
   default.

## MITRE ATT&CK Mapping
 - T1021.004
 - T1020
 - T1030
 - T1041
