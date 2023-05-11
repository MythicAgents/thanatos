+++
title = "cp"
chapter = false
weight = 103
hidden = true
+++

## Description
Copy a file from one location to another.

### Parameters
`source`
 * Path to the source file

`destination`
 * Path to the copy location

## Usage
```
cp [source] [destination]
```
```
cp -source [path] -dest [path]
```

### Examples
```
cp /home/user/file.txt /tmp/file.txt
```
```
cp -source C:\Users\Public\Desktop\file.txt -dest C:\Windows\Temp\file.txt
```

#### Supports relative paths
```
cp ../file.bin file.bin
cp file.txt ../../
```

#### Copies both files and directories
```
cp Documents Documents1
```

### Popup
Command supports using the Mythic UI popup for entering parameters.

## OPSEC Considerations
{{% notice info %}}
This command will manipulate files on disk.
{{% /notice %}}

## MITRE ATT&CK Mapping
 - T1570
