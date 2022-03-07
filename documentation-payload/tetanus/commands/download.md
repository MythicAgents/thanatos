+++
title = "download"
chapter = false
weight = 103
hidden = true
+++

## Description
Download a file from the file system.

### Parameters
`file`
 * Path to the file to download

## Usage
```
download [file]
```
```
download -file [file]
```

### Examples
```
download /home/user/.ssh/id_rsa
```
```
download -file C:\Users\Public\Desktop\file.txt
```

#### Supports relative paths
```
download ../file.txt
```

## File browser
The file browser can be used to task downloads

### Popup
Command supports using the Mythic UI popup for entering parameters.

### Other Details
 * The download will be registered as a background task which can be viewed and killed using
the `jobs` and `jobkill` commands.

## OPSEC Considerations
{{% notice info %}}
The default size to chunk files for download is `512KB`.
{{% /notice %}}

## MITRE ATT&CK Mapping
 - T1020
 - T1030
 - T1041
