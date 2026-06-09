/* MultiOS x86_64 Linker Script */

OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)

ENTRY(_start)

PHDRS
{
    text    PT_LOAD    FLAGS((1 << 0) | (1 << 2)); /* Execute + Read */
    rodata  PT_LOAD    FLAGS((1 << 2));             /* Read only */
    data    PT_LOAD    FLAGS((1 << 1) | (1 << 2));  /* Write + Read */
}

SECTIONS
{
    . = 0xFFFFFFFF80000000 + SIZEOF_HEADERS;

    .text : {
        *(.text .text.*)
    } :text

    .rodata : {
        *(.rodata .rodata.*)
    } :rodata

    .data : {
        *(.data .data.*)
    } :data

    .bss : {
        *(.bss .bss.*)
        *(COMMON)
    } :data

    /DISCARD/ : {
        *(.comment)
        *(.eh_frame)
        *(.note.gnu.build-id)
    }
}
