SECTIONS
{
    . = 0x0;
    .text
    {
        *(.text.entrypoint)
        *(.text)
        *(.text*) /* TODO: Figure out what .text sections are linked then remove the wildcard */
    }

    .rdata : ALIGN(0x100)
    {
        *(.rdata)
        *(.rdata*) /* TODO: Figure out what .rdata sections are linked and then remove the wildcard */
    }

    .data : ALIGN(0x1000)
    {
        *(.data)
        *(.data*) /* TODO: Figure out what .data sections are linked and then remove the wildcard */
    }

    /* We don't need debug info sections in release mode */
    /DISCARD/ :
    {
        *(.pdata)
        *(.xdata)
        *(.edata)
    }
}
