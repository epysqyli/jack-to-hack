use crate::parser::{Command, MemorySegment, Operation};

macro_rules! address_top_stack {
    ($asm_instructions: ident) => {
        $asm_instructions.push("@SP".to_string());
        $asm_instructions.push("M=M-1".to_string());
        $asm_instructions.push("A=M".to_string());
    };
}

macro_rules! incr_stack_pointer {
    ($asm_instructions: ident) => {
        $asm_instructions.push("@SP".to_string());
        $asm_instructions.push("M=M+1".to_string());
    };
}

pub fn generate_asm(vm_command: Command, program_name: &str) -> Vec<String> {
    let mut asm_instructions: Vec<String> = vec![];

    match vm_command.op {
        Operation::Push => {
            match vm_command.segment {
                Some(MemorySegment::Constant) => {
                    if let Some(val) = vm_command.val {
                        asm_instructions.push(format!("@{val}"));
                        asm_instructions.push("D=A".to_string());
                    } else {
                        panic!("Push operations require a value to push on the stack")
                    }
                    asm_instructions.push("@SP".to_string());
                    asm_instructions.push("A=M".to_string());
                    asm_instructions.push("M=D".to_string());
                    incr_stack_pointer!(asm_instructions);
                }
                Some(
                    MemorySegment::Local
                    | MemorySegment::Argument
                    | MemorySegment::This
                    | MemorySegment::That,
                ) => {
                    if let Some(val) = vm_command.val {
                        asm_instructions.push(format!("@{val}"));
                        asm_instructions.push("D=A".to_string());
                    } else {
                        panic!("Push operations require a value to push on the stack")
                    }
                    asm_instructions.push(vm_command.segment.unwrap().as_asm_mnemonic());
                    asm_instructions.push("A=D+M".to_string());
                    asm_instructions.push("D=M".to_string());
                    asm_instructions.push("@SP".to_string());
                    asm_instructions.push("A=M".to_string());
                    asm_instructions.push("M=D".to_string());
                    incr_stack_pointer!(asm_instructions);
                }
                Some(MemorySegment::Temp) => {
                    if let Some(val) = vm_command.val {
                        asm_instructions.push(format!("@R{}", 5 + val)); // TEMP address range is 5..12
                        asm_instructions.push("D=M".to_string());
                    } else {
                        panic!("Push operations on TEMP require a memory segment index")
                    }
                    asm_instructions.push("@SP".to_string());
                    asm_instructions.push("A=M".to_string());
                    asm_instructions.push("M=D".to_string());
                    incr_stack_pointer!(asm_instructions);
                }
                Some(MemorySegment::Static) => {
                    if let Some(val) = vm_command.val {
                        asm_instructions.push(format!("@{}.{}", program_name, val));
                    } else {
                        panic!("Push operations from static require a numeric value")
                    }
                    asm_instructions.push("D=M".to_string());
                    asm_instructions.push("@SP".to_string());
                    asm_instructions.push("A=M".to_string());
                    asm_instructions.push("M=D".to_string());
                    incr_stack_pointer!(asm_instructions);
                }
                Some(MemorySegment::Pointer) => {
                    if let Some(val) = vm_command.val {
                        match val {
                            0 => asm_instructions.push(format!("@THIS")),
                            1 => asm_instructions.push(format!("@THAT")),
                            _ => panic!("Pop operations on pointer allow values 0 or 1"),
                        }
                    } else {
                        panic!("Push from pointer requires index 0 or 1")
                    }
                    asm_instructions.push("D=M".to_string());
                    asm_instructions.push("@SP".to_string());
                    asm_instructions.push("A=M".to_string());
                    asm_instructions.push("M=D".to_string());
                    incr_stack_pointer!(asm_instructions);
                }
                None => panic!("Memory Segment is mandatory for push operations"),
            };
        }
        Operation::Pop => {
            match vm_command.segment {
                Some(
                    MemorySegment::Local
                    | MemorySegment::Argument
                    | MemorySegment::This
                    | MemorySegment::That,
                ) => {
                    address_top_stack!(asm_instructions);
                    asm_instructions.push("D=M".to_string());
                    asm_instructions.push("@R13".to_string());
                    asm_instructions.push("M=D".to_string());

                    if let Some(val) = vm_command.val {
                        asm_instructions.push(format!("@{val}"));
                        asm_instructions.push("D=A".to_string());
                    } else {
                        panic!("Pop operations require a memory segment index")
                    }

                    asm_instructions.push(vm_command.segment.unwrap().as_asm_mnemonic());
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
                Some(MemorySegment::Temp) => {
                    address_top_stack!(asm_instructions);
                    asm_instructions.push("D=M".to_string());
                    if let Some(val) = vm_command.val {
                        asm_instructions.push(format!("@R{}", 5 + val)); // TEMP address range is 5..12
                        asm_instructions.push("M=D".to_string());
                    } else {
                        panic!("Pop operations require a memory segment index")
                    }
                }
                Some(MemorySegment::Static) => {
                    address_top_stack!(asm_instructions);
                    asm_instructions.push("D=M".to_string());
                    if let Some(val) = vm_command.val {
                        asm_instructions.push(format!("@{}.{}", program_name, val));
                    } else {
                        panic!("Pop operations on static require a numeric value")
                    }
                    asm_instructions.push("M=D".to_string());
                }
                Some(MemorySegment::Pointer) => {
                    address_top_stack!(asm_instructions);
                    asm_instructions.push("D=M".to_string());
                    if let Some(val) = vm_command.val {
                        match val {
                            0 => asm_instructions.push(format!("@THIS")),
                            1 => asm_instructions.push(format!("@THAT")),
                            _ => panic!("Pop operations on pointer allow values 0 or 1"),
                        }
                    } else {
                        panic!("Pop operations on static require a numeric value")
                    }
                    asm_instructions.push("M=D".to_string());
                }
                Some(MemorySegment::Constant) => panic!("Cannot pop from Constant"),
                None => panic!("Memory Segment is mandatory for pop operations"),
            }
        }
        Operation::Add | Operation::Sub | Operation::And | Operation::Or => {
            address_top_stack!(asm_instructions);
            asm_instructions.push("D=M".to_string());
            address_top_stack!(asm_instructions);
            match vm_command.op {
                Operation::Add => asm_instructions.push("M=D+M".to_string()),
                Operation::Sub => asm_instructions.push("M=M-D".to_string()),
                Operation::And => asm_instructions.push("M=D&M".to_string()),
                Operation::Or => asm_instructions.push("M=D|M".to_string()),
                _ => (),
            }
            incr_stack_pointer!(asm_instructions);
        }
        Operation::Neg => {
            address_top_stack!(asm_instructions);
            asm_instructions.push("M=-M".to_string());
            incr_stack_pointer!(asm_instructions);
        }
        Operation::Not => {
            address_top_stack!(asm_instructions);
            asm_instructions.push("M=!M".to_string());
            incr_stack_pointer!(asm_instructions);
        }
        Operation::Eq | Operation::Gt | Operation::Lt => {
            address_top_stack!(asm_instructions);
            asm_instructions.push("D=M".to_string());
            address_top_stack!(asm_instructions);
            asm_instructions.push("D=M-D".to_string());
            asm_instructions.push("@PUSH_TRUE".to_string());

            match vm_command.op {
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

    asm_instructions
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PROGRAM_NAME: &'static str = "TestProgram";

    #[test]
    fn stack_push() {
        let vm_command: Command = Command {
            op: Operation::Push,
            segment: Some(MemorySegment::Constant),
            val: Some(1),
        };

        assert_eq!(
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            generate_asm(vm_command, TEST_PROGRAM_NAME)
        );
    }

    #[test]
    #[should_panic]
    fn push_without_value_panics() {
        let vm_command: Command = Command {
            op: Operation::Push,
            segment: Some(MemorySegment::Constant),
            val: None,
        };

        generate_asm(vm_command, TEST_PROGRAM_NAME);
    }

    fn assert_commands_eq(vm_commands: Vec<Command>, expected_asm: Vec<Vec<&str>>) {
        vm_commands
            .iter()
            .zip(expected_asm)
            // Can we use Rc here instead of deriving Copy+Clone on Command?
            .for_each(|(vm_command, expected)| {
                assert_eq!(expected, generate_asm(*vm_command, TEST_PROGRAM_NAME))
            });
    }

    #[test]
    fn stack_double_push_and_add() {
        let vm_commands: Vec<Command> = vec![
            Command {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(1),
            },
            Command {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(2),
            },
            Command {
                op: Operation::Add,
                segment: None,
                val: None,
            },
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
        let vm_commands: Vec<Command> = vec![
            Command {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(1),
            },
            Command {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(2),
            },
            Command {
                op: Operation::Sub,
                segment: None,
                val: None,
            },
        ];

        let expected_asm: Vec<Vec<&str>> = vec![
            // start from stack[0], assign D to M, incr stack
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            // start from stack[1], assign D to M, incr stack
            vec!["@2", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec![
                "@SP", "M=M-1", "A=M", "D=M", "@SP", "M=M-1", "A=M", "M=M-D", "@SP", "M=M+1",
            ],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn stack_double_push_and_logical_and() {
        let vm_commands: Vec<Command> = vec![
            Command {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(1),
            },
            Command {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(2),
            },
            Command {
                op: Operation::And,
                segment: None,
                val: None,
            },
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
        let vm_commands: Vec<Command> = vec![
            Command {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(1),
            },
            Command {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(2),
            },
            Command {
                op: Operation::Or,
                segment: None,
                val: None,
            },
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
        let vm_commands: Vec<Command> = vec![
            Command {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(1),
            },
            Command {
                op: Operation::Neg,
                segment: None,
                val: None,
            },
        ];

        let expected_asm: Vec<Vec<&str>> = vec![
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@SP", "M=M-1", "A=M", "M=-M", "@SP", "M=M+1"],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn single_stack_push_and_not() {
        let vm_commands: Vec<Command> = vec![
            Command {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(1),
            },
            Command {
                op: Operation::Not,
                segment: None,
                val: None,
            },
        ];

        let expected_asm: Vec<Vec<&str>> = vec![
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@SP", "M=M-1", "A=M", "M=!M", "@SP", "M=M+1"],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn stack_double_push_and_eq() {
        let vm_commands: Vec<Command> = vec![
            Command {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(1),
            },
            Command {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(2),
            },
            Command {
                op: Operation::Eq,
                segment: None,
                val: None,
            },
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
        let vm_commands: Vec<Command> = vec![
            Command {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(1),
            },
            Command {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(2),
            },
            Command {
                op: Operation::Lt,
                segment: None,
                val: None,
            },
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
        let vm_commands: Vec<Command> = vec![
            Command {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(1),
            },
            Command {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(2),
            },
            Command {
                op: Operation::Gt,
                segment: None,
                val: None,
            },
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
        let push_cmd = Command {
            op: Operation::Push,
            segment: Some(MemorySegment::Constant),
            val: Some(1),
        };

        let memory_segments = [
            MemorySegment::Local,
            MemorySegment::Argument,
            MemorySegment::This,
            MemorySegment::That,
        ];

        memory_segments.iter().for_each(|memory_segment| {
            assert_commands_eq(
                vec![
                    push_cmd,
                    Command {
                        op: Operation::Pop,
                        segment: Some(*memory_segment),
                        val: Some(5),
                    },
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
                        &memory_segment.as_asm_mnemonic(),
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
        let vm_commands: Vec<Command> = vec![
            Command {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(5),
            },
            Command {
                op: Operation::Pop,
                segment: Some(MemorySegment::Local),
                val: Some(2),
            },
            Command {
                op: Operation::Push,
                segment: Some(MemorySegment::Local),
                val: Some(2),
            },
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
        let vm_commands: Vec<Command> = vec![
            Command {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(3),
            },
            Command {
                op: Operation::Pop,
                segment: Some(MemorySegment::Temp),
                val: Some(4),
            },
            Command {
                op: Operation::Push,
                segment: Some(MemorySegment::Temp),
                val: Some(4),
            },
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
        let vm_commands: Vec<Command> = vec![
            Command {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(5),
            },
            Command {
                op: Operation::Pop,
                segment: Some(MemorySegment::Static),
                val: Some(1),
            },
            Command {
                op: Operation::Push,
                segment: Some(MemorySegment::Static),
                val: Some(1),
            },
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
        let vm_commands: Vec<Command> = vec![
            Command {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(5),
            },
            Command {
                op: Operation::Push,
                segment: Some(MemorySegment::Constant),
                val: Some(6),
            },
            Command {
                op: Operation::Pop,
                segment: Some(MemorySegment::Pointer),
                val: Some(0),
            },
            Command {
                op: Operation::Pop,
                segment: Some(MemorySegment::Pointer),
                val: Some(1),
            },
            Command {
                op: Operation::Push,
                segment: Some(MemorySegment::Pointer),
                val: Some(0),
            },
            Command {
                op: Operation::Push,
                segment: Some(MemorySegment::Pointer),
                val: Some(1),
            },
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
}
