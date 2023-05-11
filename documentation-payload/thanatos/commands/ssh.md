+++
title = "SSH"
chapter = false
weight = 103
hidden = true
+++

Thanatos features a built-in SSH client. This allows the agent to make remote connections
to machines using SSH without spawning any processes. The SSH client features various
mechanisms for authentication such as ssh keys, username/password combo and interacting
with running ssh agents. More information about ssh agent auth can be found in the
[ssh-agent](/agents/thanatos/commands/ssh-agent/) command documentation.  

The commands below do not depend on an SSH client being installed on the system. All SSH
connections are handled in the agent.

## Commands
 - [SSH download](/agents/thanatos/commands/ssh-download/)
 - [SSH exec](/agents/thanatos/commands/ssh-exec/)
 - [SSH ls](/agents/thanatos/commands/ssh-ls/)
 - [SSH rm](/agents/thanatos/commands/ssh-rm/)
 - [SSH spawn](/agents/thanatos/commands/ssh-spawn/)
 - [SSH upload](/agents/thanatos/commands/ssh-upload/)

