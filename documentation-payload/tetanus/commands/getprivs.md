+++
title = "getprivs"
chapter = false
weight = 103
hidden = true
+++

## Description
Get privileges for the current session through native functions.

## Usage
```
getprivs
```

## Windows
Gets the privileges for the current process token.

![getprivs_windows](../images/getprivs_windows.png)

## Linux
Gets user/group information. Uses libc to query the information.

![getprivs_linux](../images/getprivs_linux.png)

## Linux (SELinux)
Gets user/group information along with SELinux information. Uses libc for group
information and queries `/sys/fs/selinux` files for SELinux information.

![getprivs_selinux](../images/getprivs_selinux.png)

## MITRE ATT&CK Mapping
 - T1078
