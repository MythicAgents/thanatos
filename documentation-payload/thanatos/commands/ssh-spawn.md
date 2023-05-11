+++
title = "ssh-spawn"
chapter = false
weight = 103
hidden = true
+++

## Description
Spawn a Mythic payload or uploaded payload on a remote system using SSH.

### Parameters
Enter parameters through the Mythic UI by entering the command itself
```
ssh-spawn
```
![ssh-spawn_popup](../images/ssh-spawn_popup.png)

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

#### payload (Mythic Payload parameter group)
 - Payload template for Mythic to build a new payload from and spawn

#### file (Mythic Upload parameter group)
 - Payload file to upload from your machine

#### path
 - Absolute path for the uploaded payload

#### exe
 - Command used to run the payload. The `{path}` variable can be used to reference the
   contents of the `path` parameter.

### Example (Mythic Payload)
Example spawning a new payload on a remote system  

Created payloads on Mythic
![ssh-spawn_example_payloads](../images/ssh-spawn_example_payloads.png)

Currently active callbacks (only one callback from `fedora-server`)
![ssh-spawn_example_callbacks1](../images/ssh-spawn_example_callbacks1.png)

Parameters to spawn the new payload
![ssh-spawn_example_popup](../images/ssh-spawn_example_popup.png)

Command was processed and Mythic is now uploading the new agent to the target
![ssh-spawn_example_uploading](../images/ssh-spawn_example_uploading.png)

New agent is now calling back from `ubuntu-server`
![ssh-spawn_example_callbacks2](../images/ssh-spawn_example_callbacks2.png)

### Example (Upload Payload)
Instead of uploading a Mythic payload to spawn, a file upload can be specified. Switch the
parameter group to `Upload Payload`.

![ssh-spawn_file_upload](../images/ssh-spawn_file_upload.png)

### Notes
 - Key-based authentication from memory only works when the agent is on a Linux host.
 - Will upload the new payload in chunks of `512KB`.
 - When the command status is `processed`, that means the payload is being uploaded to the
   target. This can take a while depending on the payload size.
 - Does not remove the payload from the target if the `exec` command fails to execute.

## OPSEC Considerations
 - Agent will make an ssh connection to the target system.
 - Will log into a target system using SSH. OpenSSH server will log connections by
   default.
 - Uploads the new agent to disk of the target.
 - Will run a shell command to spawn the new agent.
 - New agent will not be removed after it was spawned.

## MITRE ATT&CK Mapping
 - T1021.004
 - T1055
