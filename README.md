<p align="center">
  <img alt="Thanatos Logo" src="documentation-payload/thanatos/images/thanatos.svg" height="50%" width="50%">
</p>

# Thanatos

[![GitHub License](https://img.shields.io/github/license/MythicAgents/thanatos)](https://github.com/MythicAgents/thanatos/blob/main/LICENSE)
[![GitHub Release](https://img.shields.io/github/v/release/MythicAgents/thanatos)](https://github.com/MythicAgents/thanatos/releases/latest)
[![Release](https://github.com/MythicAgents/thanatos/workflows/Release/badge.svg)](https://github.com/MythicAgents/thanatos/actions/workflows/release.yml)

Thanatos is a Windows and Linux C2 agent written in rust.

# Installation
To install Thanatos, you will need [Mythic](https://github.com/its-a-feature/Mythic) set up on a machine.

In the Mythic root directory, use `mythic-cli` to install the agent.
```bash
sudo ./mythic-cli install github https://github.com/MythicAgents/thanatos
sudo ./mythic-cli payload start thanatos
```

Thanatos supports the http C2 profile:  
```bash
sudo ./mythic-cli install github https://github.com/MythicC2Profiles/http
sudo ./mythic-cli c2 start http
```

## Features
  - Job management
  - Built-in ssh client

## General Commands
Command     | Description
------------|------------
cat         | Output the contents of a file.
cd          | Change directory.
cp          | Copy a file from [source] to [destination].
download    | Download a file from the target system (supports relative paths).
execute     | Execute a program and return output.
exit        | Exit the agent.
getenv      | Get the current environment variables.
getprivs    | Get the privileges of the agent session.
endjob      | Shutdown a running background job.
jobs        | List currently running background jobs.
ls          | List files or directories (supports relative paths).
mkdir       | Make a new directory.
mv          | Move a file from [source] to [destination] (supports relative paths).
portscan    | Scan a list of IPs for open ports.
ps          | Get a list of currently running processes.
pwd         | Print working directory.
rm          | Remove a file or directory (supports relative paths).
reconfigure | Changes the agent's modifiable configuration values.
setenv      | Set environment variable [name] to [value].
shell       | Alias for running `execute` with a command shall.
sleep       | Set the sleep interval and jitter (supports unit suffixes).
ssh-agent   | Connect to running ssh agent sockets on the host or list identities.
ssh-creds   | Configure SSH credentials
ssh-test    | Test an SSH connection
unsetenv    | Unset an environment variable.
upload      | Upload a file to the host machine.

### Windows-specific Commands
Command     | Description
------------|------------
powershell  | Alias for `execute` but using powershell.

### SSH-enabled Commands
Commands capable of executing by interacting with a remote host through SSH
Command   |
----------|
cat       |
cp        |
download  |
execute   |
ls        |
mkdir     |
mv        |
rm        |
shell     |
upload    |

### Windows SMB-enabled Commands
Commands capable of utilizing an SMB connection from Windows
Command   |
--------  |
cat       |
cp        |
download  |
ls        |
mkdir     |
mv        |
rm        |
upload    |
