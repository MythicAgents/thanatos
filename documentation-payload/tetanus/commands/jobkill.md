+++
title = "jobkill"
chapter = false
weight = 103
hidden = true
+++

## Description
Kill running background jobs

## Usage
```
jobkill [jobid]
```

### Popup
Command supports using the Mythic UI popup for entering parameters.

### Other Details
  - Jobs which are killed can still run in the background even after the job was signaled
    to stop.
  - More information about how a job handles being killed can be found in the command
    documentation pertaining to the running command.
