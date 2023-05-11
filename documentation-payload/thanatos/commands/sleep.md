+++
title = "sleep"
chapter = false
weight = 103
hidden = true
+++

## Description
Change the sleep interval and jitter of the agent.

### Parameters
`interval`
 * Interval to sleep

`jitter`
 * Jitter percentage

## Usage
```
sleep [interval] [jitter]
```
```
sleep -interval [interval] -jitter [jitter]
```

### Examples
```
sleep 10
```
```
sleep 60 30
```
```
sleep -interval 10 -jitter 23
```

#### Supports unit suffixing
```
sleep 5m # 5 minutes
```
```
sleep 2m 42 # 2 minutes with a 42% jitter
```
```
sleep 1h # 1 hour
```
```
sleep -interval 30m -jitter 23 # 30 minutes with a 23% jitter
```

Defaults to seconds.

Supported units are
 * h - hours
 * m - minutes
 * s - seconds

### Popup
Command supports using the Mythic UI popup for entering parameters.

## MITRE ATT&CK Mapping
 - T1029
