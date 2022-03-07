+++
title = "workinghours"
chapter = false
weight = 103
hidden = true
+++

## Description
Get or set the agent's working hours

### Parameters
`get`
 * Get the configured working hours

`start`
 * Set the working hours start time

`end`
 * Set the working hours end time

## Usage
```
workinghours -get
```
```
workinghours -start [HH:MM] -end [HH:MM]
```

### Examples
```
workinghours -get
```
```
workinghours -start 09:30 -end 17:05
```

### Popup
Command supports using the Mythic UI popup for entering parameters.

### Note
Use 24 hour time with two hour digits and two minute digits colon separated [HH:MM].

## MITRE ATT&CK Mapping
 - T1029
