# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.9](https://github.com/rcore-os/somehal/compare/somehal-v0.4.8...somehal-v0.4.9) - 2026-02-09

### Added

- merge overlaps region and delete reserved/bootloader from ram region ([#56](https://github.com/rcore-os/somehal/pull/56))
- implement UART RX handling and add read functions, simplify RAM finding logic ([#53](https://github.com/rcore-os/somehal/pull/53))
- 添加 write_bytes 函数以支持批量写入字节
- add debug
- 添加 spin 依赖并实现共享数据的互斥锁管理 ([#50](https://github.com/rcore-os/somehal/pull/50))
- add cpu_on
- 添加 LazyStatic 的 clean 方法并在 virt_entry 中调用

### Fixed

- support hypervisor mode EL2 boot and refactor CI ([#55](https://github.com/rcore-os/somehal/pull/55))
- static section
- 更新版本号至 0.3.15
- rsv aligin
- cpu_on
- cpu on cache
- cpu_on
- 修复内核表地址设置逻辑并更新版本号至 0.3.11
- update cpu_on return type to use PsciError for consistency
- use macros adr_l instead of asm macro
- fix
- fix log
- percpu data
- find memory
- fix aarch64 cpu on cache
- fix link before mmu
- fix riscv cpuid
- fix pg blk ([#2](https://github.com/rcore-os/somehal/pull/2))
- fix section

### Other

- release ([#59](https://github.com/rcore-os/somehal/pull/59))
- release ([#58](https://github.com/rcore-os/somehal/pull/58))
- *(somehal)* release v0.4.6 ([#57](https://github.com/rcore-os/somehal/pull/57))
- *(somehal)* release v0.4.5 ([#54](https://github.com/rcore-os/somehal/pull/54))
- release ([#51](https://github.com/rcore-os/somehal/pull/51))
- aarch64 bootloader stack use kernel stack
- release ([#46](https://github.com/rcore-os/somehal/pull/46))
- Implement a generic page table structure and associated traits
- 更新版本号至 0.3.9
- *(somehal)* release v0.3.8 ([#42](https://github.com/rcore-os/somehal/pull/42))
- static link to .data to avoid bss clean by others
- release ([#41](https://github.com/rcore-os/somehal/pull/41))
- release ([#39](https://github.com/rcore-os/somehal/pull/39))
- release ([#37](https://github.com/rcore-os/somehal/pull/37))
- 添加对 pie-boot-loader-aarch64 的支持，更新依赖项，新增 Gitee 和 GitHub 的发布获取功能
- Use prebuild loader
- *(somehal)* release v0.3.4 ([#36](https://github.com/rcore-os/somehal/pull/36))
- Loader 改为动态参数 ([#35](https://github.com/rcore-os/somehal/pull/35))
- *(somehal)* release v0.3.3 ([#33](https://github.com/rcore-os/somehal/pull/33))
- 在README中添加测试状态徽章
- *(somehal)* release v0.3.2 ([#32](https://github.com/rcore-os/somehal/pull/32))
- release ([#31](https://github.com/rcore-os/somehal/pull/31))
- 简化trap
- add irq handler
- release ([#30](https://github.com/rcore-os/somehal/pull/30))
- 更新测试配置，移除构建步骤并修正文档中的项目名称
- V03 ([#29](https://github.com/rcore-os/somehal/pull/29))
- Make fdt_ptr() function public ([#28](https://github.com/rcore-os/somehal/pull/28))
- 更新 Rust 分析器配置，启用 "vm" 特性；优化 EL2 切换函数，改用写入方式修改 HCR_EL2；在页表配置中添加共享标志并处理缓存配置。
- log
- update
- update rdrive
- use new rdrive if
- update rdrive
- new rdrive macros
- add rx
- update rdrive
- update
- update
- 优化串口输出
- update
- add el2 clk
- update rdrive
- update rdrive
- update
- 移除percpu依赖 ([#27](https://github.com/rcore-os/somehal/pull/27))
- 优化串口输出
- update
- update
- console out add \r
- adapt percpu feature
- adapt percpu
- adapt percpu
- 25 el2 support ([#26](https://github.com/rcore-os/somehal/pull/26))
- riscv boot
- update
- update
- update
- update
- percpu smp ([#24](https://github.com/rcore-os/somehal/pull/24))
- 2 cpu on  ([#23](https://github.com/rcore-os/somehal/pull/23))
- update
- update
- update ([#22](https://github.com/rcore-os/somehal/pull/22))
- update
- fmt
- update
- update
- update
- update
- update
- update
- update
- 调整 rdrive 链接位置
- update
- update
- update
- update
- update
- update
- update
- update
- power
- update
- update
- add rdrive
- 优化ld
- Merge branch 'main' into 20-引入动态驱动框架实现中断控制器和定时器枚举
- [fix] 树莓派
- update
- fmt
- x86 use any uart
- 18 优化链接脚本便于引入 ([#19](https://github.com/rcore-os/somehal/pull/19))
- 添加x86-64支持 ([#17](https://github.com/rcore-os/somehal/pull/17))
- update
- [fix]add clean bss
- cpu and memory found
- update
- update
- update
- update
- main memory
- rsv
- update
- x86 cpu list
- 11 项目结构优化 ([#12](https://github.com/rcore-os/somehal/pull/12))
- 增加主存保留信息分配器及优化代码 ([#10](https://github.com/rcore-os/somehal/pull/10))
- [fix] starfive2 support ([#9](https://github.com/rcore-os/somehal/pull/9))
- update
- sv39 support
- Move mmu relocate to pie-boot ([#8](https://github.com/rcore-os/somehal/pull/8))
- add dcache flush
- update
- fmt
- update
- fmt
- percpu data ok
- update
- entry
- 增加Riscv支持 ([#3](https://github.com/rcore-os/somehal/pull/3))
- update
- fmt
- fmt
- update
- add riscv
- update
- update
- update
- update
- update
- update
- update
- update
- fmt code
- update
- update
- mmu ok
- init
- update
- fmt
- fmt
- update
- fmt code
- updte
- update
- qemu debug
- update
- print
- update
- pte
- debug
- uboot debug ok
- update
- update
- update
- update
- u
- update
- update
- update
- 内存区域划分
- update
- update
- page table
- update
- update
- update
- update
- link to boot
- update
- update
- update
- debug ok
- debug ok
- update
- update
- update
- init

## [0.4.8](https://github.com/rcore-os/somehal/compare/somehal-v0.4.7...somehal-v0.4.8) - 2026-02-09

### Other

- release ([#58](https://github.com/rcore-os/somehal/pull/58))
- release ([#51](https://github.com/rcore-os/somehal/pull/51))
- aarch64 bootloader stack use kernel stack
- release ([#46](https://github.com/rcore-os/somehal/pull/46))
- Implement a generic page table structure and associated traits
- 更新版本号至 0.3.9
- *(somehal)* release v0.3.8 ([#42](https://github.com/rcore-os/somehal/pull/42))
- static link to .data to avoid bss clean by others
- release ([#41](https://github.com/rcore-os/somehal/pull/41))
- release ([#39](https://github.com/rcore-os/somehal/pull/39))
- release ([#37](https://github.com/rcore-os/somehal/pull/37))
- 添加对 pie-boot-loader-aarch64 的支持，更新依赖项，新增 Gitee 和 GitHub 的发布获取功能
- Use prebuild loader
- *(somehal)* release v0.3.4 ([#36](https://github.com/rcore-os/somehal/pull/36))
- Loader 改为动态参数 ([#35](https://github.com/rcore-os/somehal/pull/35))
- *(somehal)* release v0.3.3 ([#33](https://github.com/rcore-os/somehal/pull/33))
- 在README中添加测试状态徽章
- *(somehal)* release v0.3.2 ([#32](https://github.com/rcore-os/somehal/pull/32))
- release ([#31](https://github.com/rcore-os/somehal/pull/31))
- 简化trap
- add irq handler
- release ([#30](https://github.com/rcore-os/somehal/pull/30))
- 更新测试配置，移除构建步骤并修正文档中的项目名称
- V03 ([#29](https://github.com/rcore-os/somehal/pull/29))
- Make fdt_ptr() function public ([#28](https://github.com/rcore-os/somehal/pull/28))
- 更新 Rust 分析器配置，启用 "vm" 特性；优化 EL2 切换函数，改用写入方式修改 HCR_EL2；在页表配置中添加共享标志并处理缓存配置。
- log
- update
- update rdrive
- use new rdrive if
- update rdrive
- new rdrive macros
- add rx
- update rdrive
- update
- update
- 优化串口输出
- update
- add el2 clk
- update rdrive
- update rdrive
- update
- 移除percpu依赖 ([#27](https://github.com/rcore-os/somehal/pull/27))
- 优化串口输出
- update
- update
- console out add \r
- adapt percpu feature
- adapt percpu
- adapt percpu
- 25 el2 support ([#26](https://github.com/rcore-os/somehal/pull/26))
- riscv boot
- update
- update
- update
- update
- percpu smp ([#24](https://github.com/rcore-os/somehal/pull/24))
- 2 cpu on  ([#23](https://github.com/rcore-os/somehal/pull/23))
- update
- update
- update ([#22](https://github.com/rcore-os/somehal/pull/22))
- update
- fmt
- update
- update
- update
- update
- update
- update
- update
- 调整 rdrive 链接位置
- update
- update
- update
- update
- update
- update
- update
- update
- power
- update
- update
- add rdrive
- 优化ld
- Merge branch 'main' into 20-引入动态驱动框架实现中断控制器和定时器枚举
- [fix] 树莓派
- update
- fmt
- x86 use any uart
- 18 优化链接脚本便于引入 ([#19](https://github.com/rcore-os/somehal/pull/19))
- 添加x86-64支持 ([#17](https://github.com/rcore-os/somehal/pull/17))
- update
- [fix]add clean bss
- cpu and memory found
- update
- update
- update
- update
- main memory
- rsv
- update
- x86 cpu list
- 11 项目结构优化 ([#12](https://github.com/rcore-os/somehal/pull/12))
- 增加主存保留信息分配器及优化代码 ([#10](https://github.com/rcore-os/somehal/pull/10))
- [fix] starfive2 support ([#9](https://github.com/rcore-os/somehal/pull/9))
- update
- sv39 support
- Move mmu relocate to pie-boot ([#8](https://github.com/rcore-os/somehal/pull/8))
- add dcache flush
- update
- fmt
- update
- fmt
- percpu data ok
- update
- entry
- 增加Riscv支持 ([#3](https://github.com/rcore-os/somehal/pull/3))
- update
- fmt
- fmt
- update
- add riscv
- update
- update
- update
- update
- update
- update
- update
- update
- fmt code
- update
- update
- mmu ok
- init
- update
- fmt
- fmt
- update
- fmt code
- updte
- update
- qemu debug
- update
- print
- update
- pte
- debug
- uboot debug ok
- update
- update
- update
- update
- u
- update
- update
- update
- 内存区域划分
- update
- update
- page table
- update
- update
- update
- update
- link to boot
- update
- update
- update
- debug ok
- debug ok
- update
- update
- update
- init

## [0.4.7](https://github.com/rcore-os/somehal/compare/somehal-v0.4.6...somehal-v0.4.7) - 2026-02-09

### Added

- merge overlaps region and delete reserved/bootloader from ram region ([#56](https://github.com/rcore-os/somehal/pull/56))
- implement UART RX handling and add read functions, simplify RAM finding logic ([#53](https://github.com/rcore-os/somehal/pull/53))
- 添加 write_bytes 函数以支持批量写入字节
- add debug
- 添加 spin 依赖并实现共享数据的互斥锁管理 ([#50](https://github.com/rcore-os/somehal/pull/50))
- add cpu_on
- 添加 LazyStatic 的 clean 方法并在 virt_entry 中调用

### Fixed

- support hypervisor mode EL2 boot and refactor CI ([#55](https://github.com/rcore-os/somehal/pull/55))
- static section
- 更新版本号至 0.3.15
- rsv aligin
- cpu_on
- cpu on cache
- cpu_on
- 修复内核表地址设置逻辑并更新版本号至 0.3.11
- update cpu_on return type to use PsciError for consistency
- use macros adr_l instead of asm macro
- fix
- fix log
- percpu data
- find memory
- fix aarch64 cpu on cache
- fix link before mmu
- fix riscv cpuid
- fix pg blk ([#2](https://github.com/rcore-os/somehal/pull/2))
- fix section

### Other

- *(somehal)* release v0.4.6 ([#57](https://github.com/rcore-os/somehal/pull/57))
- *(somehal)* release v0.4.5 ([#54](https://github.com/rcore-os/somehal/pull/54))
- release ([#51](https://github.com/rcore-os/somehal/pull/51))
- aarch64 bootloader stack use kernel stack
- release ([#46](https://github.com/rcore-os/somehal/pull/46))
- Implement a generic page table structure and associated traits
- 更新版本号至 0.3.9
- *(somehal)* release v0.3.8 ([#42](https://github.com/rcore-os/somehal/pull/42))
- static link to .data to avoid bss clean by others
- release ([#41](https://github.com/rcore-os/somehal/pull/41))
- release ([#39](https://github.com/rcore-os/somehal/pull/39))
- release ([#37](https://github.com/rcore-os/somehal/pull/37))
- 添加对 pie-boot-loader-aarch64 的支持，更新依赖项，新增 Gitee 和 GitHub 的发布获取功能
- Use prebuild loader
- *(somehal)* release v0.3.4 ([#36](https://github.com/rcore-os/somehal/pull/36))
- Loader 改为动态参数 ([#35](https://github.com/rcore-os/somehal/pull/35))
- *(somehal)* release v0.3.3 ([#33](https://github.com/rcore-os/somehal/pull/33))
- 在README中添加测试状态徽章
- *(somehal)* release v0.3.2 ([#32](https://github.com/rcore-os/somehal/pull/32))
- release ([#31](https://github.com/rcore-os/somehal/pull/31))
- 简化trap
- add irq handler
- release ([#30](https://github.com/rcore-os/somehal/pull/30))
- 更新测试配置，移除构建步骤并修正文档中的项目名称
- V03 ([#29](https://github.com/rcore-os/somehal/pull/29))
- Make fdt_ptr() function public ([#28](https://github.com/rcore-os/somehal/pull/28))
- 更新 Rust 分析器配置，启用 "vm" 特性；优化 EL2 切换函数，改用写入方式修改 HCR_EL2；在页表配置中添加共享标志并处理缓存配置。
- log
- update
- update rdrive
- use new rdrive if
- update rdrive
- new rdrive macros
- add rx
- update rdrive
- update
- update
- 优化串口输出
- update
- add el2 clk
- update rdrive
- update rdrive
- update
- 移除percpu依赖 ([#27](https://github.com/rcore-os/somehal/pull/27))
- 优化串口输出
- update
- update
- console out add \r
- adapt percpu feature
- adapt percpu
- adapt percpu
- 25 el2 support ([#26](https://github.com/rcore-os/somehal/pull/26))
- riscv boot
- update
- update
- update
- update
- percpu smp ([#24](https://github.com/rcore-os/somehal/pull/24))
- 2 cpu on  ([#23](https://github.com/rcore-os/somehal/pull/23))
- update
- update
- update ([#22](https://github.com/rcore-os/somehal/pull/22))
- update
- fmt
- update
- update
- update
- update
- update
- update
- update
- 调整 rdrive 链接位置
- update
- update
- update
- update
- update
- update
- update
- update
- power
- update
- update
- add rdrive
- 优化ld
- Merge branch 'main' into 20-引入动态驱动框架实现中断控制器和定时器枚举
- [fix] 树莓派
- update
- fmt
- x86 use any uart
- 18 优化链接脚本便于引入 ([#19](https://github.com/rcore-os/somehal/pull/19))
- 添加x86-64支持 ([#17](https://github.com/rcore-os/somehal/pull/17))
- update
- [fix]add clean bss
- cpu and memory found
- update
- update
- update
- update
- main memory
- rsv
- update
- x86 cpu list
- 11 项目结构优化 ([#12](https://github.com/rcore-os/somehal/pull/12))
- 增加主存保留信息分配器及优化代码 ([#10](https://github.com/rcore-os/somehal/pull/10))
- [fix] starfive2 support ([#9](https://github.com/rcore-os/somehal/pull/9))
- update
- sv39 support
- Move mmu relocate to pie-boot ([#8](https://github.com/rcore-os/somehal/pull/8))
- add dcache flush
- update
- fmt
- update
- fmt
- percpu data ok
- update
- entry
- 增加Riscv支持 ([#3](https://github.com/rcore-os/somehal/pull/3))
- update
- fmt
- fmt
- update
- add riscv
- update
- update
- update
- update
- update
- update
- update
- update
- fmt code
- update
- update
- mmu ok
- init
- update
- fmt
- fmt
- update
- fmt code
- updte
- update
- qemu debug
- update
- print
- update
- pte
- debug
- uboot debug ok
- update
- update
- update
- update
- u
- update
- update
- update
- 内存区域划分
- update
- update
- page table
- update
- update
- update
- update
- link to boot
- update
- update
- update
- debug ok
- debug ok
- update
- update
- update
- init

## [0.4.6](https://github.com/rcore-os/somehal/compare/somehal-v0.4.5...somehal-v0.4.6) - 2026-02-09

### Added

- merge overlaps region and delete reserved/bootloader from ram region ([#56](https://github.com/rcore-os/somehal/pull/56))

### Fixed

- support hypervisor mode EL2 boot and refactor CI ([#55](https://github.com/rcore-os/somehal/pull/55))

## [0.4.5](https://github.com/rcore-os/somehal/compare/somehal-v0.4.4...somehal-v0.4.5) - 2025-11-25

### Added

- implement UART RX handling and add read functions, simplify RAM finding logic ([#53](https://github.com/rcore-os/somehal/pull/53))

## [0.4.2](https://github.com/rcore-os/somehal/compare/somehal-v0.4.1...somehal-v0.4.2) - 2025-09-22

### Added

- 添加 spin 依赖并实现共享数据的互斥锁管理 ([#50](https://github.com/rcore-os/somehal/pull/50))

## [0.4.1](https://github.com/rcore-os/somehal/compare/somehal-v0.3.12...somehal-v0.4.1) - 2025-09-11

### Fixed

- fix: static stack

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
