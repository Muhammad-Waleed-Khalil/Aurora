# GAS assembly for hello_world (AT&T syntax)
.section .text
.globl main
.extern aurora_println

main:
    # Call println("Hello, World!")
    lea str_0(%rip), %rdi
    call aurora_println

    # Call println("Welcome to Aurora!")
    lea str_1(%rip), %rdi
    call aurora_println

    # Return 0
    xor %eax, %eax
    ret

.section .rodata
str_0:  .asciz "Hello, World!"
str_1:  .asciz "Welcome to Aurora!"
