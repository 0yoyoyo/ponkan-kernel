global_asm!(
    ".global io_out32",
    "io_out32:",
    "    mov dx, di",
    "    mov eax, esi",
    "    out dx, eax",
    "    ret",
    "",
    ".global io_in32",
    "io_in32:",
    "    mov dx, di",
    "    in eax, dx",
    "    ret",
);
