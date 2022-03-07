+++
title = "rm"
chapter = false
weight = 103
hidden = true
+++

## Description
Remove a file or directory from the host's file system.

### Parameters
`path`
 * Path of the file or directory to remove

## Usage
```
rm [path]
```
```
rm -path [path]
```

### Examples
```
rm file.txt
```
```
rm -path /tmp/file.txt
```

#### Removes files or directories recursively
```
rm Downloads
```

#### Supports relative paths
```
rm ../../file.txt
```

### Popup
Command supports using the Mythic UI popup for entering parameters.

## Important Notes!
{{% notice info %}}
**This command will try to remove whatever path you provide it without warning. This includes
files or directories with all their children. Be cautious when removing items!**
{{% /notice %}}

## OPSEC Considerations
{{% notice info %}}
This command will remove a file or directory from disk.
{{% /notice %}}

## MITRE ATT&CK Mapping
 - T1070.004
 - T1565
