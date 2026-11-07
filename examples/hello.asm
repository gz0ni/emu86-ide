; emu86 example: print a string through DOS INT 21h, function 09h.
; Written in the dialect used by university lab manuals (MASM/TASM style).

.model small
.stack 100h

.data
msg     db 'Hello from emu86!', 0Dh, 0Ah, '$'

.code
start:
    mov ax, @data
    mov ds, ax

    lea dx, msg
    mov ah, 09h
    int 21h

    mov ax, 4C00h
    int 21h

end start
