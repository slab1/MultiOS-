; MultiOS x86_64 Multiboot2 Bootloader
; This code implements Multiboot2 compliance and prepares for long mode transition

section .multiboot
align 8

; Multiboot2 header
multiboot2_header:
    .magic:    dd 0xe85250d6          ; Multiboot2 magic number
    .arch:     dd 0                   ; Architecture: i386
    .length:   dd multiboot2_header_end - multiboot2_header
    .checksum: dd 0 - (0xe85250d6 + 0 + (multiboot2_header_end - multiboot2_header))

    ; Boot information request tag
    .boot_info_tag:
        dw 1                         ; Type: boot information request
        dw 0                         ; Flags
        dd .boot_info_tag_end - .boot_info_tag
        dd 0x00000009                ; Request: memory map
        dd 0x00000002                ; Request: ELF symbols
        dd 0x0000000b                ; Request: framebuffer
        dd 0x00000007                ; Request: module list

    ; Console entry tag
    .console_tag:
        dw 2                         ; Type: console entry
        dw 0                         ; Flags
        dd .console_tag_end - .console_tag
    .console_tag_end:

    ; Framebuffer tag (minimal)
    .fb_tag:
        dw 5                         ; Type: framebuffer
        dw 0                         ; Flags
        dd .fb_tag_end - .fb_tag
        dd 0                         ; Width (0 = auto)
        dd 0                         ; Height (0 = auto)
        dd 32                        ; Depth (32 bits)
    .fb_tag_end:

    ; Entry address tag
    .entry_tag:
        dw 4                         ; Type: entry address
        dw 0                         ; Flags
        dd .entry_tag_end - .entry_tag
        dd entry_point               ; Entry point address
    .entry_tag_end:

    ; Terminator tag
    .terminator:
        dw 0                         ; Type: end tag
        dw 0                         ; Flags
        dd 8                         ; Length
    .terminator_end:
        dd 0

multiboot2_header_end:

section .text
global _start
global entry_point
extern boot_main              ; Rust boot main function

entry_point:
    ; Save multiboot2 information
    mov eax, ebx                ; EBX contains pointer to multiboot2 information
    push eax                    ; Save for Rust boot_main
    call boot_main              ; Call Rust boot main

    ; This should never return
.loop:
    hlt
    jmp .loop

; Long mode initialization sequence
boot_main:
    ; Initialize stack
    mov esp, stack_top
    
    ; Check if CPU supports long mode
    call check_cpuid
    test eax, eax
    jz .no_longmode
    
    ; Check if CPU supports 64-bit mode
    call check_longmode
    test eax, eax
    jz .no_64bit
    
    ; Load GDT for long mode
    lgdt [gdt_descriptor]
    
    ; Enable PAE (Physical Address Extension)
    mov eax, cr4
    or eax, 1 << 5              ; Bit 5: PAE enable
    mov cr4, eax
    
    ; Set up page tables for long mode
    call setup_page_tables
    
    ; Enable paging
    mov eax, cr4
    or eax, 1 << 7              ; Bit 7: PGE enable
    mov cr4, eax
    
    ; Set CR3 to page table directory
    mov eax, page_table_l4
    mov cr3, eax
    
    ; Enable long mode
    mov ecx, 0xC0000080         ; EFER MSR
    rdmsr
    or eax, 1 << 0              ; Bit 0: LME (Long Mode Enable)
    wrmsr
    
    ; Enable protected mode
    mov eax, cr0
    or eax, 1 << 0              ; Bit 0: PE (Protected Enable)
    mov cr0, eax
    
    ; Far jump to enable long mode
    mov eax, 0x08               ; Code segment selector
    push eax
    push long_mode_entry
    retf

.no_longmode:
    mov eax, 1                  ; Error: CPUID not supported
    jmp boot_error

.no_64bit:
    mov eax, 2                  ; Error: 64-bit mode not supported
    jmp boot_error

boot_error:
    ; Print error message
    mov edi, error_msg
    mov al, 0
