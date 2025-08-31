@R0
D=M
@ZERO_OUTPUT
D;JEQ

@R1
D=M
@ZERO_OUTPUT
D;JEQ

@R2
M=0

@R1
D=M
@i
M=D

(LOOP) // add R0 to itself i times
    @i
    D=M
    @END
    D;JEQ

    @R2
    D=M
    @R0
    D=D+M
    @R2
    M=D
    @i
    M=M-1

    @LOOP
    0;JMP

(ZERO_OUTPUT)
    @0
    D=A
    @R2
    M=D
    @END
    0;JMP

(END)
    @END
    0;JMP
