use std::collections::HashMap;

use crate::command::{
    Command,
    branching::BranchingArgs,
    function::FunctionArgs,
    operation::{MemorySegment, OperationArgs},
};

pub fn compile(vm_commands: Vec<Command>) -> Vec<String> {
    AsmGenerator::generate(vm_commands)
}

struct AsmGenerator {
    instructions: Vec<String>,
    function_calls: HashMap<String, usize>,
    counter: u16,
}

impl AsmGenerator {
    fn generate(vm_commands: Vec<Command>) -> Vec<String> {
        let mut asm_generator =
            Self { instructions: vec![], function_calls: HashMap::new(), counter: 0 };

        // Set SP to 256 as first bootstrapping step
        #[cfg(not(test))]
        {
            asm_generator.add("@256");
            asm_generator.add("D=A");
            asm_generator.add("@SP");
            asm_generator.add("M=D");
        }

        vm_commands.iter().for_each(|vm_command| match vm_command {
            Command::Branching(args) => asm_generator.generate_branching_asm(args),
            Command::Function(args) => asm_generator.generate_function_asm(args),
            Command::Operation(args) => asm_generator.generate_operation_asm(args),
        });

        #[cfg(not(test))]
        asm_generator.inject_global_return();
        #[cfg(not(test))]
        asm_generator.inject_save_caller_frame();

        asm_generator.instructions
    }

    fn add(self: &mut Self, cmd: &str) {
        self.instructions.push(cmd.to_string());
    }

    fn address_top_stack(self: &mut Self) {
        self.add("@SP");
        self.add("AM=M-1");
    }

    fn incr_stack_pointer(self: &mut Self) {
        self.add("@SP");
        self.add("M=M+1");
    }

    fn push_d_reg_to_stack(self: &mut Self) {
        self.add("@SP");
        self.add("A=M");
        self.add("M=D");
        self.incr_stack_pointer();
    }

    #[allow(dead_code)]
    fn inject_global_return(self: &mut Self) {
        self.add("(GLOBAL_RETURN)");

        // frame = LCL: define frame as temp variable R5 and assign LCL to it
        self.add("@LCL");
        self.add("D=M");
        self.add("@R5");
        self.add("M=D");

        // save the return address (M[LCL] - 5) to temp variable R6
        self.add("@5");
        self.add("A=D-A");
        self.add("D=M");
        self.add("@R6");
        self.add("M=D");

        // reposition return value for the caller: pop from the stack to ARG
        self.address_top_stack();
        self.add("D=M");
        self.add("@ARG");
        self.add("A=M");
        self.add("M=D");

        // reposition SP for the caller to @ARG + 1
        self.add("@ARG");
        self.add("D=M+1");
        self.add("@SP");
        self.add("M=D");

        for (i, mem_segment) in ["@THAT", "@THIS", "@ARG", "@LCL"].iter().enumerate() {
            self.add("@R5");
            self.add("D=M");
            for _ in 1..=i {
                self.add("D=D-1");
            }
            self.add("A=D-1");
            self.add("D=M");
            self.add(mem_segment);
            self.add("M=D");
        }

        // goto return address
        self.add("@R6");
        self.add("A=M");
        self.add("0;JMP");
    }

    #[allow(dead_code)]
    fn inject_save_caller_frame(self: &mut Self) {
        self.add("(SAVE_CALLER_FRAME)");

        ["@LCL", "@ARG", "@THIS", "@THAT"].iter().for_each(|mem_segment| {
            self.add(mem_segment);
            self.add("D=M");
            self.push_d_reg_to_stack();
        });

        self.add("@SP");
        self.add("D=M");
        self.add("@R14");
        self.add("A=M");
        self.add("D=D-A");
        self.add("@ARG");
        self.add("M=D");

        self.add("@SP");
        self.add("D=M");
        self.add("@LCL");
        self.add("M=D");

        // Jump back to the remaining call instructions
        self.add("@R15");
        self.add("A=M");
        self.add("0;JMP");
    }

