# PIE-Boot

为带有MMU功能的芯片设计的启动器，完成MMU初始化并将代码运行在指定地址。

## 动机

早期OS实现采用静态方式，用配置文件定义内核入口地址和加载地址，页表映射采用静态方式，将页表项定义在代码中，MMU初始化代码极简，只需要完成部分寄存器配置和设定页表地址，但不同硬件需要重新编译，人工设定加载地址。

为了实现动态加载，需要动态生成页表，需要引入一些复杂逻辑，如页表遍历、修改等，为方便调试又会引入串口驱动，设备树解析等，代码复杂度大大提升。而MMU启动前，加载地址与运行地址不一致，内核编译选项可能为no-pie，core和三方库代码无法管控，可能编译出地址相关的代码，执行到此处便会段错误（Linux 在此处只使用能生成位置无关的c代码，避免任何三方调用）。

## 设计思路

1. 实现一个Bootloader，采用`-C relocation-model=pic -Clink-args=-pie`编译选项，编译成完全位置无关的程序，在其入口处，对`rela.dyn`段进行修复，将程序重定向到正确运行地址，保证MMU开启前程序正常运行。Bootloader负责配置页表，启动MMU并重定向到虚拟地址。

2. 内核代码可采用`-no-pie`方式编译，在`.boot_loader`段存放Bootloader的bin程序，在入口处，通过汇编传递虚拟入口地址给Bootloader, 并跳转到Bootloader位置，Bootloader负责重定向到虚拟地址，跳转回虚拟入口。

3. `Build.rs`中，通过`bindeps-simple`库，在`build`阶段，编译Bootloader，在代码中，通过`include_bytes!`嵌入到`.boot_loader`段。

    ```rust
    #[unsafe(link_section = ".boot_loader")]
    pub static LOADER_BIN: [u8; LOADER_BIN_LEN] = loader_bin();
    ```
