+++
title = "Tetanus"
chapter = true
weight = 100
+++

![logo](/agents/tetanus/images/tetanus.svg?width=400px)

## Summary
Tetanus is a general purpose Mythic C2 agent designed for both Windows and Linux targets. The
agent is written using the Rust programming language and compiles into Windows/Linux
executables, shared libraries and Windows shellcode.

## Notable Features
 - Works on both Windows and Linux targets
 - Compiles to various formats
 - Ability to spawn certain commands in separate threads
 - Port scanning
 - TCP redirection
 - SSH agent hijacking
 - Built-in SSH client
 - Directory listings through browser scripts (including SSH)
 - File download/upload from remote systems using scp
 - Configurable working hours

## Build Features

### Output Format
The agent supports compiling into various different output formats. The supported output
formats for Linux include:
 - Dynamically linked ELF executable
 - Statically linked ELF executable
 - ELF shared object

The output formats for Windows include:
 - Native Windows PE
 - Native DLL
 - Raw shellcode using donut

Each of these output formats can be built for x64 and x86 architectures.

### Encryption
The encryption used for communication is configured in the HTTP C2 profile. The two
options when configuring encryption are `Crypto type` and `Perform Key Exchange`. The
crypto type specifies whether the agent should be built with a hard coded AES key. This
key is then used when encrypting the http data for each request.  

The other option determines whether the agent should perform a key exchange during the
initial checkin. The agent will then perform a key exchange specified in the [Mythic
Documentation](https://docs.mythic-c2.net/customizing/c2-related-development/c2-profile-code/agent-side-coding/initial-checkin#encrypted-key-exchange-checkins)
for establishing a new AES key used for communication.  

When the agent is set to stage a key exchange, it will sleep a duration that is 25% of the configured
sleep interval. A configured sleep interval of 100 seconds will mean that during the
encrypted key exchange, the agent will make a C2 connection approximately every 25 seconds +/- the configured jitter.

### Checkin Retries
Tetanus allows an option for configuring the number of checkin attempts. The agent will make
these connection attempts in increasing intervals. The following formula shows how the
new sleep interval is calculated after a failed checkin attempt.  

![equation](/agents/tetanus/images/equation.svg?width=600px)

The agent will simply double the configured sleep interval after each failed check in
attempt.  
Once the agent has checked in, the sleep interval will go back to the original sleep
interval.

### Daemonize
Controls whether Linux agents should fork and run in the background. This will
automatically close the console window on Windows.

### Working hours
Specify an interval throughout the day for the agent to operate. Setting a working hours
interval of `09:00-17:00` means that the agent will only check in between 9am to 5pm
and will sleep during the off time.  
Note that the working hours interval reads from the
local time of the system it is running on. Time zone adjustment may be necessary in order to set the
correct interval.

### Linux Kernel Version
The initial check in for Linux agents will include the kernel version along with
specifying if the kernel has SELinux.

![osinfo_linux](/agents/tetanus/images/osinfo_linux.png)

![osinfo_selinux](/agents/tetanus/images/osinfo_selinux.png)

## Authors
 - [@M_alphaaa](https://twitter.com/M_alphaaa)
 - [@0xdab0](https://twitter.com/0xdab0)