    fn generate_branching_asm(self: &mut Self, args: &BranchingArgs) {
        match args {
            BranchingArgs::Label(label, fn_name) => {
                self.add(format!("({}${})", fn_name, label).as_str());
            }
            BranchingArgs::Goto(label, fn_name) => {
                self.add(format!("@{}${}", fn_name, label).as_str());
                self.add("0;JMP");
            }
            BranchingArgs::IfGoto(label, fn_name) => {
                self.address_top_stack();
                self.add("D=M");
                self.add(format!("@{}${}", fn_name, label).as_str());
                self.add("D;JNE");
            }
        }
    }

    fn generate_function_asm(self: &mut Self, args: &FunctionArgs) {
        match args {
            FunctionArgs::Function(fn_name, n_local_vars) => {
                self.add(format!("({})", fn_name).as_str());
                for i in 0..*n_local_vars {
                    self.add(format!("@{}", i).as_str());
                    self.add("D=A");
                    self.add("@LCL");
                    self.add("A=D+M");
                    self.add("M=0");
                    self.incr_stack_pointer();
                }
            }
            FunctionArgs::Call(fn_name, n_caller_args) => {
                let call_depth: usize = match self.function_calls.get_mut(fn_name) {
                    None => {
                        self.function_calls.insert(fn_name.clone(), 1);
                        0
                    }
                    Some(v) => {
                        *v += 1;
                        *v - 1
                    }
                };

                // Generate the return label instruction line number and push it to the stack
                self.add(format!("@{}$ret.{}", fn_name, call_depth).as_str());
                self.add("D=A");
                self.push_d_reg_to_stack();

                // Generate the return label instruction line number for the inner call logic
                self.add(format!("@{}$RetFromSaveCallerFrame${}", fn_name, call_depth).as_str());
                self.add("D=A");
                self.add("@R15");
                self.add("M=D");

                // Store (5 + n_caller_args) on @R14
                self.add(format!("@{}", 5 + n_caller_args).as_str());
                self.add("D=A");
                self.add("@R14");
                self.add("M=D");

                self.add("@SAVE_CALLER_FRAME");
                self.add("0;JMP");

                // The actual label being jumped to at the end by @{}_CORE_CALL_LOGIC
                self.add(format!("({}$RetFromSaveCallerFrame${})", fn_name, call_depth).as_str());

                self.add(format!("@{}", fn_name).as_str());
                self.add("0;JMP");

                // Actually place the return label, jumped to by @GLOBAL_RETURN
                self.add(format!("({}$ret.{})", fn_name, call_depth).as_str());
            }
            FunctionArgs::Return => {
                self.add("@GLOBAL_RETURN");
                self.add("0;JMP");
            }
        }
    }

