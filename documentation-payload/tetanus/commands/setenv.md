+++
title = "setenv"
chapter = false
weight = 103
hidden = true
+++

## Description
Set an environment variable.

### Parameters
`name`
 * Name of the environment variable to set

`value`
 * Value to set the new environment variable to

## Usage
```
setenv [name] [value]
```
```
setenv -name [name] -value [value]
```

### Examples
```
setenv FOO bar
```
```
setenv -name HELLO -value world
```

### Popup
Command supports using the Mythic UI popup for entering parameters.
