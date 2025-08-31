use crate::instruction::hack::{Hackable, Instruction as HackInstruction};

#[derive(PartialEq, Debug)]
pub enum Instruction {
    /// Starts with @
    A(String),
    /// All other instructions
    C(String),
}

impl Instruction {
    pub fn from(string: String) -> Option<Self> {
        if string.is_empty() {
            return None;
        }

        if let Some(instruction_type) = string.chars().nth(0) {
            return match instruction_type {
                '@' => Some(Instruction::A(string)),
                _ => Some(Instruction::C(string)),
            };
        }

        None
    }
}

impl Hackable for Instruction {
    fn to_hack(self: Self) -> Result<HackInstruction, String> {
        match self {
            Instruction::A(value) => match value.split_at(1).1.parse::<u16>() {
                Ok(val) => HackInstruction::from(format!("0{:015b}", val)),
                Err(e) => Err(e.to_string()),
            },
            Instruction::C(value) => {
                let empty = String::new();

                let (ddd, ccc, jjj) = if value.contains('=') && value.contains(';') {
                    let res1: Vec<&str> = value.split('=').collect();
                    let dest = res1.first().unwrap();
                    let res2: Vec<&str> = res1.last().unwrap().split(';').collect();
                    let comp = res2.first().unwrap();
                    let jump = res2.last().unwrap();

                    (*dest, *comp, *jump)
                } else if value.contains('=') && !value.contains(';') {
                    let res: Vec<&str> = value.split('=').collect();
                    let dest = *res.first().unwrap();
                    let comp = *res.last().unwrap();

                    (dest, comp, empty.as_str())
                } else if !value.contains('=') && value.contains(';') {
                    let res: Vec<&str> = value.split(';').collect();
                    let comp = *res.first().unwrap();
                    let jump = *res.last().unwrap();

                    (empty.as_str(), comp, jump)
                } else {
                    (empty.as_str(), empty.as_str(), empty.as_str())
                };

                let comp_bits = match ccc {
                    "0" => Some(['0', '1', '0', '1', '0', '1', '0']),
                    "1" => Some(['0', '1', '1', '1', '1', '1', '1']),
                    "-1" => Some(['0', '1', '1', '1', '0', '1', '0']),
                    "D" => Some(['0', '0', '0', '1', '1', '0', '0']),
                    "A" => Some(['0', '1', '1', '0', '0', '0', '0']),
                    "!D" => Some(['0', '0', '0', '1', '1', '0', '1']),
                    "!A" => Some(['0', '1', '1', '0', '0', '0', '1']),
                    "-D" => Some(['0', '0', '0', '1', '1', '1', '1']),
                    "-A" => Some(['0', '1', '1', '0', '0', '1', '1']),
                    "D+1" => Some(['0', '0', '1', '1', '1', '1', '1']),
                    "A+1" => Some(['0', '1', '1', '0', '1', '1', '1']),
                    "D-1" => Some(['0', '0', '0', '1', '1', '1', '0']),
                    "A-1" => Some(['0', '1', '1', '0', '0', '1', '0']),
                    "D+A" => Some(['0', '0', '0', '0', '0', '1', '0']),
                    "D-A" => Some(['0', '0', '0', '0', '0', '1', '1']),
                    "A-D" => Some(['0', '0', '0', '0', '1', '1', '1']),
                    "D&A" => Some(['0', '0', '0', '0', '0', '0', '0']),
                    "D|A" => Some(['0', '0', '1', '0', '1', '0', '1']),
                    "M" => Some(['1', '1', '1', '0', '0', '0', '0']),
                    "!M" => Some(['1', '1', '1', '0', '0', '0', '1']),
                    "-M" => Some(['1', '1', '1', '0', '0', '1', '1']),
                    "M+1" => Some(['1', '1', '1', '0', '1', '1', '1']),
                    "M-1" => Some(['1', '1', '1', '0', '0', '1', '0']),
                    "D+M" => Some(['1', '0', '0', '0', '0', '1', '0']),
                    "D-M" => Some(['1', '0', '0', '0', '0', '1', '1']),
                    "M-D" => Some(['1', '0', '0', '0', '1', '1', '1']),
                    "D&M" => Some(['1', '0', '0', '0', '0', '0', '0']),
                    "D|M" => Some(['1', '0', '1', '0', '1', '0', '1']),
                    _ => None,
                };

                if comp_bits.is_none() {
                    return Err("Invalid computation mnemonic".to_string());
                }

                let dest_bits = match ddd {
                    "" => Some(['0', '0', '0']),
                    "M" => Some(['0', '0', '1']),
                    "D" => Some(['0', '1', '0']),
                    "MD" => Some(['0', '1', '1']),
                    "A" => Some(['1', '0', '0']),
                    "AM" => Some(['1', '0', '1']),
                    "AD" => Some(['1', '1', '0']),
                    "AMD" => Some(['1', '1', '1']),
                    _ => None,
                };

                if dest_bits.is_none() {
                    return Err("Invalid destination mnemonic".to_string());
                }

                let jump_bits = match jjj {
                    "" => Some(['0', '0', '0']),
                    "JGT" => Some(['0', '0', '1']),
                    "JEQ" => Some(['0', '1', '0']),
                    "JGE" => Some(['0', '1', '1']),
                    "JLT" => Some(['1', '0', '0']),
                    "JNE" => Some(['1', '0', '1']),
                    "JLE" => Some(['1', '1', '0']),
                    "JMP" => Some(['1', '1', '1']),
                    _ => None,
                };

                if jump_bits.is_none() {
                    return Err("Invalid jump mnemonic".to_string());
                }

                let mut out: [char; 16] = ['0'; 16];
                out[0] = '1';
                out[1] = '1';
                out[2] = '1';

                let comp_bits = comp_bits.unwrap();
                out[3] = comp_bits[0];
                out[4] = comp_bits[1];
                out[5] = comp_bits[2];
                out[6] = comp_bits[3];
                out[7] = comp_bits[4];
                out[8] = comp_bits[5];
                out[9] = comp_bits[6];

                let dest_bits = dest_bits.unwrap();
                out[10] = dest_bits[0];
                out[11] = dest_bits[1];
                out[12] = dest_bits[2];

                let jump_bits = jump_bits.unwrap();
                out[13] = jump_bits[0];
                out[14] = jump_bits[1];
                out[15] = jump_bits[2];

                Ok(HackInstruction::from(out.iter().collect())?)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_instruction_type_a_is_decoded_properly() {
        let inst = "@16";

        assert_eq!(
            Instruction::A(inst.to_string()),
            Instruction::from(inst.to_string()).unwrap()
        );
    }

    #[test]
    fn test_instruction_type_c_is_decoded_properly() {
        let inst = "D;JEQ";

        assert_eq!(
            Instruction::C(inst.to_string()),
            Instruction::from(inst.to_string()).unwrap()
        );
    }

    #[test]
    fn test_type_a_from_asm_to_hack() {
        let mut asm_to_hack: HashMap<&str, &str> = HashMap::with_capacity(3);
        asm_to_hack.insert("@16", "0000000000010000");
        asm_to_hack.insert("@32", "0000000000100000");
        asm_to_hack.insert("@64", "0000000001000000");

        asm_to_hack.iter().for_each(|(asm, hack)| {
            assert_eq!(*hack, Instruction::A(asm.to_string()).to_hack().unwrap().0)
        });
    }

    #[test]
    fn test_type_c_from_asm_to_hack() {
        let mut asm_to_hack: HashMap<&str, &str> = HashMap::with_capacity(3);
        asm_to_hack.insert("MD=M+1", "1111110111011000");
        asm_to_hack.insert("D;JEQ", "1110001100000010");
        asm_to_hack.insert("D=M;JEQ", "1111110000010010");

        asm_to_hack.iter().for_each(|(asm, hack)| {
            assert_eq!(*hack, Instruction::C(asm.to_string()).to_hack().unwrap().0)
        });
    }

    #[test]
    fn test_type_c_fails_when_invalid_asm_instruction() {
        assert_eq!(
            Err("Invalid computation mnemonic".to_string()),
            Instruction::C("D=M+D".to_string()).to_hack()
        );

        assert_eq!(
            Err("Invalid destination mnemonic".to_string()),
            Instruction::C("DA=1".to_string()).to_hack()
        );

        assert_eq!(
            Err("Invalid jump mnemonic".to_string()),
            Instruction::C("D;JGG".to_string()).to_hack()
        );
    }
}
