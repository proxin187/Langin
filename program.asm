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
    mov rbx, [rbp-8]
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
    mov qword [rbp-16], 2
    ;; -- VARIABLE --
    ;; -- BINARY EXPRESSION --
    mov qword [rbp-32], 6
    mov rax, [rbp-16]
    sub rax, [rbp-32]
    ;; -- BINARY EXPRESSION --
    mov qword [rbp-32], 10
    mov qword [rbp-40], rax
    mov rax, [rbp-32]
    add rax, [rbp-40]
    mov qword [rbp-24], rax
    ;; -- MUTATE VARIABLE --
    mov qword [rbp-24], 2
    ;; -- VARIABLE --
    ;; -- BINARY EXPRESSION --
    mov qword [rbp-40], 2
    mov qword [rbp-48], 420
    mov rax, [rbp-40]
    add rax, [rbp-48]
    mov qword [rbp-32], rax
    ;; -- VARIABLE --
    ;; -- FUNCTION CALL --
    ;; -- BINARY EXPRESSION --
    mov qword [rbp-48], 2
    mov rax, [rbp-24]
    add rax, [rbp-48]
    mov rdi, rax
    call square
    mov qword [rbp-40], rax
    ;; -- VARIABLE --
    ;; -- REFERENCE --
    lea rax, [rbp-40]
    mov qword [rbp-48], rax
    ;; -- MUTATE POINTER --
    mov qword [rbp-56], 0
    mov rax, [rbp-48]
    mov rbx, [rbp-56]
    mov [rax], rbx
    ;; -- RETURN --
    mov rax, [rbp-24]
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
