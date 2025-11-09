; Handcrafted assembly from AIR output for hello_world
section .text
global main
extern aurora_println

main:
    ; Call println("Hello, World!")
    lea rdi, [rel str_0]
    call aurora_println

    ; Call println("Welcome to Aurora!")
    lea rdi, [rel str_1]
    call aurora_println

    ; Return 0
    xor eax, eax
    ret

section .data
str_0:  db "Hello, World!", 0
str_1:  db "Welcome to Aurora!", 0
