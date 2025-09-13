use crate::parser::Command;
use crate::parser::branching::BranchingArgs;
use crate::parser::function::FunctionArgs;
use crate::parser::operation::*;

macro_rules! address_top_stack {
    ($asm_instructions: ident) => {
        $asm_instructions.push("@SP".to_string());
        $asm_instructions.push("M=M-1".to_string());
        $asm_instructions.push("A=M".to_string());
    };
}

macro_rules! assign_d_reg_to_stack {
    ($asm_instructions: ident) => {
        $asm_instructions.push("@SP".to_string());
        $asm_instructions.push("A=M".to_string());
        $asm_instructions.push("M=D".to_string());
    };
}

macro_rules! incr_stack_pointer {
    ($asm_instructions: ident) => {
        $asm_instructions.push("@SP".to_string());
        $asm_instructions.push("M=M+1".to_string());
    };
}

macro_rules! push_d_reg_to_stack {
    ($asm_instructions: ident) => {
        assign_d_reg_to_stack!($asm_instructions);
        incr_stack_pointer!($asm_instructions);
    };
}

fn generate_branching_asm(args: &BranchingArgs, asm: &mut Vec<String>) {
    match args {
        BranchingArgs::Label(label) => {
            asm.push(format!("({})", label));
        }
        BranchingArgs::Goto(label) => {
            asm.push(format!("@{}", label));
            asm.push("0;JMP".to_string());
        }
        BranchingArgs::IfGoto(label) => {
            address_top_stack!(asm);
            asm.push("D=M".to_string());
            asm.push(format!("@{}", label));
            asm.push("D;JNE".to_string());
        }
    }
}

fn generate_function_asm(args: &FunctionArgs, asm: &mut Vec<String>, program_name: &str) {
    // TODO: implement recursive function calls
    // Functions need to have the full unique name FileName.FunctionName.
    // Each Jack program will require one Main.main function, called by Sys.init.
    match args {
        FunctionArgs::Function(fn_name, n_local_vars) => {
            asm.push(format!("({}.{})", program_name, fn_name));
            for i in 0..*n_local_vars {
                asm.push(format!("@{}", i));
                asm.push("D=A".to_string());
                asm.push("@LCL".to_string());
                asm.push("A=D+M".to_string());
                asm.push("M=0".to_string());
            }
        }
        FunctionArgs::Call(fn_name, n_caller_args) => {
            asm.push(format!("@{}.ReturnFrom.{}", program_name, fn_name));
            asm.push("D=A".to_string());
            push_d_reg_to_stack!(asm);

            asm.push("@LCL".to_string());
            asm.push("D=M".to_string());
            push_d_reg_to_stack!(asm);

            asm.push("@ARG".to_string());
            asm.push("D=M".to_string());
            push_d_reg_to_stack!(asm);

            asm.push("@THIS".to_string());
            asm.push("D=M".to_string());
            push_d_reg_to_stack!(asm);

            asm.push("@THAT".to_string());
            asm.push("D=M".to_string());
            push_d_reg_to_stack!(asm);

            asm.push("@SP".to_string());
            asm.push("D=M".to_string());
            asm.push(format!("@{}", 5 + n_caller_args));
            asm.push("D=D-A".to_string());
            asm.push("@ARG".to_string());
            asm.push("M=D".to_string());

            asm.push("@SP".to_string());
            asm.push("D=M".to_string());
            asm.push("@LCL".to_string());
            asm.push("M=D".to_string());

            asm.push(format!("@{}.{}", program_name, fn_name));
            asm.push("0;JMP".to_string());

            asm.push(format!("({}.ReturnFrom.{})", program_name, fn_name));
        }
        FunctionArgs::Return => {
            // frame = LCL: define frame as temp variable R5 and assign LCL to it
            asm.push("@LCL".to_string());
            asm.push("D=M".to_string());
            asm.push("@R5".to_string());
            asm.push("M=D".to_string());

            // save the return address (M[LCL] - 5) to temp variable R6
            asm.push("@5".to_string());
            asm.push("D=D-A".to_string());
            asm.push("@R6".to_string());
            asm.push("M=D".to_string());

            // reposition return value for the caller: pop from the stack to ARG
            address_top_stack!(asm);
            asm.push("D=M".to_string());
            asm.push("@ARG".to_string());
            asm.push("A=M".to_string());
            asm.push("M=D".to_string());

            // reposition SP for the caller to @ARG + 1
            asm.push("@ARG".to_string());
            asm.push("D=M+1".to_string());
            asm.push("@SP".to_string());
            asm.push("M=D".to_string());

            // restore THAT for the caller
            asm.push("@R5".to_string());
            asm.push("D=M".to_string());
            asm.push("A=D-1".to_string());
            asm.push("D=M".to_string());
            asm.push("@THAT".to_string());
            asm.push("M=D".to_string());

            // restore THIS for the caller
            asm.push("@R5".to_string());
            asm.push("D=M".to_string());
            asm.push("D=D-1".to_string());
            asm.push("A=D-1".to_string());
            asm.push("D=M".to_string());
            asm.push("@THIS".to_string());
            asm.push("M=D".to_string());

            // restore ARG for the caller
            asm.push("@R5".to_string());
            asm.push("D=M".to_string());
            asm.push("D=D-1".to_string());
            asm.push("D=D-1".to_string());
            asm.push("A=D-1".to_string());
            asm.push("D=M".to_string());
            asm.push("@ARG".to_string());
            asm.push("M=D".to_string());

            // restore LCL for the caller
            asm.push("@R5".to_string());
            asm.push("D=M".to_string());
            asm.push("D=D-1".to_string());
            asm.push("D=D-1".to_string());
            asm.push("D=D-1".to_string());
            asm.push("A=D-1".to_string());
            asm.push("D=M".to_string());
            asm.push("@LCL".to_string());
            asm.push("M=D".to_string());

            // goto return address
            // TODO: implement recursive function calls
            asm.push("@R6".to_string());
            asm.push("A=M".to_string());
            asm.push("A=M".to_string());
            asm.push("0;JMP".to_string());
        }
    }
}

