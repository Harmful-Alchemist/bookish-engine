use std::fmt::{Display, Formatter};

pub type Label = u16;

type Register = u8;
pub type RegisterContent = i16;

//Execution
#[derive(Clone, Debug)]
// #[repr(C)] if we want to bit match :P : 24bit length instructions (max '3 * Register' or 'register + Label') + something to know the variant
pub enum Instruction {
    Negate {
        register: Register,
    },
    AndI {
        register: Register,
        constant: RegisterContent,
    },
    Jump {
        instruction: Label,
    },
    JumpIf {
        instruction: Label,
        test: Register,
    }, //Jump if zero in test register
    StoreI {
        constant: RegisterContent,
        register: Register,
    },
    Copy {
        src: Register,
        dest: Register,
    },
    Subtract {
        lhs: Register,
        rhs: Register,
        dest: Register,
    },
    SubtractI {
        register: Register,
        constant: RegisterContent,
    },
    Add {
        lhs: Register,
        rhs: Register,
        dest: Register,
    },
    AddI {
        register: Register,
        constant: RegisterContent,
    },
    ShiftLeft {
        register: Register,
        amount: u8,
    },
    ShiftRight {
        register: Register,
        amount: u8,
    },
}

impl Instruction {
    fn as_str(&self) -> &'static str {
        match self {
            Instruction::Negate { .. } => "Negate",
            Instruction::AndI { .. } => "AndI",
            Instruction::Jump { .. } => "Jump",
            Instruction::JumpIf { .. } => "JumpIf",
            Instruction::StoreI { .. } => "StoreI",
            Instruction::Copy { .. } => "Copy",
            Instruction::Subtract { .. } => "Subtract",
            Instruction::SubtractI { .. } => "SubtractI",
            Instruction::Add { .. } => "Add",
            Instruction::AddI { .. } => "Add",
            Instruction::ShiftLeft { .. } => "ShiftLeft",
            Instruction::ShiftRight { .. } => "ShiftRight",
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let dsp = self.as_str();
        write!(f, "{}", dsp)
    }
}

pub struct Machine {
    registers: [RegisterContent; 5],
    pc: u16,
}

impl Machine {
    pub fn new() -> Self {
        Self {
            registers: [0; 5],
            pc: 0,
        }
    }

    pub fn answer_by_convention(&self) -> RegisterContent {
        self.registers[3]
    }

    pub fn run(&mut self, program: Vec<Instruction>) {
        self.pc = 0;
        loop {
            let instruction = &program[self.pc as usize];

            match instruction {
                Instruction::Negate { register } => {
                    self.registers[*register as usize] = !self.registers[*register as usize]
                    //^ *constant;
                }
                Instruction::AndI { register, constant } => {
                    self.registers[*register as usize] &= constant;
                }
                Instruction::Jump { instruction } => {
                    self.pc = *instruction;
                    continue;
                }
                Instruction::JumpIf { instruction, test } => {
                    if self.registers[*test as usize] != 0 {
                        self.pc = *instruction;
                        continue;
                    }
                }
                Instruction::StoreI { constant, register } => {
                    self.registers[*register as usize] = *constant;
                }
                Instruction::Copy { src, dest } => {
                    self.registers[*dest as usize] = self.registers[*src as usize];
                }
                Instruction::Subtract { lhs, rhs, dest } => {
                    self.registers[*dest as usize] =
                        self.registers[*lhs as usize] - self.registers[*rhs as usize];
                }
                Instruction::Add { lhs, rhs, dest } => {
                    self.registers[*dest as usize] =
                        self.registers[*lhs as usize] + self.registers[*rhs as usize];
                }
                Instruction::AddI { register, constant } => {
                    self.registers[*register as usize] += *constant;
                }
                Instruction::ShiftLeft { register, amount } => {
                    self.registers[*register as usize] <<= amount;
                }
                Instruction::ShiftRight { register, amount } => {
                    self.registers[*register as usize] =
                        (self.registers[*register as usize] as u16 >> amount) as RegisterContent;
                }
                Instruction::SubtractI { register, constant } => {
                    self.registers[*register as usize] -= *constant;
                }
            }

            println!(
                "|{:^3?}|{:^12}| {:08b} | {:08b} | {:08b} | (test: {:08b})",
                self.registers[0],
                &program[self.pc as usize].as_str(),
                self.registers[1],
                self.registers[2],
                self.registers[3],
                self.registers[4],
            );
            self.pc += 1;
            if self.pc == program.len() as u16 {
                break;
            }
        }
    }
}
