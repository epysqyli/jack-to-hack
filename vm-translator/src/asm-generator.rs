use crate::parser::Command;
use crate::parser::branching::BranchingArgs;
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

fn generate_branching_asm(branching_args: &BranchingArgs, asm_instructions: &mut Vec<String>) {
    match branching_args {
        BranchingArgs::Label(label) => {
            asm_instructions.push(format!("({})", label));
        }
        BranchingArgs::Goto(label) => {
            asm_instructions.push(format!("@{}", label));
            asm_instructions.push("0;JMP".to_string());
        }
        BranchingArgs::IfGoto(label) => {
            address_top_stack!(asm_instructions);
            asm_instructions.push("D=M".to_string());
            asm_instructions.push(format!("@{}", label));
            asm_instructions.push("D;JNE".to_string());
        }
    }
}

fn generate_operation_asm(
    operation_args: &OperationArgs,
    asm_instructions: &mut Vec<String>,
    program_name: &str,
) {
    match operation_args {
        OperationArgs::Push(mem_segment, val) => {
            match mem_segment {
                MemorySegment::Constant => {
                    asm_instructions.push(format!("@{}", val));
                    asm_instructions.push("D=A".to_string());
                    assign_d_reg_to_stack!(asm_instructions);
                    incr_stack_pointer!(asm_instructions);
                }
                MemorySegment::Local
                | MemorySegment::Argument
                | MemorySegment::This
                | MemorySegment::That => {
                    asm_instructions.push(format!("@{}", val));
                    asm_instructions.push("D=A".to_string());
                    asm_instructions.push(mem_segment.as_asm_mnemonic());
                    asm_instructions.push("A=D+M".to_string());
                    asm_instructions.push("D=M".to_string());
                    assign_d_reg_to_stack!(asm_instructions);
                    incr_stack_pointer!(asm_instructions);
                }
                MemorySegment::Temp => {
                    // TEMP address range is 5..12
                    asm_instructions.push(format!("@R{}", 5 + val));
                    asm_instructions.push("D=M".to_string());
                    assign_d_reg_to_stack!(asm_instructions);
                    incr_stack_pointer!(asm_instructions);
                }
                MemorySegment::Static => {
                    asm_instructions.push(format!("@{}.{}", program_name, val));
                    asm_instructions.push("D=M".to_string());
                    assign_d_reg_to_stack!(asm_instructions);
                    incr_stack_pointer!(asm_instructions);
                }
                MemorySegment::Pointer => {
                    match val {
                        0 => asm_instructions.push(format!("@THIS")),
                        1 => asm_instructions.push(format!("@THAT")),
                        _ => {
                            panic!("Pop operations on pointer allow values 0 or 1")
                        }
                    }
                    asm_instructions.push("D=M".to_string());
                    assign_d_reg_to_stack!(asm_instructions);
                    incr_stack_pointer!(asm_instructions);
                }
            };
        }
        OperationArgs::Pop(mem_segment, val) => {
            match mem_segment {
                MemorySegment::Local
                | MemorySegment::Argument
                | MemorySegment::This
                | MemorySegment::That => {
                    address_top_stack!(asm_instructions);
                    asm_instructions.push("D=M".to_string());
                    asm_instructions.push("@R13".to_string());
                    asm_instructions.push("M=D".to_string());

                    asm_instructions.push(format!("@{}", val));
                    asm_instructions.push("D=A".to_string());
                    asm_instructions.push(mem_segment.as_asm_mnemonic());
                    asm_instructions.push("A=D+M".to_string());
                    asm_instructions.push("D=A".to_string());
                    asm_instructions.push("@R14".to_string());
                    asm_instructions.push("M=D".to_string());
                    asm_instructions.push("@R13".to_string());
                    asm_instructions.push("D=M".to_string());
                    asm_instructions.push("@R14".to_string());
                    asm_instructions.push("A=M".to_string());
                    asm_instructions.push("M=D".to_string());
                }
                MemorySegment::Temp => {
                    address_top_stack!(asm_instructions);
                    asm_instructions.push("D=M".to_string());
                    // TEMP address range is 5..12
                    asm_instructions.push(format!("@R{}", 5 + val));
                    asm_instructions.push("M=D".to_string());
                }
                MemorySegment::Static => {
                    address_top_stack!(asm_instructions);
                    asm_instructions.push("D=M".to_string());
                    asm_instructions.push(format!("@{}.{}", program_name, val));
                    asm_instructions.push("M=D".to_string());
                }
                MemorySegment::Pointer => {
                    address_top_stack!(asm_instructions);
                    asm_instructions.push("D=M".to_string());
                    match val {
                        0 => asm_instructions.push(format!("@THIS")),
                        1 => asm_instructions.push(format!("@THAT")),
                        _ => {
                            panic!("Pop operations on pointer allow values 0 or 1")
                        }
                    }
                    asm_instructions.push("M=D".to_string());
                }
                MemorySegment::Constant => panic!("Cannot pop from Constant"),
            }
        }
        OperationArgs::Add | OperationArgs::Sub | OperationArgs::And | OperationArgs::Or => {
            address_top_stack!(asm_instructions);
            asm_instructions.push("D=M".to_string());
            address_top_stack!(asm_instructions);
            match operation_args {
                OperationArgs::Add => asm_instructions.push("M=D+M".to_string()),
                OperationArgs::Sub => asm_instructions.push("M=M-D".to_string()),
                OperationArgs::And => asm_instructions.push("M=D&M".to_string()),
                OperationArgs::Or => asm_instructions.push("M=D|M".to_string()),
                _ => (),
            }
            incr_stack_pointer!(asm_instructions);
        }
        OperationArgs::Neg => {
            address_top_stack!(asm_instructions);
            asm_instructions.push("M=-M".to_string());
            incr_stack_pointer!(asm_instructions);
        }
        OperationArgs::Not => {
            address_top_stack!(asm_instructions);
            asm_instructions.push("M=!M".to_string());
            incr_stack_pointer!(asm_instructions);
        }
        OperationArgs::Eq | OperationArgs::Gt | OperationArgs::Lt => {
            address_top_stack!(asm_instructions);
            asm_instructions.push("D=M".to_string());
            address_top_stack!(asm_instructions);
            asm_instructions.push("D=M-D".to_string());
            asm_instructions.push("@PUSH_TRUE".to_string());

            match operation_args {
                OperationArgs::Eq => asm_instructions.push("D;JEQ".to_string()),
                OperationArgs::Lt => asm_instructions.push("D;JLT".to_string()),
                OperationArgs::Gt => asm_instructions.push("D;JGT".to_string()),
                _ => {}
            }

            asm_instructions.push("(PUSH_FALSE)".to_string());
            asm_instructions.push("@SP".to_string());
            asm_instructions.push("A=M".to_string());
            asm_instructions.push("M=0".to_string());
            asm_instructions.push("@NO_OP".to_string());
            asm_instructions.push("0;JMP".to_string());

            asm_instructions.push("(PUSH_TRUE)".to_string());
            asm_instructions.push("@SP".to_string());
            asm_instructions.push("A=M".to_string());
            asm_instructions.push("M=-1".to_string());

            asm_instructions.push("(NO_OP)".to_string());
            incr_stack_pointer!(asm_instructions);
        }
    }
}

pub fn generate_asm(vm_command: &Command, program_name: &str) -> Vec<String> {
    let mut asm_instructions: Vec<String> = vec![];

    match vm_command {
        Command::Branching(branching_args) => {
            generate_branching_asm(branching_args, &mut asm_instructions)
        }
        Command::Function => panic!("TODO"),
        Command::Operation(operation_args) => {
            generate_operation_asm(operation_args, &mut asm_instructions, program_name);
        }
    }

    asm_instructions
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
}