fn generate_operation_asm(args: &OperationArgs, asm: &mut Vec<String>, program_name: &str) {
    match args {
        OperationArgs::Push(mem_segment, val) => {
            match mem_segment {
                MemorySegment::Constant => {
                    asm.push(format!("@{}", val));
                    asm.push("D=A".to_string());
                    assign_d_reg_to_stack!(asm);
                    incr_stack_pointer!(asm);
                }
                MemorySegment::Local
                | MemorySegment::Argument
                | MemorySegment::This
                | MemorySegment::That => {
                    asm.push(format!("@{}", val));
                    asm.push("D=A".to_string());
                    asm.push(mem_segment.as_asm_mnemonic());
                    asm.push("A=D+M".to_string());
                    asm.push("D=M".to_string());
                    assign_d_reg_to_stack!(asm);
                    incr_stack_pointer!(asm);
                }
                MemorySegment::Temp => {
                    // TEMP address range is 5..12
                    asm.push(format!("@R{}", 5 + val));
                    asm.push("D=M".to_string());
                    assign_d_reg_to_stack!(asm);
                    incr_stack_pointer!(asm);
                }
                MemorySegment::Static => {
                    asm.push(format!("@{}.{}", program_name, val));
                    asm.push("D=M".to_string());
                    assign_d_reg_to_stack!(asm);
                    incr_stack_pointer!(asm);
                }
                MemorySegment::Pointer => {
                    match val {
                        0 => asm.push(format!("@THIS")),
                        1 => asm.push(format!("@THAT")),
                        _ => panic!("Pop operations on pointer allow values 0 or 1"),
                    }
                    asm.push("D=M".to_string());
                    assign_d_reg_to_stack!(asm);
                    incr_stack_pointer!(asm);
                }
            };
        }
        OperationArgs::Pop(mem_segment, val) => {
            match mem_segment {
                MemorySegment::Local
                | MemorySegment::Argument
                | MemorySegment::This
                | MemorySegment::That => {
                    address_top_stack!(asm);
                    asm.push("D=M".to_string());
                    asm.push("@R13".to_string());
                    asm.push("M=D".to_string());

                    asm.push(format!("@{}", val));
                    asm.push("D=A".to_string());
                    asm.push(mem_segment.as_asm_mnemonic());
                    asm.push("A=D+M".to_string());
                    asm.push("D=A".to_string());
                    asm.push("@R14".to_string());
                    asm.push("M=D".to_string());
                    asm.push("@R13".to_string());
                    asm.push("D=M".to_string());
                    asm.push("@R14".to_string());
                    asm.push("A=M".to_string());
                    asm.push("M=D".to_string());
                }
                MemorySegment::Temp => {
                    address_top_stack!(asm);
                    asm.push("D=M".to_string());
                    // TEMP address range is 5..12
                    asm.push(format!("@R{}", 5 + val));
                    asm.push("M=D".to_string());
                }
                MemorySegment::Static => {
                    address_top_stack!(asm);
                    asm.push("D=M".to_string());
                    asm.push(format!("@{}.{}", program_name, val));
                    asm.push("M=D".to_string());
                }
                MemorySegment::Pointer => {
                    address_top_stack!(asm);
                    asm.push("D=M".to_string());
                    match val {
                        0 => asm.push(format!("@THIS")),
                        1 => asm.push(format!("@THAT")),
                        _ => panic!("Pop operations on pointer allow values 0 or 1"),
                    }
                    asm.push("M=D".to_string());
                }
                MemorySegment::Constant => panic!("Cannot pop from Constant"),
            }
        }
        OperationArgs::Add | OperationArgs::Sub | OperationArgs::And | OperationArgs::Or => {
            address_top_stack!(asm);
            asm.push("D=M".to_string());
            address_top_stack!(asm);
            match args {
                OperationArgs::Add => asm.push("M=D+M".to_string()),
                OperationArgs::Sub => asm.push("M=M-D".to_string()),
                OperationArgs::And => asm.push("M=D&M".to_string()),
                OperationArgs::Or => asm.push("M=D|M".to_string()),
                _ => (),
            }
            incr_stack_pointer!(asm);
        }
        OperationArgs::Neg => {
            address_top_stack!(asm);
            asm.push("M=-M".to_string());
            incr_stack_pointer!(asm);
        }
        OperationArgs::Not => {
            address_top_stack!(asm);
            asm.push("M=!M".to_string());
            incr_stack_pointer!(asm);
        }
        OperationArgs::Eq | OperationArgs::Gt | OperationArgs::Lt => {
            address_top_stack!(asm);
            asm.push("D=M".to_string());
            address_top_stack!(asm);
            asm.push("D=M-D".to_string());
            asm.push("@PUSH_TRUE".to_string());

            match args {
                OperationArgs::Eq => asm.push("D;JEQ".to_string()),
                OperationArgs::Lt => asm.push("D;JLT".to_string()),
                OperationArgs::Gt => asm.push("D;JGT".to_string()),
                _ => {}
            }

            asm.push("(PUSH_FALSE)".to_string());
            asm.push("@SP".to_string());
            asm.push("A=M".to_string());
            asm.push("M=0".to_string());
            asm.push("@NO_OP".to_string());
            asm.push("0;JMP".to_string());

            asm.push("(PUSH_TRUE)".to_string());
            asm.push("@SP".to_string());
            asm.push("A=M".to_string());
            asm.push("M=-1".to_string());

            asm.push("(NO_OP)".to_string());
            incr_stack_pointer!(asm);
        }
    }
}

