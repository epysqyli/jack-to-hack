pub trait Hackable {
    fn to_hack(self: Self) -> Result<Instruction, String>;
}

#[derive(Debug, PartialEq)]
pub struct Instruction(pub String);

impl Instruction {
	pub	fn from(s: String) -> Result<Self, String> {
		let valid_chars = vec!['0', '1'];

		match s.len() {
			16 => {
				if s.chars().any(|c| !valid_chars.contains(&c)) {
					return Err("Only '0' and '1' are valid".to_string());
				}

				Ok(Self(s))
			},
			_ => Err("Hack instructions need to be of length 16".to_string()),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn instruction_is_created() {
		let a_inst = format!("0{:015b}", 16).to_string();
		assert!(Instruction::from(a_inst).is_ok());
	}

	#[test]
	fn instruction_creation_fails_due_to_invalid_length() {
		let a_inst = format!("0{:08b}", 16).to_string();
		assert!(Instruction::from(a_inst).is_err());
	}

	#[test]
	fn instruction_creation_fails_due_to_invalid_chars() {
		let asm_inst = "00FA".to_string();
		assert!(Instruction::from(asm_inst).is_err());
	}
}
