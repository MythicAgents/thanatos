SECTIONS
{
    . = 0x0;
    .text : ALIGN(0x1000)
    {
        *(.text.entrypoint)
        *(.text)
        *(.text*)
    }

    .rdata : ALIGN(0x100)
    {
        *(.rdata)
        *(.rdata*)
    }

    .data : ALIGN(0x1000)
    {
        *(.data)
        *(.data*)
    }

    /* We don't need debug info */
    /DISCARD/ :
    {
        *(.pdata)
        *(.xdata)
        *(.edata)
    }
}