pub fn generate_asm(vm_command: &Command, program_name: &str) -> Vec<String> {
    let mut asm: Vec<String> = vec![];

    match vm_command {
        Command::Branching(args) => generate_branching_asm(args, &mut asm),
        Command::Function(args) => generate_function_asm(args, &mut asm, program_name),
        Command::Operation(args) => generate_operation_asm(args, &mut asm, program_name),
    }

    asm
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PROGRAM_NAME: &'static str = "TestProgram";

    #[test]
    fn stack_push() {
        let vm_command: Command =
            Command::Operation(OperationArgs::Push(MemorySegment::Constant, 1));

        assert_eq!(
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            generate_asm(&vm_command, TEST_PROGRAM_NAME)
        );
    }

    fn assert_commands_eq(vm_commands: Vec<&Command>, expected_asm: Vec<Vec<&str>>) {
        vm_commands
            .iter()
            .zip(expected_asm)
            .for_each(|(vm_command, expected)| {
                assert_eq!(expected, generate_asm(vm_command, TEST_PROGRAM_NAME))
            });
    }

    #[test]
    fn stack_double_push_and_add() {
        let vm_commands: Vec<&Command> = vec![
            &Command::Operation(OperationArgs::Push(MemorySegment::Constant, 1)),
            &Command::Operation(OperationArgs::Push(MemorySegment::Constant, 2)),
            &Command::Operation(OperationArgs::Add),
        ];

        let expected_asm: Vec<Vec<&str>> = vec![
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@2", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec![
                "@SP", "M=M-1", "A=M", "D=M", "@SP", "M=M-1", "A=M", "M=D+M", "@SP", "M=M+1",
            ],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn stack_double_push_and_sub() {
        let vm_commands: Vec<&Command> = vec![
            &Command::Operation(OperationArgs::Push(MemorySegment::Constant, 1)),
            &Command::Operation(OperationArgs::Push(MemorySegment::Constant, 2)),
            &Command::Operation(OperationArgs::Sub),
        ];

        let expected_asm: Vec<Vec<&str>> = vec![
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@2", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec![
                "@SP", "M=M-1", "A=M", "D=M", "@SP", "M=M-1", "A=M", "M=M-D", "@SP", "M=M+1",
            ],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn stack_double_push_and_logical_and() {
        let vm_commands: Vec<&Command> = vec![
            &Command::Operation(OperationArgs::Push(MemorySegment::Constant, 1)),
            &Command::Operation(OperationArgs::Push(MemorySegment::Constant, 2)),
            &Command::Operation(OperationArgs::And),
        ];

        let expected_asm: Vec<Vec<&str>> = vec![
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@2", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec![
                "@SP", "M=M-1", "A=M", "D=M", "@SP", "M=M-1", "A=M", "M=D&M", "@SP", "M=M+1",
            ],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn stack_double_push_and_logical_or() {
        let vm_commands: Vec<&Command> = vec![
            &Command::Operation(OperationArgs::Push(MemorySegment::Constant, 1)),
            &Command::Operation(OperationArgs::Push(MemorySegment::Constant, 2)),
            &Command::Operation(OperationArgs::Or),
        ];

        let expected_asm: Vec<Vec<&str>> = vec![
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@2", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec![
                "@SP", "M=M-1", "A=M", "D=M", "@SP", "M=M-1", "A=M", "M=D|M", "@SP", "M=M+1",
            ],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn single_stack_push_and_neg() {
        let vm_commands: Vec<&Command> = vec![
            &Command::Operation(OperationArgs::Push(MemorySegment::Constant, 1)),
            &Command::Operation(OperationArgs::Neg),
        ];

        let expected_asm: Vec<Vec<&str>> = vec![
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@SP", "M=M-1", "A=M", "M=-M", "@SP", "M=M+1"],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn single_stack_push_and_not() {
        let vm_commands: Vec<&Command> = vec![
            &Command::Operation(OperationArgs::Push(MemorySegment::Constant, 1)),
            &Command::Operation(OperationArgs::Not),
        ];

        let expected_asm: Vec<Vec<&str>> = vec![
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@SP", "M=M-1", "A=M", "M=!M", "@SP", "M=M+1"],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn stack_double_push_and_eq() {
        let vm_commands: Vec<&Command> = vec![
            &Command::Operation(OperationArgs::Push(MemorySegment::Constant, 1)),
            &Command::Operation(OperationArgs::Push(MemorySegment::Constant, 2)),
            &Command::Operation(OperationArgs::Eq),
        ];

        let expected_asm: Vec<Vec<&str>> = vec![
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@2", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec![
                "@SP",
                "M=M-1",
                "A=M",
                "D=M",
                "@SP",
                "M=M-1",
                "A=M",
                "D=M-D",
                "@PUSH_TRUE",
                "D;JEQ",
                "(PUSH_FALSE)",
                "@SP",
                "A=M",
                "M=0",
                "@NO_OP",
                "0;JMP",
                "(PUSH_TRUE)",
                "@SP",
                "A=M",
                "M=-1",
                "(NO_OP)",
                "@SP",
                "M=M+1",
            ],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn stack_double_push_and_lt() {
        let vm_commands: Vec<&Command> = vec![
            &Command::Operation(OperationArgs::Push(MemorySegment::Constant, 1)),
            &Command::Operation(OperationArgs::Push(MemorySegment::Constant, 2)),
            &Command::Operation(OperationArgs::Lt),
        ];

        let expected_asm: Vec<Vec<&str>> = vec![
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@2", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec![
                "@SP",
                "M=M-1",
                "A=M",
                "D=M",
                "@SP",
                "M=M-1",
                "A=M",
                "D=M-D",
                "@PUSH_TRUE",
                "D;JLT",
                "(PUSH_FALSE)",
                "@SP",
                "A=M",
                "M=0",
                "@NO_OP",
                "0;JMP",
                "(PUSH_TRUE)",
                "@SP",
                "A=M",
                "M=-1",
                "(NO_OP)",
                "@SP",
                "M=M+1",
            ],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn stack_double_push_and_gt() {
        let vm_commands: Vec<&Command> = vec![
            &Command::Operation(OperationArgs::Push(MemorySegment::Constant, 1)),
            &Command::Operation(OperationArgs::Push(MemorySegment::Constant, 2)),
            &Command::Operation(OperationArgs::Gt),
        ];

        let expected_asm: Vec<Vec<&str>> = vec![
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@2", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec![
                "@SP",
                "M=M-1",
                "A=M",
                "D=M",
                "@SP",
                "M=M-1",
                "A=M",
                "D=M-D",
                "@PUSH_TRUE",
                "D;JGT",
                "(PUSH_FALSE)",
                "@SP",
                "A=M",
                "M=0",
                "@NO_OP",
                "0;JMP",
                "(PUSH_TRUE)",
                "@SP",
                "A=M",
                "M=-1",
                "(NO_OP)",
                "@SP",
                "M=M+1",
            ],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn push_twice_add_push_and_sub() {
        let vm_commands: Vec<&Command> = vec![
            &Command::Operation(OperationArgs::Push(MemorySegment::Constant, 5)),
            &Command::Operation(OperationArgs::Push(MemorySegment::Constant, 5)),
            &Command::Operation(OperationArgs::Add),
            &Command::Operation(OperationArgs::Push(MemorySegment::Constant, 10)),
            &Command::Operation(OperationArgs::Sub),
        ];

        let expected_asm: Vec<Vec<&str>> = vec![
            vec!["@5", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@5", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec![
                "@SP", "M=M-1", "A=M", "D=M", "@SP", "M=M-1", "A=M", "M=D+M", "@SP", "M=M+1",
            ],
            vec!["@10", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec![
                "@SP", "M=M-1", "A=M", "D=M", "@SP", "M=M-1", "A=M", "M=M-D", "@SP", "M=M+1",
            ],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn push_to_stack_and_pop_from_stack_to_memory_segment() {
        let push_cmd = Command::Operation(OperationArgs::Push(MemorySegment::Constant, 1));

        let memory_segments = [
            MemorySegment::Local,
            MemorySegment::Argument,
            MemorySegment::This,
            MemorySegment::That,
        ];

        memory_segments.into_iter().for_each(|memory_segment| {
            let mem_segment_asm = &memory_segment.as_asm_mnemonic();
            assert_commands_eq(
                vec![
                    &push_cmd,
                    &Command::Operation(OperationArgs::Pop(memory_segment, 5)),
                ],
                vec![
                    vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
                    vec![
                        "@SP",
                        "M=M-1",
                        "A=M",
                        "D=M",
                        "@R13",
                        "M=D",
                        "@5",
                        "D=A",
                        mem_segment_asm,
                        "A=D+M",
                        "D=A",
                        "@R14",
                        "M=D",
                        "@R13",
                        "D=M",
                        "@R14",
                        "A=M",
                        "M=D",
                    ],
                ],
            );
        });
    }

    #[test]
    fn push_to_stack_pop_to_local_and_back_to_stack() {
        let vm_commands: Vec<&Command> = vec![
            &Command::Operation(OperationArgs::Push(MemorySegment::Constant, 5)),
            &Command::Operation(OperationArgs::Pop(MemorySegment::Local, 2)),
            &Command::Operation(OperationArgs::Push(MemorySegment::Local, 2)),
        ];

        let expected_asm = vec![
            vec!["@5", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec![
                "@SP", "M=M-1", "A=M", "D=M", "@R13", "M=D", "@2", "D=A", "@LCL", "A=D+M", "D=A",
                "@R14", "M=D", "@R13", "D=M", "@R14", "A=M", "M=D",
            ],
            vec![
                "@2", "D=A", "@LCL", "A=D+M", "D=M", "@SP", "A=M", "M=D", "@SP", "M=M+1",
            ],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn push_to_stack_pop_to_temp_and_push_from_temp_to_stack() {
        let vm_commands: Vec<&Command> = vec![
            &Command::Operation(OperationArgs::Push(MemorySegment::Constant, 3)),
            &Command::Operation(OperationArgs::Pop(MemorySegment::Temp, 4)),
            &Command::Operation(OperationArgs::Push(MemorySegment::Temp, 4)),
        ];

        let expected_asm = vec![
            // push the constant 3 onto the stack
            vec!["@3", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            // pop 3 from stack and save it on temp index 4, i.e. memory address 9
            vec!["@SP", "M=M-1", "A=M", "D=M", "@R9", "M=D"],
            // // push to stack from temp index 4
            vec!["@R9", "D=M", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn push_to_stack_and_pop_to_static_and_push_back_to_stack() {
        let vm_commands: Vec<&Command> = vec![
            &Command::Operation(OperationArgs::Push(MemorySegment::Constant, 5)),
            &Command::Operation(OperationArgs::Pop(MemorySegment::Static, 1)),
            &Command::Operation(OperationArgs::Push(MemorySegment::Static, 1)),
        ];

        let expected_asm = vec![
            vec!["@5", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@SP", "M=M-1", "A=M", "D=M", "@TestProgram.1", "M=D"],
            vec!["@TestProgram.1", "D=M", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn push_to_stack_and_pop_to_pointers() {
        let vm_commands: Vec<&Command> = vec![
            &Command::Operation(OperationArgs::Push(MemorySegment::Constant, 5)),
            &Command::Operation(OperationArgs::Push(MemorySegment::Constant, 6)),
            &Command::Operation(OperationArgs::Pop(MemorySegment::Pointer, 0)),
            &Command::Operation(OperationArgs::Pop(MemorySegment::Pointer, 1)),
            &Command::Operation(OperationArgs::Push(MemorySegment::Pointer, 0)),
            &Command::Operation(OperationArgs::Push(MemorySegment::Pointer, 1)),
        ];

        let expected_asm = vec![
            vec!["@5", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@6", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@SP", "M=M-1", "A=M", "D=M", "@THIS", "M=D"],
            vec!["@SP", "M=M-1", "A=M", "D=M", "@THAT", "M=D"],
            vec!["@THIS", "D=M", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@THAT", "D=M", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn define_and_goto_label() {
        let label_definition = Command::Branching(BranchingArgs::Label("TEST".to_string()));
        let goto_label = Command::Branching(BranchingArgs::Goto("TEST".to_string()));

        assert_commands_eq(
            vec![&label_definition, &goto_label],
            vec![vec!["(TEST)"], vec!["@TEST", "0;JMP"]],
        );
    }

    #[test]
    fn define_and_if_goto_label() {
        let label_definition = Command::Branching(BranchingArgs::Label("TEST".to_string()));
        let if_goto_label = Command::Branching(BranchingArgs::IfGoto("TEST".to_string()));

        let expected_asm = vec![
            vec!["(TEST)"],
            vec!["@SP", "M=M-1", "A=M", "D=M", "@TEST", "D;JNE"],
        ];

        assert_commands_eq(vec![&label_definition, &if_goto_label], expected_asm);
    }

    #[test]
    fn define_function_signature_with_no_local_vars() {
        let cmd = Command::Function(FunctionArgs::Function("TestFunc".to_string(), 0));
        let expected_asm = vec![vec!["(TestProgram.TestFunc)"]];

        assert_commands_eq(vec![&cmd], expected_asm);
    }

    #[test]
    fn define_function_signature_with_two_local_vars() {
        let cmd = Command::Function(FunctionArgs::Function("TestFunc".to_string(), 2));

        let expected_asm = vec![vec![
            "(TestProgram.TestFunc)",
            "@0",
            "D=A",
            "@LCL",
            "A=D+M",
            "M=0",
            "@1",
            "D=A",
            "@LCL",
            "A=D+M",
            "M=0",
        ]];

        assert_commands_eq(vec![&cmd], expected_asm);
    }

    #[test]
    fn call_sum_function_with_no_local_vars() {
        let call_fn_command = &Command::Function(FunctionArgs::Call("Sum".to_string(), 2));
        let define_fn_command = &Command::Function(FunctionArgs::Function("Sum".to_string(), 0));

        let vm_commands: Vec<&Command> = vec![
            &Command::Operation(OperationArgs::Push(MemorySegment::Constant, 1)),
            &Command::Operation(OperationArgs::Push(MemorySegment::Constant, 2)),
            &call_fn_command,
            &define_fn_command,
            &Command::Operation(OperationArgs::Push(MemorySegment::Argument, 0)),
            &Command::Operation(OperationArgs::Push(MemorySegment::Argument, 1)),
            &Command::Operation(OperationArgs::Add),
        ];

        let expected_asm = vec![
            // --- begin push args --- //
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@2", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            // --- end push args --- //

            // --- begin call function --- //
            vec![
                // Push return address
                "@TestProgram.ReturnFrom.Sum",
                "D=A",
                "@SP",
                "A=M",
                "M=D",
                "@SP",
                "M=M+1",
                // push LCL
                "@LCL",
                "D=M",
                "@SP",
                "A=M",
                "M=D",
                "@SP",
                "M=M+1",
                // push ARG
                "@ARG",
                "D=M",
                "@SP",
                "A=M",
                "M=D",
                "@SP",
                "M=M+1",
                // push THIS
                "@THIS",
                "D=M",
                "@SP",
                "A=M",
                "M=D",
                "@SP",
                "M=M+1",
                // push THAT
                "@THAT",
                "D=M",
                "@SP",
                "A=M",
                "M=D",
                "@SP",
                "M=M+1",
                // reposition ARG to SP - 5 - number of args
                "@SP",
                "D=M",
                "@7",
                "D=D-A",
                "@ARG",
                "M=D",
                // reposition LCL to SP
                "@SP",
                "D=M",
                "@LCL",
                "M=D",
                // goto callee
                "@TestProgram.Sum",
                "0;JMP",
                // inject return address label to the asm instructions
                "(TestProgram.ReturnFrom.Sum)",
            ],
            // --- end call function --- //

            // // --- begin function definition --- //
            vec!["(TestProgram.Sum)"],
            // no local vars init since there are no local vars
            //
            // push arg0 to stack
            vec![
                "@0", "D=A", "@ARG", "A=D+M", "D=M", "@SP", "A=M", "M=D", "@SP", "M=M+1",
            ],
            // push arg1 to stack
            vec![
                "@1", "D=A", "@ARG", "A=D+M", "D=M", "@SP", "A=M", "M=D", "@SP", "M=M+1",
            ],
            // add
            vec![
                "@SP", "M=M-1", "A=M", "D=M", "@SP", "M=M-1", "A=M", "M=D+M", "@SP", "M=M+1",
            ],
            // --- end function definition --- //
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn return_from_function() {
        let expected_asm: Vec<&str> = vec![
            "@LCL", "D=M", "@R5", "M=D", "@5", "D=D-A", "@R6", "M=D", "@SP", "M=M-1", "A=M", "D=M",
            "@ARG", "A=M", "M=D", "@ARG", "D=M+1", "@SP", "M=D", "@R5", "D=M", "A=D-1", "D=M",
            "@THAT", "M=D", "@R5", "D=M", "D=D-1", "A=D-1", "D=M", "@THIS", "M=D", "@R5", "D=M",
            "D=D-1", "D=D-1", "A=D-1", "D=M", "@ARG", "M=D", "@R5", "D=M", "D=D-1", "D=D-1",
            "D=D-1", "A=D-1", "D=M", "@LCL", "M=D", "@R6", "A=M", "A=M", "0;JMP",
        ];

        assert_eq!(
            expected_asm,
            generate_asm(
                &Command::Function(FunctionArgs::Return),
                self::TEST_PROGRAM_NAME
            )
        );
    }

    #[test]
    #[ignore = "TODO"]
    fn call_function_from_within_function() {}

    #[test]
    #[ignore = "TODO"]
    fn call_function_recursively() {}
}
