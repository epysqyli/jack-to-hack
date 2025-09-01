# VM Translator
Virtual stack machine translator for the [From Nand To Tetris](https://www.nand2tetris.org/course) course.

# Example

VM stack machine input:
```
push constant 1
push constant 2
add
```

Assembly output:
```
@1
D=A
@SP
A=M
M=D
@SP
M=M+1
@2
D=A
@SP
A=M
M=D
@SP
M=M+1
@SP
M=M-1
A=M
D=M
@SP
M=M-1
A=M
M=D+M
@SP
M=M+1
(END)
@END
0;JMP

```
