# Changelog

Changes to the Thanatos project.


This changelog file adheres to [keepachangelog](https://keepachangelog.com/en/1.1.0/).

## [0.1.7] - 2024-02-02

### Changed

- Modify lint workflow to include HTTP configuration 60b34aa

### Fixed

- Fix build error due to profiles not being mut 7a29fe4
- Fix typo in sshspawn command parameter description 93e5bb3
- fix(#19): Add Thanatos artifacts into base image e3dbd69

## [0.1.6] - 2024-01-31

### Added

- `remote_images` key in config.json metadata a109d9b
- Agent capabilities metadata file `agent_capabilities.json` cc3cc48
- Github Actions workflow for running static analyzers against codebase
- Github Actions workflow for staging releases in a `release/**` branch

### Changed

- Separated out the `.gitignore` file to multiple directories

### Fixed

- Typo in the Mythic code for the `mv` command
- Removed all build warnings
- Replaced deprecated for supported ones 81dde08

## [0.1.5] - 2023-12-18

### Fixed

- Windows builds failing due to missing libssp link flag 871ce1d

## [0.1.4] - 2023-12-12

### Added

- Github actions workflow automation

## [0.1.3] - 2023-12-11

### Changed

- Time stamp manipulation from seconds to milliseconds
- Pinned python version to python 3.11
- Rewrite ls browser script

### Fixed

- ssh list of commands
- Upload command
- ps command on Linux

## [0.1.2] - 2023-11-29

## [0.1.1] - 2023-11-29

### Added

- BSD-3 License to project
- Support for Mythic v3

### Changed

- Renamed agent from Tetanus to Thanatos
- Changed deprecated chrono crate calls to supported ones

### Fixed

- Upload command not working due to Mythic changes
- Builds for 32 bit Linux
- Installed right openssl perl dependencies for Windows builds
- Bumped compiled openssl version

### Removed

- Support for building Windows shellcode. Will be back in v0.2.0
- Support for statically linked Linux 32-bit payloads due to musl/openssl limitation

## [0.1.0] - 2022-03-25

Initial public release


[0.1.7]: https://github.com/MythicAgents/thanatos/compare/v0.1.6...v0.1.7
[0.1.6]: https://github.com/MythicAgents/thanatos/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/MythicAgents/thanatos/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/MythicAgents/thanatos/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/MythicAgents/thanatos/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/MythicAgents/thanatos/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/MythicAgents/thanatos/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/MythicAgents/thanatos/releases/tag/v0.1.0
