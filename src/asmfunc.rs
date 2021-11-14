global_asm!(
    ".global io_out32",
    "io_out32:",
    "    mov dx, di",
    "    mov eax, esi",
    "    out dx, eax",
    "    ret",
);

global_asm!(
    ".global io_in32",
    "io_in32:",
    "    mov dx, di",
    "    in eax, dx",
    "    ret",
);

global_asm!(
    ".global get_cs",
    "get_cs:",
    "    xor eax, eax",
    "    mov ax, cs",
    "    ret",
);

global_asm!(
    ".global load_idt",
    "load_idt:",
    "    push rbp",
    "    mov rbp, rsp",
    "    sub rsp, 10",
    "    mov [rsp], di",
    "    mov [rsp + 2], rsi",
    "    lidt [rsp]",
    "    mov rsp, rbp",
    "    pop rbp",
    "    ret",
);

global_asm!(
    ".global load_gdt",
    "load_gdt:",
    "    push rbp",
    "    mov rbp, rsp",
    "    sub rsp, 10",
    "    mov [rsp], di",
    "    mov [rsp + 2], rsi",
    "    lgdt [rsp]",
    "    mov rsp, rbp",
    "    pop rbp",
    "    ret",
);

global_asm!(
    ".global set_ds_all",
    "set_ds_all:",
    "    mov ds, di",
    "    mov es, di",
    "    mov fs, di",
    "    mov gs, di",
    "    ret",
);

global_asm!(
    ".global set_cs_ss",
    "set_cs_ss:",
    "    push rbp",
    "    mov rbp, rsp",
    "    mov ss, si",
    "    mov rax, OFFSET next",
    "    push rdi",
    "    push rax",
    "    rex64 retf",
    "next:",
    "    mov rsp, rbp",
    "    pop rbp",
    "    ret",
);

global_asm!(
    ".global set_cr3",
    "set_cr3:",
    "    mov cr3, rdi",
    "    ret",
);

global_asm!(
    ".global kernel_main",
    "kernel_main:",
    "    mov rsp, OFFSET KERNEL_MAIN_STACK + 1024 * 1024",
    "    call kernel_main_new_stack",
    ".fin:",
    "    hlt",
    "    jmp .fin",
);
