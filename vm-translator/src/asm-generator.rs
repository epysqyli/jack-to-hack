use crate::parser::Command;
use crate::parser::branching::BranchingCommand;
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

macro_rules! decr_stack_pointer {
    ($asm_instructions: ident) => {
        $asm_instructions.push("@SP".to_string());
        $asm_instructions.push("M=M-1".to_string());
    };
}

pub fn generate_asm(vm_command: &Command, program_name: &str) -> Vec<String> {
    let mut asm_instructions: Vec<String> = vec![];

    match vm_command {
        Command::Branching(branching_args) => match branching_args.cmd {
            BranchingCommand::Label => {
                asm_instructions.push(format!("({})", branching_args.label));
            }
            BranchingCommand::Goto => panic!("TODO"),
            BranchingCommand::IfGoto => panic!("TODO"),
        },
        Command::Function => panic!("TODO"),
        Command::Operation(operation_args) => {
            match operation_args.op {
                Operation::Push => {
                    match &operation_args.segment {
                        Some(mem_segment) => {
                            match mem_segment {
                                MemorySegment::Constant => {
                                    if operation_args.val.is_none() {
                                        panic!(
                                            "Push operations require a value to push on the stack"
                                        )
                                    }
                                    asm_instructions
                                        .push(format!("@{}", operation_args.val.unwrap()));
                                    asm_instructions.push("D=A".to_string());
                                    assign_d_reg_to_stack!(asm_instructions);
                                    incr_stack_pointer!(asm_instructions);
                                }
                                MemorySegment::Local
                                | MemorySegment::Argument
                                | MemorySegment::This
                                | MemorySegment::That => {
                                    if operation_args.val.is_none() {
                                        panic!(
                                            "Push operations from memory segments require an index"
                                        )
                                    }
                                    asm_instructions
                                        .push(format!("@{}", operation_args.val.unwrap()));
                                    asm_instructions.push("D=A".to_string());
                                    asm_instructions.push(
                                        operation_args.segment.as_ref().unwrap().as_asm_mnemonic(),
                                    );
                                    asm_instructions.push("A=D+M".to_string());
                                    asm_instructions.push("D=M".to_string());
                                    assign_d_reg_to_stack!(asm_instructions);
                                    incr_stack_pointer!(asm_instructions);
                                }
                                MemorySegment::Temp => {
                                    if operation_args.val.is_none() {
                                        panic!(
                                            "Push operations on TEMP require a memory segment index"
                                        )
                                    }
                                    // TEMP address range is 5..12
                                    asm_instructions
                                        .push(format!("@R{}", 5 + operation_args.val.unwrap()));
                                    asm_instructions.push("D=M".to_string());
                                    assign_d_reg_to_stack!(asm_instructions);
                                    incr_stack_pointer!(asm_instructions);
                                }
                                MemorySegment::Static => {
                                    if operation_args.val.is_none() {
                                        panic!(
                                            "Push operations from static require a numeric value"
                                        )
                                    }
                                    asm_instructions.push(format!(
                                        "@{}.{}",
                                        program_name,
                                        operation_args.val.unwrap()
                                    ));
                                    asm_instructions.push("D=M".to_string());
                                    assign_d_reg_to_stack!(asm_instructions);
                                    incr_stack_pointer!(asm_instructions);
                                }
                                MemorySegment::Pointer => {
                                    if operation_args.val.is_none() {
                                        panic!("Push from pointer requires index 0 or 1")
                                    }
                                    match operation_args.val.unwrap() {
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
                            }
                        }
                        None => panic!("Memory Segment is mandatory for push operations"),
                    };
                }
                Operation::Pop => {
                    match &operation_args.segment {
                        Some(mem_segment) => {
                            match mem_segment {
                                MemorySegment::Local
                                | MemorySegment::Argument
                                | MemorySegment::This
                                | MemorySegment::That => {
                                    address_top_stack!(asm_instructions);
                                    asm_instructions.push("D=M".to_string());
                                    asm_instructions.push("@R13".to_string());
                                    asm_instructions.push("M=D".to_string());

                                    if operation_args.val.is_none() {
                                        panic!("Pop operations require a memory segment index")
                                    }

                                    asm_instructions
                                        .push(format!("@{}", operation_args.val.unwrap()));
                                    asm_instructions.push("D=A".to_string());
                                    asm_instructions.push(
                                        operation_args.segment.as_ref().unwrap().as_asm_mnemonic(),
                                    );
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
                                    if operation_args.val.is_none() {
                                        panic!("Pop operations require a memory segment index")
                                    }
                                    // TEMP address range is 5..12
                                    asm_instructions
                                        .push(format!("@R{}", 5 + operation_args.val.unwrap()));
                                    asm_instructions.push("M=D".to_string());
                                }
                                MemorySegment::Static => {
                                    address_top_stack!(asm_instructions);
                                    asm_instructions.push("D=M".to_string());
                                    if operation_args.val.is_none() {
                                        panic!("Pop operations on static require a numeric value")
                                    }
                                    asm_instructions.push(format!(
                                        "@{}.{}",
                                        program_name,
                                        operation_args.val.unwrap()
                                    ));
                                    asm_instructions.push("M=D".to_string());
                                }
                                MemorySegment::Pointer => {
                                    address_top_stack!(asm_instructions);
                                    asm_instructions.push("D=M".to_string());
                                    if operation_args.val.is_none() {
                                        panic!("Pop operations on static require a numeric value")
                                    }
                                    match operation_args.val.unwrap() {
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
                        None => panic!("Memory Segment is mandatory for pop operations"),
                    }
                }
                Operation::Add | Operation::Sub | Operation::And | Operation::Or => {
                    address_top_stack!(asm_instructions);
                    asm_instructions.push("D=M".to_string());
                    address_top_stack!(asm_instructions);
                    match operation_args.op {
                        Operation::Add => asm_instructions.push("M=D+M".to_string()),
                        Operation::Sub => asm_instructions.push("M=M-D".to_string()),
                        Operation::And => asm_instructions.push("M=D&M".to_string()),
                        Operation::Or => asm_instructions.push("M=D|M".to_string()),
                        _ => (),
                    }
                    decr_stack_pointer!(asm_instructions);
                }
                Operation::Neg => {
                    address_top_stack!(asm_instructions);
                    asm_instructions.push("M=-M".to_string());
                }
                Operation::Not => {
                    address_top_stack!(asm_instructions);
                    asm_instructions.push("M=!M".to_string());
                }
                Operation::Eq | Operation::Gt | Operation::Lt => {
                    address_top_stack!(asm_instructions);
                    asm_instructions.push("D=M".to_string());
                    address_top_stack!(asm_instructions);
                    asm_instructions.push("D=M-D".to_string());
                    asm_instructions.push("@PUSH_TRUE".to_string());

                    match operation_args.op {
                        Operation::Eq => asm_instructions.push("D;JEQ".to_string()),
                        Operation::Lt => asm_instructions.push("D;JLT".to_string()),
                        Operation::Gt => asm_instructions.push("D;JGT".to_string()),
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
    }

    asm_instructions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::branching::BranchingArgs;

    const TEST_PROGRAM_NAME: &'static str = "TestProgram";

    #[test]
    fn stack_push() {
        let vm_command: Command = Command::Operation(OperationArgs {
            op: Operation::Push,
            segment: Some(MemorySegment::Constant),
            val: Some(1),
        });

        assert_eq!(
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            generate_asm(&vm_command, TEST_PROGRAM_NAME)
        );
    }

    #[test]
    #[should_panic]
    fn push_without_value_panics() {
        let vm_command: Command = Command::Operation(OperationArgs {
            op: Operation::Push,
            segment: Some(MemorySegment::Constant),
            val: None,
        });

        generate_asm(&vm_command, TEST_PROGRAM_NAME);
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
            &Command::Operation(OperationArgs {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(1),
            }),
            &Command::Operation(OperationArgs {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(2),
            }),
            &Command::Operation(OperationArgs {
                op: Operation::Add,
                segment: None,
                val: None,
            }),
        ];

        let expected_asm: Vec<Vec<&str>> = vec![
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@2", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec![
                "@SP", "M=M-1", "A=M", "D=M", "@SP", "M=M-1", "A=M", "M=D+M", "@SP", "M=M-1",
            ],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn stack_double_push_and_sub() {
        let vm_commands: Vec<&Command> = vec![
            &Command::Operation(OperationArgs {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(1),
            }),
            &Command::Operation(OperationArgs {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(2),
            }),
            &Command::Operation(OperationArgs {
                op: Operation::Sub,
                segment: None,
                val: None,
            }),
        ];

        let expected_asm: Vec<Vec<&str>> = vec![
            // start from stack[0], assign D to M, incr stack
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            // start from stack[1], assign D to M, incr stack
            vec!["@2", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec![
                "@SP", "M=M-1", "A=M", "D=M", "@SP", "M=M-1", "A=M", "M=M-D", "@SP", "M=M-1",
            ],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn stack_double_push_and_logical_and() {
        let vm_commands: Vec<&Command> = vec![
            &Command::Operation(OperationArgs {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(1),
            }),
            &Command::Operation(OperationArgs {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(2),
            }),
            &Command::Operation(OperationArgs {
                op: Operation::And,
                segment: None,
                val: None,
            }),
        ];

        let expected_asm: Vec<Vec<&str>> = vec![
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@2", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec![
                "@SP", "M=M-1", "A=M", "D=M", "@SP", "M=M-1", "A=M", "M=D&M", "@SP", "M=M-1",
            ],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn stack_double_push_and_logical_or() {
        let vm_commands: Vec<&Command> = vec![
            &Command::Operation(OperationArgs {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(1),
            }),
            &Command::Operation(OperationArgs {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(2),
            }),
            &Command::Operation(OperationArgs {
                op: Operation::Or,
                segment: None,
                val: None,
            }),
        ];

        let expected_asm: Vec<Vec<&str>> = vec![
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@2", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec![
                "@SP", "M=M-1", "A=M", "D=M", "@SP", "M=M-1", "A=M", "M=D|M", "@SP", "M=M-1",
            ],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn single_stack_push_and_neg() {
        let vm_commands: Vec<&Command> = vec![
            &Command::Operation(OperationArgs {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(1),
            }),
            &Command::Operation(OperationArgs {
                op: Operation::Neg,
                segment: None,
                val: None,
            }),
        ];

        let expected_asm: Vec<Vec<&str>> = vec![
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@SP", "M=M-1", "A=M", "M=-M"],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn single_stack_push_and_not() {
        let vm_commands: Vec<&Command> = vec![
            &Command::Operation(OperationArgs {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(1),
            }),
            &Command::Operation(OperationArgs {
                op: Operation::Not,
                segment: None,
                val: None,
            }),
        ];

        let expected_asm: Vec<Vec<&str>> = vec![
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@SP", "M=M-1", "A=M", "M=!M"],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn stack_double_push_and_eq() {
        let vm_commands: Vec<&Command> = vec![
            &Command::Operation(OperationArgs {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(1),
            }),
            &Command::Operation(OperationArgs {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(2),
            }),
            &Command::Operation(OperationArgs {
                op: Operation::Eq,
                segment: None,
                val: None,
            }),
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
            &Command::Operation(OperationArgs {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(1),
            }),
            &Command::Operation(OperationArgs {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(2),
            }),
            &Command::Operation(OperationArgs {
                op: Operation::Lt,
                segment: None,
                val: None,
            }),
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
            &Command::Operation(OperationArgs {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(1),
            }),
            &Command::Operation(OperationArgs {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(2),
            }),
            &Command::Operation(OperationArgs {
                op: Operation::Gt,
                segment: None,
                val: None,
            }),
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
    fn push_to_stack_and_pop_from_stack_to_memory_segment() {
        let push_cmd = Command::Operation(OperationArgs {
            op: Operation::Push,
            segment: Some(MemorySegment::Constant),
            val: Some(1),
        });

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
                    &Command::Operation(OperationArgs {
                        op: Operation::Pop,
                        segment: Some(memory_segment),
                        val: Some(5),
                    }),
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
            &Command::Operation(OperationArgs {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(5),
            }),
            &Command::Operation(OperationArgs {
                op: Operation::Pop,
                segment: Some(MemorySegment::Local),
                val: Some(2),
            }),
            &Command::Operation(OperationArgs {
                op: Operation::Push,
                segment: Some(MemorySegment::Local),
                val: Some(2),
            }),
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
            &Command::Operation(OperationArgs {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(3),
            }),
            &Command::Operation(OperationArgs {
                op: Operation::Pop,
                segment: Some(MemorySegment::Temp),
                val: Some(4),
            }),
            &Command::Operation(OperationArgs {
                op: Operation::Push,
                segment: Some(MemorySegment::Temp),
                val: Some(4),
            }),
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
            &Command::Operation(OperationArgs {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(5),
            }),
            &Command::Operation(OperationArgs {
                op: Operation::Pop,
                segment: Some(MemorySegment::Static),
                val: Some(1),
            }),
            &Command::Operation(OperationArgs {
                op: Operation::Push,
                segment: Some(MemorySegment::Static),
                val: Some(1),
            }),
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
            &Command::Operation(OperationArgs {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(5),
            }),
            &Command::Operation(OperationArgs {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(6),
            }),
            &Command::Operation(OperationArgs {
                op: Operation::Pop,
                segment: Some(MemorySegment::Pointer),
                val: Some(0),
            }),
            &Command::Operation(OperationArgs {
                op: Operation::Pop,
                segment: Some(MemorySegment::Pointer),
                val: Some(1),
            }),
            &Command::Operation(OperationArgs {
                op: Operation::Push,
                segment: Some(MemorySegment::Pointer),
                val: Some(0),
            }),
            &Command::Operation(OperationArgs {
                op: Operation::Push,
                segment: Some(MemorySegment::Pointer),
                val: Some(1),
            }),
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
    fn define_label() {
        let label = "TEST".to_string();

        let vm_commands: Vec<Command> = vec![Command::Branching(BranchingArgs {
            cmd: BranchingCommand::Label,
            label: label,
        })];

        let expected_asm = vec![vec!["(TEST)"]];

        assert_commands_eq(vec![&vm_commands[0]], expected_asm);
    }
}
