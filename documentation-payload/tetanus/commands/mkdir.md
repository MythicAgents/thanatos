+++
title = "mkdir"
chapter = false
weight = 103
hidden = true
+++

## Description
Create a directory

### Parameters
`directory`
 * New directory to create

## Usage
```
mkdir [directory]
```
```
mkdir -dir [directory]
```

### Examples
```
mkdir /tmp/newdir
```
```
mkdir -dir /home/user/newdir
```
```
mkdir C:\Users\Public\Desktop\newdir
```

### Popup
Command supports using the Mythic UI popup for entering parameters.

## OPSEC Considerations
{{% notice info %}}
This command will manipulate files on disk.
{{% /notice %}}

## MITRE ATT&CK Mapping
 - T1106
