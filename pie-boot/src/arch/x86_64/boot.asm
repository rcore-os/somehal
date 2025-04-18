.code32
.section .bss
.balign 16
boot_stack_bottom:
    resb 16384
boot_stack_top:

.section .text
.global _start
_start:
    # set 32 bits stack
    mov esp, boot_stack_top

    mov     edi, eax        # arg1: magic: 0x2BADB002
    mov     esi, ebx        # arg2: multiboot info

    jmp     entry32

# 函数实现
vga_print_string:
    pusha                   # 保存所有寄存器
    mov edx, 0xB8000        # 设置 VGA 文本模式的显存地址

.print_char:
    mov al, [esi]           # 从 esi 指向的字符串中读取一个字符
    cmp al, 0               # 如果字符是 0，表示字符串结束
    je .done                # 跳转到结束

    mov ah, 0x0F            # 设置字符属性（白色字符，黑色背景）
    mov [edx], ax           # 将字符和属性写入显存
    add edx, 2              # 移动到下一个字符位置
    inc esi                 # 移动到字符串的下一个字符
    jmp .print_char         # 继续打印下一个字符

.done:
    popa                    # 恢复所有寄存器
    ret                     # 返回


.section .data
error_message db 'Error occurred!', 0


.section .text
entry32:
    mov esi, error_message
    call print_string
    hlt