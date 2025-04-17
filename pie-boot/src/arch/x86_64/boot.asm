bits 32

section .bss
align 16
boot_stack_bottom:
    resb 16384
boot_stack_top:

section .text
global _start
_start:
    // set 32 bits stack
    mov esp, stack_top

    // save Multiboot
    mov edi, ebx

    call check_long_mode

    call setup_paging

    ; load gdt
    lgdt [gdt64.pointer]

    ; jump to 64bit
    jmp gdt64.code_segment:long_mode_start

check_long_mode:
    ; check cpu
    mov eax, 0x80000000
    cpuid
    cmp eax, 0x80000001
    jb .no_long_mode

    ; check LM bit in edx
    mov eax, 0x80000001
    cpuid
    test edx, 1 << 29
    jz .no_long_mode
    ret

.no_long_mode:
    mov al, 'L'
    jmp error

setup_paging:
    ; set PML4
    mov eax, pml4_table
    or eax, 0x07  ; Present + RW + User
    mov [pml4_table], eax

    ; set PDPT
    mov eax, pdpt_table
    or eax, 0x07
    mov [pml4_table + 0], eax

    ; 1gb
    mov eax, pd_table
    or eax, 0x07
    mov [pdpt_table + 0], eax

    ; 2mb 
    mov ecx, 0
    mov eax, 0x87  ; Present + RW + PS (大页)
.loop:
    mov [pd_table + ecx*8], eax
    add eax, 0x200000
    inc ecx
    cmp ecx, 512
    jne .loop

    ; enable PAE
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    ; set PML4
    mov eax, pml4_table
    mov cr3, eax

    ; enable long mode
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    ; enable paging
    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax

    ret

error:
    ; error handler
    mov dword [0xB8000], 0x4F524F45
    mov dword [0xB8004], 0x4F3A4F52
    mov dword [0xB8008], 0x4F204F20
    mov byte  [0xB800A], al
    hlt

section .bss
align 4096
pml4_table:
    resb 4096
pdpt_table:
    resb 4096
pd_table:
    resb 4096

section .rodata
gdt64:
    dq 0
.code_segment: equ $ - gdt64
    dq (1<<44) | (1<<47) | (1<<41) | (1<<43) | (1<<53) ; 64位代码段
.pointer:
    dw $ - gdt64 - 1
    dq gdt64

bits 64
long_mode_start:
    ; setup segments
    mov ax, 0
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    ; setup 64 stack
    mov rsp, stack_top

    call {entry}

    hlt