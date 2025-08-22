# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.9](https://github.com/rcore-os/somehal/compare/somehal-v0.3.7...somehal-v0.3.9) - 2025-08-19

### Added

- add cpu_on

### Fixed

- update cpu_on return type to use PsciError for consistency

### Other

- static link to .data to avoid bss clean by others

## [0.3.7](https://github.com/rcore-os/somehal/compare/somehal-v0.3.6...somehal-v0.3.7) - 2025-08-19

### Added

- 添加 LazyStatic 的 clean 方法并在 virt_entry 中调用

## [0.3.6](https://github.com/rcore-os/somehal/compare/somehal-v0.3.5...somehal-v0.3.6) - 2025-08-19

### Fixed

- use macros adr_l instead of asm macro

## [0.3.5](https://github.com/rcore-os/somehal/compare/somehal-v0.3.4...somehal-v0.3.5) - 2025-08-18

### Other

- Use prebuild loader

## [0.3.4](https://github.com/rcore-os/somehal/compare/somehal-v0.3.3...somehal-v0.3.4) - 2025-08-18

### Other

- Loader 改为动态参数 ([#35](https://github.com/rcore-os/somehal/pull/35))

## [0.3.3](https://github.com/rcore-os/somehal/compare/somehal-v0.3.2...somehal-v0.3.3) - 2025-07-24

### Other

- 在README中添加测试状态徽章

## [0.3.2](https://github.com/rcore-os/somehal/compare/somehal-v0.3.1...somehal-v0.3.2) - 2025-07-24

### Other

- release ([#31](https://github.com/rcore-os/somehal/pull/31))

## [0.3.1](https://github.com/rcore-os/somehal/compare/somehal-v0.3.0...somehal-v0.3.1) - 2025-07-22

### Other

- 更新测试配置，移除构建步骤并修正文档中的项目名称

## [0.2.20](https://github.com/rcore-os/pie-boot/compare/pie-boot-v0.2.19...pie-boot-v0.2.20) - 2025-07-08

### Other

- *(pie-boot-loader-aarch64)* release v0.1.25 ([#36](https://github.com/rcore-os/pie-boot/pull/36))

## [0.2.19](https://github.com/rcore-os/pie-boot/compare/pie-boot-v0.2.18...pie-boot-v0.2.19) - 2025-07-08

### Added

- 更新 KIMAGE_VSIZE 常量并在 Cargo.toml 中添加 kdef-pgtable/space-low 特性

### Other

- Merge branch 'master' of github.com:rcore-os/pie-boot

## [0.2.18](https://github.com/rcore-os/pie-boot/compare/pie-boot-v0.2.17...pie-boot-v0.2.18) - 2025-07-08

### Added

- 添加对 AArch64 EL2 的支持

## [0.2.17](https://github.com/rcore-os/pie-boot/compare/pie-boot-v0.2.16...pie-boot-v0.2.17) - 2025-07-08

### Added

- add AArch64 support with new register handling and memory setup functions

## [0.2.16](https://github.com/rcore-os/pie-boot/compare/pie-boot-v0.2.15...pie-boot-v0.2.16) - 2025-07-07

### Other

- updated the following local packages: pie-boot-loader-aarch64

## [0.2.15](https://github.com/rcore-os/pie-boot/compare/pie-boot-v0.2.14...pie-boot-v0.2.15) - 2025-07-07

### Other

- updated the following local packages: pie-boot-loader-aarch64

## [0.2.14](https://github.com/rcore-os/pie-boot/compare/pie-boot-v0.2.13...pie-boot-v0.2.14) - 2025-07-07

### Other

- updated the following local packages: pie-boot-if, pie-boot-loader-aarch64

## [0.2.13](https://github.com/rcore-os/pie-boot/compare/pie-boot-v0.2.12...pie-boot-v0.2.13) - 2025-07-07

### Fixed

- update function reference for memory region end calculation

### Other

- Merge branch 'master' of github.com:rcore-os/pie-boot

## [0.2.12](https://github.com/rcore-os/pie-boot/compare/pie-boot-v0.2.11...pie-boot-v0.2.12) - 2025-07-02

### Other

- updated the following local packages: pie-boot-loader-aarch64

## [0.2.11](https://github.com/rcore-os/pie-boot/compare/pie-boot-v0.2.10...pie-boot-v0.2.11) - 2025-07-01

### Added

- skip zero-sized memory regions in FDT setup

## [0.2.10](https://github.com/rcore-os/pie-boot/compare/pie-boot-v0.2.9...pie-boot-v0.2.10) - 2025-06-27

### Other

- mainmem_start_rsv merge regions

## [0.2.9](https://github.com/rcore-os/pie-boot/compare/pie-boot-v0.2.8...pie-boot-v0.2.9) - 2025-06-27

### Added

- update memory region handling to correctly classify regions as Reserved

### Other

- Merge branch 'master' of github.com:rcore-os/pie-boot

## [0.2.8](https://github.com/rcore-os/pie-boot/compare/pie-boot-v0.2.7...pie-boot-v0.2.8) - 2025-06-26

### Other

- updated the following local packages: pie-boot-loader-aarch64

## [0.2.7](https://github.com/rcore-os/pie-boot/compare/pie-boot-v0.2.6...pie-boot-v0.2.7) - 2025-06-25

### Other

- updated the following local packages: pie-boot-loader-aarch64

## [0.2.6](https://github.com/rcore-os/pie-boot/compare/pie-boot-v0.2.5...pie-boot-v0.2.6) - 2025-06-25

### Added

- add setup_debugcon function and enhance BootInfo structure with debug console support

## [0.2.5](https://github.com/rcore-os/pie-boot/compare/pie-boot-v0.2.4...pie-boot-v0.2.5) - 2025-06-20

### Other

- updated the following local packages: pie-boot-loader-aarch64

## [0.2.4](https://github.com/rcore-os/pie-boot/compare/pie-boot-v0.2.3...pie-boot-v0.2.4) - 2025-06-20

### Other

- expose KIMAGE_VADDR and KLINER_OFFSET from kdef_pgtable

## [0.2.3](https://github.com/rcore-os/pie-boot/compare/pie-boot-v0.2.2...pie-boot-v0.2.3) - 2025-06-20

### Other

- updated the following local packages: pie-boot-if, pie-boot-loader-aarch64

## [0.2.2](https://github.com/rcore-os/pie-boot/compare/pie-boot-v0.2.1...pie-boot-v0.2.2) - 2025-06-19

### Other

- api boot_info

## [0.2.0](https://github.com/rcore-os/pie-boot/compare/pie-boot-v0.1.10...pie-boot-v0.2.0) - 2025-06-17

### Other

- updated the following local packages: pie-boot-if, pie-boot-loader-aarch64

## [0.1.10](https://github.com/rcore-os/pie-boot/compare/pie-boot-v0.1.9...pie-boot-v0.1.10) - 2025-06-17

### Fixed

- update pie-boot-loader-aarch64 version to 0.1.10

## [0.1.9](https://github.com/rcore-os/pie-boot/compare/pie-boot-v0.1.8...pie-boot-v0.1.9) - 2025-06-17

### Other

- updated the following local packages: pie-boot-if

## [0.1.8](https://github.com/rcore-os/pie-boot/compare/pie-boot-v0.1.7...pie-boot-v0.1.8) - 2025-06-15

### Other

- 明确crate构建目标

## [0.1.7](https://github.com/rcore-os/pie-boot/compare/pie-boot-v0.1.6...pie-boot-v0.1.7) - 2025-06-14

### Added

- default target for pkg
