+++
title = "SSH"
chapter = false
weight = 103
hidden = true
+++

Tetanus features a built-in SSH client. This allows the agent to make remote connections
to machines using SSH without spawning any processes. The SSH client features various
mechanisms for authentication such as ssh keys, username/password combo and interacting
with running ssh agents. More information about ssh agent auth can be found in the
[ssh-agent](/agents/tetanus/commands/ssh-agent/) command documentation.  

The commands below do not depend on an SSH client being installed on the system. All SSH
connections are handled in the agent.

## Commands
 - [SSH download](/agents/tetanus/commands/ssh-download/)
 - [SSH exec](/agents/tetanus/commands/ssh-exec/)
 - [SSH ls](/agents/tetanus/commands/ssh-ls/)
 - [SSH rm](/agents/tetanus/commands/ssh-rm/)
 - [SSH spawn](/agents/tetanus/commands/ssh-spawn/)
 - [SSH upload](/agents/tetanus/commands/ssh-upload/)

