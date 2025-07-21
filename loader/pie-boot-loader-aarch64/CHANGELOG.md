# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.25](https://github.com/rcore-os/pie-boot/compare/pie-boot-loader-aarch64-v0.1.24...pie-boot-loader-aarch64-v0.1.25) - 2025-07-08

### Other

- update Cargo.lock dependencies

## [0.1.24](https://github.com/rcore-os/pie-boot/compare/pie-boot-loader-aarch64-v0.1.23...pie-boot-loader-aarch64-v0.1.24) - 2025-07-08

### Added

- 添加对 AArch64 EL2 的支持

## [0.1.23](https://github.com/rcore-os/pie-boot/compare/pie-boot-loader-aarch64-v0.1.22...pie-boot-loader-aarch64-v0.1.23) - 2025-07-08

### Added

- add AArch64 support with new register handling and memory setup functions

## [0.1.22](https://github.com/rcore-os/pie-boot/compare/pie-boot-loader-aarch64-v0.1.21...pie-boot-loader-aarch64-v0.1.22) - 2025-07-07

### Fixed

- correct memory attribute flags in setup_table_regs function

### Other

- Merge branch 'master' of github.com:rcore-os/pie-boot

## [0.1.21](https://github.com/rcore-os/pie-boot/compare/pie-boot-loader-aarch64-v0.1.20...pie-boot-loader-aarch64-v0.1.21) - 2025-07-07

### Added

- add aarch64-cpu-ext dependency and update memory management functions

### Other

- Merge branch 'master' of github.com:rcore-os/pie-boot

## [0.1.20](https://github.com/rcore-os/pie-boot/compare/pie-boot-loader-aarch64-v0.1.19...pie-boot-loader-aarch64-v0.1.20) - 2025-07-07

### Added

- add free_memory_start to BootInfo and implement current memory retrieval

## [0.1.19](https://github.com/rcore-os/pie-boot/compare/pie-boot-loader-aarch64-v0.1.18...pie-boot-loader-aarch64-v0.1.19) - 2025-07-02

### Added

- *(cache)* add cache management functions and integrate dcache_all in entry

### Other

- Merge branch 'master' of github.com:rcore-os/pie-boot

## [0.1.18](https://github.com/rcore-os/pie-boot/compare/pie-boot-loader-aarch64-v0.1.17...pie-boot-loader-aarch64-v0.1.18) - 2025-07-01

### Added

- skip zero-sized memory regions in FDT setup

## [0.1.17](https://github.com/rcore-os/pie-boot/compare/pie-boot-loader-aarch64-v0.1.16...pie-boot-loader-aarch64-v0.1.17) - 2025-06-26

### Added

- optimize flush_tlb function to mask virtual address bits for improved TLB flush operation

## [0.1.16](https://github.com/rcore-os/pie-boot/compare/pie-boot-loader-aarch64-v0.1.15...pie-boot-loader-aarch64-v0.1.16) - 2025-06-25

### Added

- enhance enable_mmu and new_boot_table functions to accept FDT pointer

## [0.1.15](https://github.com/rcore-os/pie-boot/compare/pie-boot-loader-aarch64-v0.1.14...pie-boot-loader-aarch64-v0.1.15) - 2025-06-25

### Added

- add setup_debugcon function and enhance BootInfo structure with debug console support

## [0.1.14](https://github.com/rcore-os/pie-boot/compare/pie-boot-loader-aarch64-v0.1.13...pie-boot-loader-aarch64-v0.1.14) - 2025-06-20

### Added

- enhance Pte creation with cache handling and add NoCache variant to CacheKind

### Fixed

- change PTE cache kind to NoCache in new_boot_table function

## [0.1.13](https://github.com/rcore-os/pie-boot/compare/pie-boot-loader-aarch64-v0.1.12...pie-boot-loader-aarch64-v0.1.13) - 2025-06-20

### Added

- enhance boot information structure and debug console initialization

## [0.1.12](https://github.com/rcore-os/pie-boot/compare/pie-boot-loader-aarch64-v0.1.11...pie-boot-loader-aarch64-v0.1.12) - 2025-06-19

### Other

- boot_info
- More boot info ([#15](https://github.com/rcore-os/pie-boot/pull/15))

## [0.1.11](https://github.com/rcore-os/pie-boot/compare/pie-boot-loader-aarch64-v0.1.10...pie-boot-loader-aarch64-v0.1.11) - 2025-06-17

### Added

- enhance memory management with CacheKind and reserve area tracking

## [0.1.10](https://github.com/rcore-os/pie-boot/compare/pie-boot-loader-aarch64-v0.1.9...pie-boot-loader-aarch64-v0.1.10) - 2025-06-17

### Fixed

- pte add cache for codes

## [0.1.9](https://github.com/rcore-os/pie-boot/compare/pie-boot-loader-aarch64-v0.1.8...pie-boot-loader-aarch64-v0.1.9) - 2025-06-17

### Other

- remove unused macros and clean up console module; update BootArgs struct to include kernel image addresses

## [0.1.8](https://github.com/rcore-os/pie-boot/compare/pie-boot-loader-aarch64-v0.1.7...pie-boot-loader-aarch64-v0.1.8) - 2025-06-15

### Other

- 明确crate构建目标

## [0.1.7](https://github.com/rcore-os/pie-boot/compare/pie-boot-loader-aarch64-v0.1.6...pie-boot-loader-aarch64-v0.1.7) - 2025-06-14

### Other

- update Cargo.lock dependencies
