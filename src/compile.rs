use crate::execute::Instruction::{
    Add, AndI, Copy, JumpIf, Negate, ShiftLeft, ShiftRight, StoreI, Subtract, SubtractI,
};
use crate::execute::{Instruction, Label};
use crate::parsing::Node;

pub const RESULT_REGISTER: u8 = 3; //Or Quotient
pub const ITERATION_REGISTER: u8 = 0;
const MULTIPLIER_REGISTER: u8 = 1;
const MULTIPLICAND_REGISTER: u8 = 2;
const DIVISOR_REGISTER: u8 = 1;
const REMAINDER_REGISTER: u8 = 2;
const TEST_REGISTER: u8 = 4;
pub struct Compiler {
    instructions: Vec<Instruction>,
    ast: Vec<Node>,
    // free: [bool; 4],
}

impl Compiler {
    pub fn new(ast: Vec<Node>) -> Self {
        Self {
            instructions: Vec::new(),
            ast,
            // free: [true; 4],
        }
    }

    fn first_free(free: &[bool; 4]) -> Option<usize> {
        free.iter().position(|x| *x)
    }

    pub fn compile(mut self) -> Result<Vec<Instruction>, String> {
        for node in &self.ast {
            Self::compile_node(&mut self.instructions, node)?;
        }
        Ok(self.instructions)
    }

    fn compile_node(
        instructions: &mut Vec<Instruction>,
        // free: &mut [bool; 4],
        node: &Node,
    ) -> Result<(), String> {
        match node {
            Node::NumberN(x) => {
                instructions.push(StoreI {
                    constant: *x,
                    register: RESULT_REGISTER,
                });
                Ok(())
            }
            Node::MulN { rhs, lhs } => {
                //rhs can be another type
                Self::compile_node(instructions, rhs.as_ref())?;
                instructions.push(Copy {
                    src: RESULT_REGISTER,
                    dest: MULTIPLICAND_REGISTER,
                });

                //lhs is always a number
                Self::compile_node(instructions, lhs.as_ref())?;
                instructions.push(Copy {
                    src: RESULT_REGISTER,
                    dest: MULTIPLIER_REGISTER,
                });

                // Zero-out answer register
                instructions.push(AndI {
                    register: RESULT_REGISTER,
                    constant: 0,
                });

                instructions.push(StoreI {
                    constant: 4, //TODO maybe 5
                    register: ITERATION_REGISTER,
                });

                let jump_point = instructions.len() as u16;

                //Step 1
                instructions.push(Copy {
                    src: MULTIPLIER_REGISTER,
                    dest: TEST_REGISTER,
                });
                instructions.push(Negate {
                    register: TEST_REGISTER,
                });
                instructions.push(AndI {
                    register: TEST_REGISTER,
                    constant: 1,
                });
                let forward = instructions.len() + 2;
                instructions.push(JumpIf {
                    instruction: forward as Label,
                    test: TEST_REGISTER,
                });
                instructions.push(Add {
                    lhs: MULTIPLICAND_REGISTER,
                    rhs: RESULT_REGISTER,
                    dest: RESULT_REGISTER,
                });

                //step 2
                instructions.push(ShiftLeft {
                    register: MULTIPLICAND_REGISTER,
                    amount: 1,
                });

                //step 3
                instructions.push(ShiftRight {
                    register: MULTIPLIER_REGISTER,
                    amount: 1,
                });

                //Iterate
                instructions.push(SubtractI {
                    register: ITERATION_REGISTER,
                    constant: 1,
                });

                instructions.push(JumpIf {
                    instruction: jump_point,
                    test: ITERATION_REGISTER,
                });

                Ok(())
            }
            Node::DivN { rhs, lhs } => {
                todo!()
            }
            Node::Temp(_) => Err("Bad parsing!".to_string()),
        }
    }
}
