@i
M=0

(RESET_AND_CLEAR_OR_FILL)
    @i
    M=0

    @KBD
    D=M
    @FILL_SCREEN
    D;JNE

    @KBD
    D=M
    @CLEAR_SCREEN
    D;JEQ

(CLEAR_SCREEN)
    @KBD
    D=M
    @RESET_AND_CLEAR_OR_FILL
    D;JNE

    @SCREEN
    D=A
    @i
    D=D+M
    A=D
    M=0

    @i
    M=M+1
    @CLEAR_SCREEN
    0;JMP

(FILL_SCREEN)
    @KBD
    D=M
    @RESET_AND_CLEAR_OR_FILL
    D;JEQ

    @SCREEN
    D=A
    @i
    D=D+M
    A=D
    M=!M

    @i
    M=M+1
    @FILL_SCREEN
    0;JMP