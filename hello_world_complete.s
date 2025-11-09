; AIR for module: main

section .text
global main

main:
.L0:
    ; MIR: Call { dest: None, func: Const(String("aurora_println")
    lea rdi, str_0
    call aurora_println
    ; MIR: Call { dest: None, func: Const(String("aurora_println")
    lea rdi, str_1
    call aurora_println
    ; MIR: Return { value: None, span: Span { start: 0, end: 0, fi
    xor rax, rax
    ret


section .data
str_0:  db 72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100, 33, 0
str_1:  db 87, 101, 108, 99, 111, 109, 101, 32, 116, 111, 32, 65, 117, 114, 111, 114, 97, 33, 0
