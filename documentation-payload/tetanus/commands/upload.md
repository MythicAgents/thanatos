+++
title = "upload"
chapter = false
weight = 103
hidden = true
+++

## Description
Upload a file to the remote system

### Parameters
Enter parameters through the Mythic UI.
```
upload
```
![upload_popup](../images/upload_popup.png)

##### file
 - File from the local system to upload

##### path
 - Remote path to upload the file to

### Popup
Command supports using the Mythic UI popup for entering parameters.

### Other Details
 * Progress of the upload is reflected back to the Mythic UI

## OPSEC Considerations
{{% notice info %}}
The default size to chunk files for download is `512KB`. Will write a file to disk
{{% /notice %}}

## MITRE ATT&CK Mapping
 - T1030
 - T1105
 - T1132
