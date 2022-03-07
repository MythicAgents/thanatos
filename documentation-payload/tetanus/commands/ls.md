+++
title = "ls"
chapter = false
weight = 103
hidden = true
+++

## Description
List files and directories.

### Parameters
`path`
 * Path for directory listing

## Usage
```
ls [path]
```
```
ls -path [directory]
```

### Examples
```
ls .
```
```
ls -path /tmp
```
```
ls C:\
```

#### No arguments
Entering ls with no arguments will list the current working directory.
```
ls <Enter>
```

#### Supports relative paths
```
ls ../../
ls ../foo
```

### Time
The dates and times associated with each file are local to your system's configured time.

## MITRE ATT&CK Mapping
 - T1083
 - T1106
