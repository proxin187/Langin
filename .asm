format ELF64 executable 3
segment readable executable
    ;; -- FUNCTION --
main:
    push rbp
    mov rbp, rsp
    ;; -- VARIABLE --
    mov qword [rbp-8], 0
    ;; -- WHILE --
BB_2:
    ;; -- COMPARISON --
    mov qword [rbp-16], 10
    mov rax, [rbp-8]
    cmp rax, [rbp-16]
    je BB_3
    ;; -- MUTATE VARIABLE --
    ;; -- BINARY EXPRESSION --
    mov qword [rbp-16], 1
    mov rax, [rbp-8]
    add rax, [rbp-16]
    mov qword [rbp-8], rax
    jmp BB_2
BB_3:
    ;; -- RETURN --
    mov rax, 0
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