    fn generate_operation_asm(self: &mut Self, args: &OperationArgs) {
        match args {
            OperationArgs::Push(mem_segment, val, filename) => {
                match mem_segment {
                    MemorySegment::Constant => {
                        self.add(format!("@{}", val).as_str());
                        self.add("D=A");
                        self.push_d_reg_to_stack();
                    }
                    MemorySegment::Local
                    | MemorySegment::Argument
                    | MemorySegment::This
                    | MemorySegment::That => {
                        self.add(format!("@{}", val).as_str());
                        self.add("D=A");
                        self.add(mem_segment.as_asm_mnemonic().as_str());
                        self.add("A=D+M");
                        self.add("D=M");
                        self.push_d_reg_to_stack();
                    }
                    MemorySegment::Temp => {
                        // TEMP address range is 5..12
                        self.add(format!("@R{}", 5 + val).as_str());
                        self.add("D=M");
                        self.push_d_reg_to_stack();
                    }
                    MemorySegment::Static => {
                        self.add(format!("@{}.{}", filename, val).as_str());
                        self.add("D=M");
                        self.push_d_reg_to_stack();
                    }
                    MemorySegment::Pointer => {
                        match val {
                            0 => self.add(format!("@THIS").as_str()),
                            1 => self.add(format!("@THAT").as_str()),
                            _ => panic!("Pop operations on pointer allow values 0 or 1"),
                        }
                        self.add("D=M");
                        self.push_d_reg_to_stack();
                    }
                };
            }
            OperationArgs::Pop(mem_segment, val, filename) => {
                match mem_segment {
                    MemorySegment::Local
                    | MemorySegment::Argument
                    | MemorySegment::This
                    | MemorySegment::That => {
                        /* Load memory segment index onto D */
                        self.add(format!("@{}", val).as_str());
                        self.add("D=A");

                        /* Load (memory segment base address + index) onto R13 */
                        self.add(mem_segment.as_asm_mnemonic().as_str());
                        self.add("D=D+M");
                        self.add("@R13");
                        self.add("M=D");

                        /* Pop top stack value onto memory segment index */
                        self.address_top_stack();
                        self.add("D=M");
                        self.add("@R13");
                        self.add("A=M");
                        self.add("M=D");
                    }
                    MemorySegment::Temp => {
                        self.address_top_stack();
                        self.add("D=M");
                        // TEMP address range is 5..12
                        self.add(format!("@R{}", 5 + val).as_str());
                        self.add("M=D");
                    }
                    MemorySegment::Static => {
                        self.address_top_stack();
                        self.add("D=M");
                        self.add(format!("@{}.{}", filename, val).as_str());
                        self.add("M=D");
                    }
                    MemorySegment::Pointer => {
                        self.address_top_stack();
                        self.add("D=M");
                        match val {
                            0 => self.add(format!("@THIS").as_str()),
                            1 => self.add(format!("@THAT").as_str()),
                            _ => panic!("Pop operations on pointer allow values 0 or 1"),
                        }
                        self.add("M=D");
                    }
                    MemorySegment::Constant => panic!("Cannot pop stack to itself"),
                }
            }
            OperationArgs::Add | OperationArgs::Sub | OperationArgs::And | OperationArgs::Or => {
                self.address_top_stack();
                self.add("D=M");
                self.address_top_stack();
                match args {
                    OperationArgs::Add => self.add("M=D+M"),
                    OperationArgs::Sub => self.add("M=M-D"),
                    OperationArgs::And => self.add("M=D&M"),
                    OperationArgs::Or => self.add("M=D|M"),
                    _ => (),
                }
                self.incr_stack_pointer();
            }
            OperationArgs::Neg => {
                self.address_top_stack();
                self.add("M=-M");
                self.incr_stack_pointer();
            }
            OperationArgs::Not => {
                self.address_top_stack();
                self.add("M=!M");
                self.incr_stack_pointer();
            }
            OperationArgs::Eq(fn_name)
            | OperationArgs::Gt(fn_name)
            | OperationArgs::Lt(fn_name) => {
                self.address_top_stack();
                self.add("D=M");
                self.address_top_stack();
                self.add("D=M-D");
                self.add(format!("@{}.PUSH_TRUE.{}", fn_name, self.counter).as_str());

                match args {
                    OperationArgs::Eq(_) => self.add("D;JEQ"),
                    OperationArgs::Lt(_) => self.add("D;JLT"),
                    OperationArgs::Gt(_) => self.add("D;JGT"),
                    _ => {}
                }

                self.add("@SP");
                self.add("A=M");
                self.add("M=0");
                self.add(format!("@{}.NO_OP.{}", fn_name, self.counter).as_str());
                self.add("0;JMP");

                self.add(format!("({}.PUSH_TRUE.{})", fn_name, self.counter).as_str());
                self.add("@SP");
                self.add("A=M");
                self.add("M=-1");

                self.add(format!("({}.NO_OP.{})", fn_name, self.counter).as_str());
                self.incr_stack_pointer();
                self.counter += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILENAME: &'static str = "Filename";

    fn assert_commands_eq(vm_commands: Vec<Command>, expected_asm: Vec<Vec<&str>>) {
        let expected: Vec<&str> = expected_asm.into_iter().flat_map(|asm| asm).collect();

        let actual = AsmGenerator::generate(vm_commands);

        assert_eq!(expected, actual);
    }

    #[test]
    fn stack_push() {
        assert_commands_eq(
            vec![Command::Operation(OperationArgs::Push(
                MemorySegment::Constant,
                1,
                self::FILENAME.to_string(),
            ))],
            vec![vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"]],
        );
    }

    #[test]
    fn stack_double_push_and_add() {
        let vm_commands: Vec<Command> = vec![
            Command::Operation(OperationArgs::Push(
                MemorySegment::Constant,
                1,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Push(
                MemorySegment::Constant,
                2,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Add),
        ];

        let expected_asm: Vec<Vec<&str>> = vec![
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@2", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@SP", "AM=M-1", "D=M", "@SP", "AM=M-1", "M=D+M", "@SP", "M=M+1"],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn stack_double_push_and_sub() {
        let vm_commands: Vec<Command> = vec![
            Command::Operation(OperationArgs::Push(
                MemorySegment::Constant,
                1,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Push(
                MemorySegment::Constant,
                2,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Sub),
        ];

        let expected_asm: Vec<Vec<&str>> = vec![
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@2", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@SP", "AM=M-1", "D=M", "@SP", "AM=M-1", "M=M-D", "@SP", "M=M+1"],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn stack_double_push_and_logical_and() {
        let vm_commands: Vec<Command> = vec![
            Command::Operation(OperationArgs::Push(
                MemorySegment::Constant,
                1,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Push(
                MemorySegment::Constant,
                2,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::And),
        ];

        let expected_asm: Vec<Vec<&str>> = vec![
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@2", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@SP", "AM=M-1", "D=M", "@SP", "AM=M-1", "M=D&M", "@SP", "M=M+1"],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn stack_double_push_and_logical_or() {
        let vm_commands: Vec<Command> = vec![
            Command::Operation(OperationArgs::Push(
                MemorySegment::Constant,
                1,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Push(
                MemorySegment::Constant,
                2,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Or),
        ];

        let expected_asm: Vec<Vec<&str>> = vec![
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@2", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@SP", "AM=M-1", "D=M", "@SP", "AM=M-1", "M=D|M", "@SP", "M=M+1"],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn single_stack_push_and_neg() {
        let vm_commands: Vec<Command> = vec![
            Command::Operation(OperationArgs::Push(
                MemorySegment::Constant,
                1,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Neg),
        ];

        let expected_asm: Vec<Vec<&str>> = vec![
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@SP", "AM=M-1", "M=-M", "@SP", "M=M+1"],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn single_stack_push_and_not() {
        let vm_commands: Vec<Command> = vec![
            Command::Operation(OperationArgs::Push(
                MemorySegment::Constant,
                1,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Not),
        ];

        let expected_asm: Vec<Vec<&str>> = vec![
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@SP", "AM=M-1", "M=!M", "@SP", "M=M+1"],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn stack_double_push_and_eq() {
        let vm_commands: Vec<Command> = vec![
            Command::Operation(OperationArgs::Push(
                MemorySegment::Constant,
                1,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Push(
                MemorySegment::Constant,
                2,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Eq("TestFunction".to_string())),
        ];

        let expected_asm: Vec<Vec<&str>> = vec![
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@2", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec![
                "@SP",
                "AM=M-1",
                "D=M",
                "@SP",
                "AM=M-1",
                "D=M-D",
                "@TestFunction.PUSH_TRUE.0",
                "D;JEQ",
                "@SP",
                "A=M",
                "M=0",
                "@TestFunction.NO_OP.0",
                "0;JMP",
                "(TestFunction.PUSH_TRUE.0)",
                "@SP",
                "A=M",
                "M=-1",
                "(TestFunction.NO_OP.0)",
                "@SP",
                "M=M+1",
            ],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn stack_double_push_and_lt() {
        let vm_commands: Vec<Command> = vec![
            Command::Operation(OperationArgs::Push(
                MemorySegment::Constant,
                1,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Push(
                MemorySegment::Constant,
                2,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Lt("TestFunction".to_string())),
        ];

        let expected_asm: Vec<Vec<&str>> = vec![
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@2", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec![
                "@SP",
                "AM=M-1",
                "D=M",
                "@SP",
                "AM=M-1",
                "D=M-D",
                "@TestFunction.PUSH_TRUE.0",
                "D;JLT",
                "@SP",
                "A=M",
                "M=0",
                "@TestFunction.NO_OP.0",
                "0;JMP",
                "(TestFunction.PUSH_TRUE.0)",
                "@SP",
                "A=M",
                "M=-1",
                "(TestFunction.NO_OP.0)",
                "@SP",
                "M=M+1",
            ],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn stack_double_push_and_gt() {
        let vm_commands: Vec<Command> = vec![
            Command::Operation(OperationArgs::Push(
                MemorySegment::Constant,
                1,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Push(
                MemorySegment::Constant,
                2,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Gt("TestFunction".to_string())),
        ];

        let expected_asm: Vec<Vec<&str>> = vec![
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@2", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec![
                "@SP",
                "AM=M-1",
                "D=M",
                "@SP",
                "AM=M-1",
                "D=M-D",
                "@TestFunction.PUSH_TRUE.0",
                "D;JGT",
                "@SP",
                "A=M",
                "M=0",
                "@TestFunction.NO_OP.0",
                "0;JMP",
                "(TestFunction.PUSH_TRUE.0)",
                "@SP",
                "A=M",
                "M=-1",
                "(TestFunction.NO_OP.0)",
                "@SP",
                "M=M+1",
            ],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn push_twice_add_push_and_sub() {
        let vm_commands: Vec<Command> = vec![
            Command::Operation(OperationArgs::Push(
                MemorySegment::Constant,
                5,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Push(
                MemorySegment::Constant,
                5,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Add),
            Command::Operation(OperationArgs::Push(
                MemorySegment::Constant,
                10,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Sub),
        ];

        let expected_asm: Vec<Vec<&str>> = vec![
            vec!["@5", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@5", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@SP", "AM=M-1", "D=M", "@SP", "AM=M-1", "M=D+M", "@SP", "M=M+1"],
            vec!["@10", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@SP", "AM=M-1", "D=M", "@SP", "AM=M-1", "M=M-D", "@SP", "M=M+1"],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn push_to_stack_and_pop_from_stack_to_memory_segment() {
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
                    Command::Operation(OperationArgs::Push(
                        MemorySegment::Constant,
                        1,
                        self::FILENAME.to_string(),
                    )),
                    Command::Operation(OperationArgs::Pop(
                        memory_segment,
                        5,
                        self::FILENAME.to_string(),
                    )),
                ],
                vec![
                    vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
                    vec![
                        "@5",
                        "D=A",
                        mem_segment_asm,
                        "D=D+M",
                        "@R13",
                        "M=D",
                        "@SP",
                        "AM=M-1",
                        "D=M",
                        "@R13",
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
            Command::Operation(OperationArgs::Push(
                MemorySegment::Constant,
                5,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Pop(
                MemorySegment::Local,
                2,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Push(
                MemorySegment::Local,
                2,
                self::FILENAME.to_string(),
            )),
        ];

        let expected_asm = vec![
            vec!["@5", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec![
                "@2", "D=A", "@LCL", "D=D+M", "@R13", "M=D", "@SP", "AM=M-1", "D=M", "@R13", "A=M",
                "M=D",
            ],
            vec!["@2", "D=A", "@LCL", "A=D+M", "D=M", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn push_to_stack_pop_to_temp_and_push_from_temp_to_stack() {
        let vm_commands: Vec<Command> = vec![
            Command::Operation(OperationArgs::Push(
                MemorySegment::Constant,
                3,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Pop(
                MemorySegment::Temp,
                4,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Push(
                MemorySegment::Temp,
                4,
                self::FILENAME.to_string(),
            )),
        ];

        let expected_asm = vec![
            // push the constant 3 onto the stack
            vec!["@3", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            // pop 3 from stack and save it on temp index 4, i.e. memory address 9
            vec!["@SP", "AM=M-1", "D=M", "@R9", "M=D"],
            // // push to stack from temp index 4
            vec!["@R9", "D=M", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn push_to_stack_and_pop_to_static_and_push_back_to_stack() {
        let vm_commands: Vec<Command> = vec![
            Command::Operation(OperationArgs::Push(
                MemorySegment::Constant,
                5,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Pop(
                MemorySegment::Static,
                1,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Push(
                MemorySegment::Static,
                1,
                self::FILENAME.to_string(),
            )),
        ];

        let expected_asm = vec![
            vec!["@5", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@SP", "AM=M-1", "D=M", "@Filename.1", "M=D"],
            vec!["@Filename.1", "D=M", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn push_to_stack_and_pop_to_pointers() {
        let vm_commands: Vec<Command> = vec![
            Command::Operation(OperationArgs::Push(
                MemorySegment::Constant,
                5,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Push(
                MemorySegment::Constant,
                6,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Pop(
                MemorySegment::Pointer,
                0,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Pop(
                MemorySegment::Pointer,
                1,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Push(
                MemorySegment::Pointer,
                0,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Push(
                MemorySegment::Pointer,
                1,
                self::FILENAME.to_string(),
            )),
        ];

        let expected_asm = vec![
            vec!["@5", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@6", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@SP", "AM=M-1", "D=M", "@THIS", "M=D"],
            vec!["@SP", "AM=M-1", "D=M", "@THAT", "M=D"],
            vec!["@THIS", "D=M", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@THAT", "D=M", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn define_and_goto_label() {
        let label_definition = Command::Branching(BranchingArgs::Label(
            "TestLabel".to_string(),
            "TestFunction".to_string(),
        ));
        let goto_label = Command::Branching(BranchingArgs::Goto(
            "TestLabel".to_string(),
            "TestFunction".to_string(),
        ));

        assert_commands_eq(
            vec![label_definition, goto_label],
            vec![vec!["(TestFunction$TestLabel)"], vec!["@TestFunction$TestLabel", "0;JMP"]],
        );
    }

    #[test]
    fn identical_labels_are_different_within_different_functions() {
        let vm_commands = vec![
            Command::Branching(BranchingArgs::Label(
                "Test".to_string(),
                "FirstFunction".to_string(),
            )),
            Command::Branching(BranchingArgs::Label(
                "Test".to_string(),
                "SecondFunction".to_string(),
            )),
        ];

        let expected_asm = vec![vec!["(FirstFunction$Test)"], vec!["(SecondFunction$Test)"]];

        assert_commands_eq(
            vm_commands.into_iter().map(|vm_cmd| vm_cmd).collect::<Vec<Command>>(),
            expected_asm,
        );
    }

    #[test]
    fn define_and_if_goto_label() {
        let label_definition = Command::Branching(BranchingArgs::Label(
            "TestLabel".to_string(),
            "TestFunction".to_string(),
        ));
        let if_goto_label = Command::Branching(BranchingArgs::IfGoto(
            "TestLabel".to_string(),
            "TestFunction".to_string(),
        ));

        let expected_asm = vec![
            vec!["(TestFunction$TestLabel)"],
            vec!["@SP", "AM=M-1", "D=M", "@TestFunction$TestLabel", "D;JNE"],
        ];

        assert_commands_eq(vec![label_definition, if_goto_label], expected_asm);
    }

    #[test]
    fn define_function_signature_with_no_local_vars() {
        let cmd = Command::Function(FunctionArgs::Function("TestFunc".to_string(), 0));
        let expected_asm = vec![vec!["(TestFunc)"]];

        assert_commands_eq(vec![cmd], expected_asm);
    }

    #[test]
    fn define_function_signature_with_two_local_vars() {
        let cmd = Command::Function(FunctionArgs::Function("TestFunc".to_string(), 2));

        let expected_asm = vec![vec![
            "(TestFunc)",
            "@0",
            "D=A",
            "@LCL",
            "A=D+M",
            "M=0",
            "@SP",
            "M=M+1",
            "@1",
            "D=A",
            "@LCL",
            "A=D+M",
            "M=0",
            "@SP",
            "M=M+1",
        ]];

        assert_commands_eq(vec![cmd], expected_asm);
    }

    #[test]
    fn call_sum_function_with_no_local_vars() {
        let call_fn_command = Command::Function(FunctionArgs::Call("Sum".to_string(), 2));
        let define_fn_command = Command::Function(FunctionArgs::Function("Sum".to_string(), 0));

        let vm_commands: Vec<Command> = vec![
            Command::Operation(OperationArgs::Push(
                MemorySegment::Constant,
                1,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Push(
                MemorySegment::Constant,
                2,
                self::FILENAME.to_string(),
            )),
            call_fn_command,
            define_fn_command,
            Command::Operation(OperationArgs::Push(
                MemorySegment::Argument,
                0,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Push(
                MemorySegment::Argument,
                1,
                self::FILENAME.to_string(),
            )),
            Command::Operation(OperationArgs::Add),
        ];

        let expected_asm = vec![
            // --- begin push args --- //
            vec!["@1", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            vec!["@2", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            // --- end push args --- //

            // --- begin call function --- //
            vec![
                // Push return address
                "@Sum$ret.0",
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
                "@Sum",
                "0;JMP",
                // inject return address label to the asm instructions
                "(Sum$ret.0)",
            ],
            // --- end call function --- //

            // // --- begin function definition --- //
            vec!["(Sum)"],
            // no local vars init since there are no local vars
            //
            // push arg0 to stack
            vec!["@0", "D=A", "@ARG", "A=D+M", "D=M", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            // push arg1 to stack
            vec!["@1", "D=A", "@ARG", "A=D+M", "D=M", "@SP", "A=M", "M=D", "@SP", "M=M+1"],
            // add
            vec!["@SP", "AM=M-1", "D=M", "@SP", "AM=M-1", "M=D+M", "@SP", "M=M+1"],
            // --- end function definition --- //
        ];

        assert_commands_eq(vm_commands, expected_asm);
    }

    #[test]
    fn return_from_function() {
        assert_commands_eq(
            vec![Command::Function(FunctionArgs::Return)],
            vec![vec!["@GLOBAL_RETURN", "0;JMP"]],
        );
    }

    #[test]
    fn global_return_is_created_correctly() {
        let mut asm_generator =
            AsmGenerator { counter: 0, function_calls: HashMap::new(), instructions: vec![] };

        asm_generator.inject_global_return();

        assert_eq!(
            asm_generator.instructions,
            vec![
                "(GLOBAL_RETURN)",
                "@LCL",
                "D=M",
                "@R5",
                "M=D",
                "@5",
                "A=D-A",
                "D=M",
                "@R6",
                "M=D",
                "@SP",
                "AM=M-1",
                "D=M",
                "@ARG",
                "A=M",
                "M=D",
                "@ARG",
                "D=M+1",
                "@SP",
                "M=D",
                "@R5",
                "D=M",
                "A=D-1",
                "D=M",
                "@THAT",
                "M=D",
                "@R5",
                "D=M",
                "D=D-1",
                "A=D-1",
                "D=M",
                "@THIS",
                "M=D",
                "@R5",
                "D=M",
                "D=D-1",
                "D=D-1",
                "A=D-1",
                "D=M",
                "@ARG",
                "M=D",
                "@R5",
                "D=M",
                "D=D-1",
                "D=D-1",
                "D=D-1",
                "A=D-1",
                "D=M",
                "@LCL",
                "M=D",
                "@R6",
                "A=M",
                "0;JMP",
            ]
        );
    }

    #[test]
    fn recursive_function_calls_assigns_labels_properly() {
        let vm_commands: Vec<Command> = vec![
            Command::Function(FunctionArgs::Call("Test".to_string(), 0)),
            Command::Function(FunctionArgs::Function("Test".to_string(), 0)),
            Command::Function(FunctionArgs::Call("Test".to_string(), 0)),
            Command::Function(FunctionArgs::Return),
        ];

        let asm_commands = AsmGenerator::generate(vm_commands);

        assert!(asm_commands.iter().filter(|cmd| *cmd == "(Test$ret.0)").count() == 1);
        assert!(asm_commands.iter().filter(|cmd| *cmd == "@Test$ret.0").count() == 1);
        assert!(asm_commands.iter().filter(|cmd| *cmd == "(Test$ret.1)").count() == 1);
        assert!(asm_commands.iter().filter(|cmd| *cmd == "@Test$ret.1").count() == 1);
    }
}
