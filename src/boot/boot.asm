MAGIC equ 0xe85250d6
ARCH equ 0
HEADER_LENGTH equ header_start - header_end
CHECKSUM equ 0x100000000 - (MAGIC + ARCH + HEADER_LENGTH)

extern vga_print

section .multiboot
header_start:
    dd MAGIC
    dd ARCH
    dd HEADER_LENGTH
    dd CHECKSUM

    dw 0
    dw 0
    dd 8
header_end:

global start
extern long_mode_start
section .text
bits 32
start:
    mov esp, stack_top

    mov edi, ebx

    call check_multiboot
    call check_cpuid
    call check_long_mode

    call setup_page_tables
    call enable_paging_long_mode

    lgdt [gdt64.pointer]
    jmp gdt64.code:long_mode_start

setup_page_tables:
    mov eax, p4_table
    or eax, 0b11
    mov [p4_table + 511 * 8], eax

    mov eax, p3_table
    or eax, 0b11
    mov [p4_table], eax

    mov eax, p2_table
    or eax, 0b11
    mov [p3_table], eax

    mov ecx, 0
.map_p2_table:
    mov eax, 0x200000
    mul ecx
    or eax, 0b11 | 1 << 7
    mov [p2_table + ecx * 8], eax

    inc ecx
    cmp ecx, 512
    jne .map_p2_table

    ret

enable_paging_long_mode:
    mov eax, p4_table
    mov cr3, eax

    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax

    ret

check_multiboot:
    cmp eax, 0x36d76289
    jne .no_multiboot
    ret
.no_multiboot:
    mov al, "0"
    jmp error

check_cpuid:
    pushfd
    pop eax

    mov ecx, eax
    xor eax, 1 << 21

    push eax
    popfd

    pushfd
    pop eax

    push ecx
    popfd

    cmp eax, ecx
    je .no_cpuid
    ret
.no_cpuid:
    mov al, "1"
    jmp error

check_long_mode:
    mov eax, 0x80000000
    cpuid
    cmp eax, 0x80000001
    jb .no_long_mode

    mov eax, 0x80000001
    cpuid
    test edx, 1 << 29
    jz .no_long_mode
    ret
.no_long_mode:
    mov al, "2"

error:
    mov dword [0xb8000], 0x4f524f45 ; ER
    mov dword [0xb8004], 0x4f3a4f52 ; R:
    mov dword [0xb8008], 0x4f204f20 ; [space][space]
    mov byte [0xb800a], al          ; [error code]
    hlt

section .bss
align 4096
p4_table:
    resb 4096
p3_table:
    resb 4096
p2_table:
    resb 4096
stack_bottom:
    resb 16384
stack_top:

section .rodata
gdt64:
    dq 0
.code: equ $ - gdt64
    dq 1 << 43 | 1 << 44 | 1 << 47 | 1 << 53
.pointer:
    dw $ - gdt64 - 1
    dq gdt64