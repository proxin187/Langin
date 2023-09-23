format ELF64 executable 3
segment readable executable
    ;; -- FUNCTION --
square:
    push rbp
    mov rbp, rsp
    mov [rbp-8], rdi
    ;; -- RETURN --
    ;; -- BINARY EXPRESSION --
    mov rax, [rbp-8]
    mov rax, [rbp-8]
    mul rbx
    mov rax, rax
    jmp square_ret
square_ret:
    pop rbp
    ret
    ;; -- FUNCTION --
main:
    push rbp
    mov rbp, rsp
    ;; -- VARIABLE --
    ;; -- BINARY EXPRESSION --
    mov qword [rbp-24], 2
    mov qword [rbp-32], 6
    mov rax, [rbp-24]
    sub rax, [rbp-32]
    ;; -- BINARY EXPRESSION --
    mov qword [rbp-24], 10
    mov qword [rbp-32], rax
    mov rax, [rbp-24]
    add rax, [rbp-32]
    mov qword [rbp-16], rax
    ;; -- MUTATE VARIABLE --
    mov qword [rbp-16], 2
    ;; -- VARIABLE --
    ;; -- FUNCTION CALL --
    ;; -- BINARY EXPRESSION --
    mov qword [rbp-32], 2
    mov rax, [rbp-16]
    add rax, [rbp-32]
    mov rdi, rax
    call square
    mov qword [rbp-24], rax
    ;; -- VARIABLE --
    ;; -- REFERENCE --
    lea rax, [rbp-24]
    mov qword [rbp-32], rax
    ;; -- MUTATE POINTER --
    mov qword [rbp-40], 0
    mov rax, [rbp-32]
    mov rbx, [rbp-40]
    mov [rax], rbx
    ;; -- VARIABLE --
    mov qword [rbp-48], 0
    ;; -- RETURN --
    mov rax, [rbp-16]
    jmp main_ret
main_ret:
    pop rbp
    ret
entry start
start:
    call main
    mov rdi, rax
    mov rax, 60
    syscall
segment readable writeable
