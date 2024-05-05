use crate::execute::Instruction::{
    Add, AddI, AndI, Copy, Jump, JumpIf, Negate, ShiftLeft, ShiftRight, StoreI, Subtract, SubtractI,
};
use crate::execute::{Instruction, Label};
use crate::parsing::Node;

pub const RESULT_REGISTER: u8 = 3;
pub const REMAINDER_REGISTER: u8 = 3;
pub const ITERATION_REGISTER: u8 = 0;
const MULTIPLIER_REGISTER: u8 = 1;
const MULTIPLICAND_REGISTER: u8 = 2;
const DIVISOR_REGISTER: u8 = 2;
const QUOTIENT_REGISTER: u8 = 1;
const TEST_REGISTER: u8 = 4;
pub struct Compiler {
    instructions: Vec<Instruction>,
    ast: Vec<Node>,
}

impl Compiler {
    pub fn new(ast: Vec<Node>) -> Self {
        Self {
            instructions: Vec::new(),
            ast,
        }
    }

    pub fn compile(mut self) -> Result<Vec<Instruction>, String> {
        for node in &self.ast {
            Self::compile_node(&mut self.instructions, node)?;
        }
        Ok(self.instructions)
    }

    fn compile_node(instructions: &mut Vec<Instruction>, node: &Node) -> Result<(), String> {
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
                    constant: 4,
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
                //lhs is always a number, but is actually the RHS! TODO
                Self::compile_node(instructions, lhs.as_ref())?;
                instructions.push(Copy {
                    src: RESULT_REGISTER,
                    dest: DIVISOR_REGISTER,
                });

                //rhs can be another type
                Self::compile_node(instructions, rhs.as_ref())?;

                instructions.push(ShiftLeft {
                    register: DIVISOR_REGISTER,
                    amount: 4,
                });

                // We want the remainder in the answer register anyway

                //Zero-out quotient register
                instructions.push(StoreI {
                    constant: 0,
                    register: QUOTIENT_REGISTER,
                });

                instructions.push(StoreI {
                    constant: 5,
                    register: ITERATION_REGISTER,
                });

                let jump_point = instructions.len() as u16;

                //step 1
                instructions.push(Subtract {
                    lhs: REMAINDER_REGISTER,
                    rhs: DIVISOR_REGISTER,
                    dest: REMAINDER_REGISTER,
                });

                //step 2
                instructions.push(Copy {
                    src: REMAINDER_REGISTER,
                    dest: TEST_REGISTER,
                });

                instructions.push(AndI {
                    register: TEST_REGISTER,
                    constant: i8::MIN,
                }); //0x10

                let label1 = (instructions.len() + 4) as u16;
                instructions.push(JumpIf {
                    instruction: label1,
                    test: TEST_REGISTER,
                });
                //branch for rem >= 0
                instructions.push(ShiftLeft {
                    register: QUOTIENT_REGISTER,
                    amount: 1,
                });
                instructions.push(AddI {
                    //Could use SubtractI with negative number but whatevs.
                    register: QUOTIENT_REGISTER,
                    constant: 1,
                });

                let label2 = (instructions.len() + 3) as u16;
                instructions.push(Jump {
                    instruction: label2,
                }); //Jump to step 3

                //branch for rem < 0
                instructions.push(Add {
                    lhs: REMAINDER_REGISTER,
                    rhs: DIVISOR_REGISTER,
                    dest: REMAINDER_REGISTER,
                });
                instructions.push(ShiftLeft {
                    register: QUOTIENT_REGISTER,
                    amount: 1,
                });

                //step 3
                instructions.push(ShiftRight {
                    register: DIVISOR_REGISTER,
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

                //Forget remainder
                instructions.push(Copy {
                    src: QUOTIENT_REGISTER,
                    dest: RESULT_REGISTER,
                });

                Ok(())
            }
            Node::Temp(_) => Err("Bad parsing!".to_string()),
        }
    }
}
