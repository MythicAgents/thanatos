+++
title = "mv"
chapter = false
weight = 103
hidden = true
+++

## Description
Move a file from one location to another

### Parameters
`source`
 * Path to the source file

`destination`
 * Path to the move location

## Usage
```
mv [source] [destination]
```
```
mv -source [path] -dest [path]
```

### Examples
```
mv /home/user/file.txt /tmp/file.txt
```
```
mv -source file.txt -dest newfile.txt
```
```
mv C:\Users\Public\Desktop\file.txt C:\Windows\Temp\file.txt
```

#### Supports relative paths
```
mv ../file.bin file.bin
mv file.txt ../../
```

### Popup
Command supports using the Mythic UI popup for entering parameters.

## OPSEC Considerations
{{% notice info %}}
This command will manipulate files on disk.
{{% /notice %}}

## MITRE ATT&CK Mapping
 - T1106