.loop:
    stosb
    cmp al, 0
    jne .loop
    
    ; Halt system
    hlt
    jmp .boot_error

; Check CPUID support
check_cpuid:
    pushfd
    pushfd
    pop eax
    mov ecx, eax
    xor eax, 0x200000           ; Toggle ID bit
    push eax
    popfd
    pushfd
    pop eax
    popfd
    cmp eax, ecx
    jz .no_cpuid                ; CPUID not supported
    mov eax, 1                  ; CPUID supported
    ret
.no_cpuid:
    xor eax, eax                ; CPUID not supported
    ret

; Check for 64-bit mode support
check_longmode:
    push ebx
    
    ; Check CPUID leaf 0x80000001
    mov eax, 0x80000001
    cpuid
    test edx, 1 << 29           ; Bit 29: LM (Long Mode)
    jz .no_longmode64
    
    mov eax, 1                  ; 64-bit mode supported
    pop ebx
    ret
    
.no_longmode64:
    xor eax, eax                ; 64-bit mode not supported
    pop ebx
    ret

; Setup page tables for long mode
setup_page_tables:
    ; Clear page tables
    mov edi, page_table_l4
    xor eax, eax
    mov ecx, 4096 * 4           ; 4 pages
    rep stosd
    
    ; Setup PML4 (Page Map Level 4)
    mov eax, page_table_l3
    or eax, 3                   ; Present, Read/Write
    mov [page_table_l4], eax
    
    ; Setup PDP (Page Directory Pointer)
    mov eax, page_table_l2
    or eax, 3                   ; Present, Read/Write
    mov [page_table_l3], eax
    
    ; Setup PD (Page Directory)
    mov eax, page_table_l1
    or eax, 3                   ; Present, Read/Write
    mov [page_table_l2], eax
    
    ; Setup page tables for identity mapping of first 1GB
    mov eax, 3                  ; Present, Read/Write
    mov ecx, 512                ; 512 entries for 1GB
    mov edi, page_table_l1
    
.loop:
    stosd
    add eax, 0x1000
    loop .loop
    
    ret

; 64-bit long mode entry point
long_mode_entry:
    ; Set up segment registers
    mov ax, 0x10                ; Data segment selector
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    
    ; Clear extended registers
    xor rax, rax
    xor rbx, rbx
    xor rcx, rcx
    xor rdx, rdx
    xor rsi, rsi
    xor rdi, rdi
    xor rbp, rbp
    xor r8, r8
    xor r9, r9
    xor r10, r10
    xor r11, r11
    xor r12, r12
    xor r13, r13
    xor r14, r14
    xor r15, r15
    
    ; Call boot_main for 64-bit
    call boot_main_64bit
    
    ; Halt system if we return
    hlt
    jmp $

; Boot main for 64-bit mode
extern kernel_main_64bit
extern create_multiboot2_info

boot_main_64bit:
    ; Load multiboot2 information from stack
    pop rdi                     ; RDI = multiboot2 info pointer
    
    ; Create kernel boot information
    call create_multiboot2_info
    
    ; Jump to kernel
    call kernel_main_64bit
    
    ; Should never return
    jmp $

; Global Descriptor Table
section .rodata
gdt_start:
    ; Null descriptor
    dq 0x0000000000000000
    
    ; Code segment descriptor
    dq 0x0020980000000000       ; Code segment (64-bit, DPL=0)
    
    ; Data segment descriptor
    dq 0x0000920000000000       ; Data segment (64-bit, DPL=0)
    
gdt_end:

gdt_descriptor:
    dw gdt_end - gdt_start - 1 ; Limit
    dq gdt_start               ; Base

; Error message
error_msg: db "Boot error: CPU does not support 64-bit mode", 0

section .bss
align 16
stack_bottom:
    resb 16384                 ; 16KB stack
stack_top:

; Page tables (1 page each)
align 4096
page_table_l4:
    resb 4096
page_table_l3:
    resb 4096
page_table_l2:
    resb 4096
page_table_l1:
    resb 4096