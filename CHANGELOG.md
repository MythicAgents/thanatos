# Changelog

Changes to the Thanatos project.


This changelog file adheres to [keepachangelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- Support for building the payload on offline build servers [#47](https://github.com/MythicAgents/thanatos/pull/47).

## [0.1.13] - 2025-05-18

### Added

- Configuration options for building the shared library which modify how it can be loaded [1cd04aa](https://github.com/MythicAgents/thanatos/commit/1cd04aa22203dad47ff3c02eee12b049cd2d0d1d).

#### Changed

- Update `mythic-container` python dependency to 0.4.19 [4dc6582](https://github.com/MythicAgents/thanatos/commit/4dc6582b671e9072cc1f0b07e0163a55ffb3602c).
- Refactor `portscan.rs` to satisfy clippy lints [e1a16d0](https://github.com/MythicAgents/thanatos/commit/e1a16d0fe2c2596df819f7020e80c52cb0bf94db).

## [0.1.12] - 2025-05-18

### Fixed

- Issue where the agent will exit after performing the initial checkin [51e1554](https://github.com/MythicAgents/thanatos/commit/51e1554e7e6b9c96daf5a3bc44e44f5c714e17b0).

## [0.1.11] - 2024-12-19

### Fixed

- Fixes netstat command to include the browser script [7ef0969](https://github.com/MythicAgents/thanatos/commit/7ef09696abc939a773d49bef28a8dabb6dd7d9d2).

## [0.1.10] - 2024-12-19

### Added

- `netstat` command for returning active network connections authored by [@maclarel](https://github.com/maclarel) added in [#40](https://github.com/MythicAgents/thanatos/pull/40).

### Changed

- Deprecated functions from the [chrono](https://crates.io/crates/chrono) crate were removed and replaced with supported versions [#42](https://github.com/MythicAgents/thanatos/pull/42).
- Miscellaneous project source refactoring to cleanup compile and lint warnings [#43](https://github.com/MythicAgents/thanatos/pull/43).

## [0.1.9] - 2024-03-17

### Added

- add changelogtool.py [48c6b7b](https://github.com/MythicAgents/thanatos/commit/48c6b7b40469eed3ed334dc7f7e527f264329245)

### Changed

- **releases:** add release automation [3ac7905](https://github.com/MythicAgents/thanatos/commit/3ac79057d7a5f3374c43ce64d171ee96f88d4c88)
- **ci:** add container build action [d38aa47](https://github.com/MythicAgents/thanatos/commit/d38aa47b6833a981e907ce420dcd12194448474b)
- **ci:** remove main from lint checks [be67690](https://github.com/MythicAgents/thanatos/commit/be676906537cce5d854ac376368db7d96cc7da61)

### Removed

- **ci:** remove base image build composite workflow [86815f4](https://github.com/MythicAgents/thanatos/commit/86815f412766995608148e3beacc6235703ba524)

## [0.1.8] - 2024-02-06

### Added

- Scripts for running CI lint tests locally
- Path triggers for release workflow

### Changed

- Removed `libssp-0.dll` runtime dependency for Windows agents
- Set line length for Mythic code CI checks to 90 columns
- Bumped docker container Fedora version to Fedora 39

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

[unreleased]: https://github.com/MythicAgents/thanatos/compare/v0.1.13...HEAD
[0.1.13]: https://github.com/MythicAgents/thanatos/compare/v0.1.12...v0.1.13
[0.1.12]: https://github.com/MythicAgents/thanatos/compare/v0.1.11...v0.1.12
[0.1.11]: https://github.com/MythicAgents/thanatos/compare/v0.1.10...v0.1.11
[0.1.10]: https://github.com/MythicAgents/thanatos/compare/v0.1.9...v0.1.10
[0.1.9]: https://github.com/MythicAgents/thanatos/compare/v0.1.8...v0.1.9
[0.1.8]: https://github.com/MythicAgents/thanatos/compare/v0.1.7...v0.1.8
[0.1.7]: https://github.com/MythicAgents/thanatos/compare/v0.1.6...v0.1.7
[0.1.6]: https://github.com/MythicAgents/thanatos/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/MythicAgents/thanatos/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/MythicAgents/thanatos/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/MythicAgents/thanatos/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/MythicAgents/thanatos/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/MythicAgents/thanatos/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/MythicAgents/thanatos/releases/tag/v0.1.0
