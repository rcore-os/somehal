# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.0](https://github.com/rcore-os/pie-boot/compare/pie-boot-if-v0.5.0...pie-boot-if-v0.6.0) - 2025-07-07

### Added

- add free_memory_start to BootInfo and implement current memory retrieval

## [0.5.0](https://github.com/rcore-os/pie-boot/compare/pie-boot-if-v0.4.1...pie-boot-if-v0.5.0) - 2025-06-25

### Added

- add setup_debugcon function and enhance BootInfo structure with debug console support

## [0.4.1](https://github.com/rcore-os/pie-boot/compare/pie-boot-if-v0.4.0...pie-boot-if-v0.4.1) - 2025-06-20

### Added

- enhance boot information structure and debug console initialization

## [0.4.0](https://github.com/rcore-os/pie-boot/compare/pie-boot-if-v0.3.0...pie-boot-if-v0.4.0) - 2025-06-19

### Other

- More boot info ([#15](https://github.com/rcore-os/pie-boot/pull/15))

## [0.3.0](https://github.com/rcore-os/pie-boot/compare/pie-boot-if-v0.2.0...pie-boot-if-v0.3.0) - 2025-06-17

### Added

- enhance memory management with CacheKind and reserve area tracking

## [0.2.0](https://github.com/rcore-os/pie-boot/compare/pie-boot-if-v0.1.1...pie-boot-if-v0.2.0) - 2025-06-17

### Other

- remove unused macros and clean up console module; update BootArgs struct to include kernel image addresses
