+++
title = "ssh-ls"
chapter = false
weight = 103
hidden = true
+++

## Description
Grab a file listing of a remote system using SSH.

### Parameters
Enter parameters through the Mythic UI with the `List` parameter group.
```
ssh -ls
```
![ssh-ls_popup](../images/ssh-ls_popup.png)

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

#### list directory
 - Absolute path of the directory to list.


![ssh-ls_output](../images/ssh-ls_output.png)

### Browser Script
The browser script allows for additional `ssh -ls` tasking to be issued using the previously used credentials
by clicking on the `LS` button next to the directory name.

![ssh-ls_ls_tasking](../images/ssh-ls_ls_tasking.png)

The actions menu will issue new tasks to [ssh -rm](/agents/tetanus/commands/ssh-rm/) and [ssh -download](/agents/tetanus/commands/ssh-download/) for removing and downloading entries.
This will reuse the credential and host information from the previous task.

![ssh-ls_actions](../images/ssh-ls_actions.png)

An [ssh -cat](/agents/tetanus/commands/ssh-cat/) task can be issued from the browser script
for entries which are files by clicking on the `CAT` button.

![ssh-ls_cat](../images/ssh-ls_cat.png)

### Notes
 - Key-based authentication from memory only works when the agent is on a Linux host.
 - Only connects to the target system once to grab the file listing.
 - Will not spawn any shell commands on the target system.

## OPSEC Considerations
 - Agent will make an ssh connection to the target system.
 - Will log into a target system using SSH. OpenSSH server will log connections by
   default.

## MITRE ATT&CK Mapping
 - T1021.004
 - T1083
 - T1106
