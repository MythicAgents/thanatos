+++
title = "ssh-upload"
chapter = false
weight = 103
hidden = true
+++

## Description
Upload a file from your system to a remote system using scp.

### Parameters
Enter parameters through the Mythic UI with the `Upload` parameter group.
```
ssh -upload
```
![ssh-upload_popup](../images/ssh-upload_popup.png)

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
 - File to upload

#### upload path
 - Absolute path for the uploaded file

#### mode (octal)
 - Octal permissions for the uploaded file

### Example
Uploading `run.sh` to `192.168.122.55`
![ssh-upload_example_popup](../images/ssh-upload_example_popup.png)

### Notes
 - Key-based authentication from memory only works when the agent is on a Linux host.
 - File being uploaded never touches the host's disk when being transferred.
 - Only connects to the target system once to upload the file.
 - The `processed` status means the file is currently being uploaded to the agent.

## OPSEC Considerations
 - Agent will make an ssh connection to the target system.
 - Will create a file on the target system.
 - File chunk size for transfer is `512KB`.
 - Will log into a target system using SSH. OpenSSH server will log connections by
   default.

## MITRE ATT&CK Mapping
 - T1021.004
 - T1030
 - T1105
 - T1132
