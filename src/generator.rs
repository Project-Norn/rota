use crate::instruction::{Instruction, Mnemonic, Operand, RegSize, Register};
use std::collections::HashMap;

struct Generator {
    output: Vec<u8>,
    labels: HashMap<String, u32>,
    unresolved_jumps: Vec<UnresolvedJump>,
}

type UnresolvedJump = (String, u32);

pub fn generate(insts: Vec<Instruction>) -> Result<Vec<u8>, String> {
    let mut generator = Generator::new();
    generator.generate(insts)
}

impl Generator {
    fn new() -> Self {
        Self {
            output: Vec::new(),
            labels: HashMap::new(),
            unresolved_jumps: Vec::new(),
        }
    }

    fn generate(&mut self, insts: Vec<Instruction>) -> Result<Vec<u8>, String> {
        for inst in insts {
            self.gen_inst(inst)?;
        }
        self.resolve_jump()?;
        Ok(self.output.clone())
    }

    fn gen_inst(&mut self, inst: Instruction) -> Result<(), String> {
        match inst {
            Instruction::PseudoOp { .. } => {}
            Instruction::Label { name } => {
                let addr = self.output.len();
                self.labels.insert(name, addr as u32);
            }
            Instruction::NullaryOp(op) => self.gen_nullary_op(op)?,
            Instruction::UnaryOp(op, operand) => self.gen_unary_op(op, operand)?,
            Instruction::BinaryOp(op, operand1, operand2) => {
                self.gen_biary_op(op, operand1, operand2)?
            }
        }
        Ok(())
    }

    fn gen_nullary_op(&mut self, m: Mnemonic) -> Result<(), String> {
        match m {
            Mnemonic::Ret => self.gen(0xC3),
            x => return Err(format!("unexpected mnemonic: {:?}", x)),
        }
        Ok(())
    }

    fn gen_unary_op(&mut self, m: Mnemonic, operand: Operand) -> Result<(), String> {
        match m {
            Mnemonic::Push => match operand {
                Operand::Immidiate { value } => self.gen_i(0x6A, value),
                Operand::Register { reg } => {
                    if reg.size() != RegSize::QWord {
                        return Err(format!("unexpected operand: {:?}", reg));
                    }
                    self.gen_o(0x50, reg)
                }
                x => return Err(format!("unexpected operand: {:?}", x)),
            },
            Mnemonic::Pop => match operand {
                Operand::Register { reg } => {
                    if reg.size() != RegSize::QWord {
                        return Err(format!("unexpected operand: {:?}", reg));
                    }
                    self.gen_o(0x58, reg)
                }
                x => return Err(format!("unexpected operand: {:?}", x)),
            },
            Mnemonic::IDiv => match operand {
                Operand::Register { reg } => self.gen_m(&[0xF7], 7, reg),
                x => return Err(format!("unexpected operand: {:?}", x)),
            },
            Mnemonic::Jmp => self.gen_jump(&[0xE9], operand)?,
            Mnemonic::Sete => self.gen_set(0x94, operand)?,
            Mnemonic::Je => self.gen_jump(&[0x0F, 0x84], operand)?,
            Mnemonic::Setne => self.gen_set(0x95, operand)?,
            Mnemonic::Setl => self.gen_set(0x9C, operand)?,
            Mnemonic::Setle => self.gen_set(0x9E, operand)?,
            Mnemonic::Setg => self.gen_set(0x9F, operand)?,
            Mnemonic::Setge => self.gen_set(0x9D, operand)?,
            Mnemonic::Call => self.gen_jump(&[0xE8], operand)?,
            x => return Err(format!("unexpected mnemonic: {:?}", x)),
        }
        Ok(())
    }

    fn gen_biary_op(
        &mut self,
        m: Mnemonic,
        operand1: Operand,
        operand2: Operand,
    ) -> Result<(), String> {
        if !is_same_reg_size(&operand1, &operand2) {
            return Err(format!(
                "operand type mismatch: {:?} and {:?}",
                operand1, operand2
            ));
        }

        let reg1 = expect_register(operand1)?;
        match m {
            Mnemonic::Add => match operand2 {
                Operand::Register { reg: reg2 } => self.gen_mr(0x01, reg1, reg2),
                Operand::Immidiate { value } => self.gen_mi(0x83, 0, reg1, value),
                x => return Err(format!("unexpected operand: {:?}", x)),
            },
            Mnemonic::Sub => match operand2 {
                Operand::Register { reg: reg2 } => self.gen_mr(0x29, reg1, reg2),
                Operand::Immidiate { value } => self.gen_mi(0x83, 5, reg1, value),
                x => return Err(format!("unexpected operand: {:?}", x)),
            },
            Mnemonic::IMul => {
                let reg2 = expect_register(operand2)?;
                self.gen_rm(&[0x0F, 0xAF], reg1, reg2);
            }
            Mnemonic::Xor => match operand2 {
                Operand::Register { reg: reg2 } => self.gen_mr(0x31, reg1, reg2),
                Operand::Immidiate { value } => self.gen_mi(0x83, 6, reg1, value),
                x => return Err(format!("unexpected operand: {:?}", x)),
            },
            Mnemonic::Mov => match operand2 {
                Operand::Register { reg: reg2 } => self.gen_rm(&[0x8B], reg1, reg2),
                Operand::Immidiate { value } => self.gen_mi32(0xC7, 0, reg1, value),
                x => return Err(format!("unexpected operand: {:?}", x)),
            },
            Mnemonic::And => match operand2 {
                Operand::Register { reg: reg2 } => self.gen_rm(&[0x23], reg1, reg2),
                Operand::Immidiate { value } => self.gen_mi(0x83, 4, reg1, value),
                x => return Err(format!("unexpected operand: {:?}", x)),
            },
            Mnemonic::Or => match operand2 {
                Operand::Register { reg: reg2 } => self.gen_mr(0x09, reg1, reg2),
                Operand::Immidiate { value } => self.gen_mi(0x83, 1, reg1, value),
                x => return Err(format!("unexpected operand: {:?}", x)),
            },
            Mnemonic::Cmp => match operand2 {
                Operand::Register { reg: reg2 } => self.gen_mr(0x39, reg1, reg2),
                Operand::Immidiate { value } => self.gen_mi(0x83, 7, reg1, value),
                x => return Err(format!("unexpected operand: {:?}", x)),
            },
            x => return Err(format!("unexpected mnemonic: {:?}", x)),
        }
        Ok(())
    }

    // [opcodes]+[opr](4)
    fn gen_jump(&mut self, opcodes: &[u8], operand: Operand) -> Result<(), String> {
        match operand {
            Operand::Label { name } => {
                // jmp命令のオペランド部分の開始アドレス
                let jump_opr = (self.output.len() + opcodes.len()) as u32;
                // labelのアドレス(labelの次の命令の開始アドレス)
                let label_addr = self.lookup_label(name, jump_opr);
                // jmp命令の次の命令の開始アドレス
                let after_jump_addr = jump_opr + 4;
                let diff = label_addr.wrapping_sub(after_jump_addr);
                self.gen_d32(opcodes, diff);
            }
            x => return Err(format!("unexpected operand: {:?}", x)),
        }
        Ok(())
    }

    fn gen_set(&mut self, opcode: u8, operand: Operand) -> Result<(), String> {
        let reg1 = expect_register(operand)?;
        if reg1.size() != RegSize::Byte {
            return Err(format!("expected r8"));
        }
        self.gen_m(&[0x0F, opcode], 0, reg1);
        Ok(())
    }

    fn gen_i(&mut self, opcode: u8, imm: u32) {
        self.gen(opcode);
        self.gen(imm as u8);
    }

    fn gen_o(&mut self, opcode: u8, reg: Register) {
        if reg.only_in_64bit() {
            self.gen_rex(false, false, false, true);
        }
        self.gen(opcode + reg.number());
    }

    fn gen_m(&mut self, opcodes: &[u8], reg: u8, r: Register) {
        if r.size() == RegSize::QWord || r.only_in_64bit() {
            self.gen_rex(r.size() == RegSize::QWord, false, false, r.only_in_64bit());
        }
        self.gen_bytes(opcodes);
        self.gen(calc_modrm(0b11, reg, r.number()));
    }

    fn gen_d(&mut self, opcode: u8, offset: u8) {
        self.gen(opcode);
        self.gen(offset);
    }

    // TODO
    fn gen_d32(&mut self, opcodes: &[u8], offset: u32) {
        self.gen_bytes(opcodes);
        self.gen32(offset);
    }

    fn gen_mr(&mut self, opcode: u8, opr1: Register, opr2: Register) {
        if opr1.size() == RegSize::QWord {
            self.gen_rex(true, opr2.only_in_64bit(), false, opr1.only_in_64bit());
        }
        self.gen(opcode);
        self.gen(calc_modrm(0b11, opr2.number(), opr1.number()));
    }

    fn gen_mi(&mut self, opcode: u8, reg: u8, opr1: Register, opr2: u32) {
        if opr1.size() == RegSize::QWord {
            self.gen_rex(true, false, false, opr1.only_in_64bit());
        }
        self.gen(opcode);
        self.gen(calc_modrm(0b11, reg, opr1.number()));
        self.gen(opr2 as u8);
    }

    // TODO
    fn gen_mi32(&mut self, opcode: u8, reg: u8, opr1: Register, opr2: u32) {
        if opr1.size() == RegSize::QWord {
            self.gen_rex(true, false, false, opr1.only_in_64bit());
        }
        self.gen(opcode);
        self.gen(calc_modrm(0b11, reg, opr1.number()));
        self.gen32(opr2);
    }

    fn gen_rm(&mut self, opcodes: &[u8], opr1: Register, opr2: Register) {
        if opr1.size() == RegSize::QWord {
            self.gen_rex(true, opr1.only_in_64bit(), false, opr2.only_in_64bit());
        }
        self.gen_bytes(opcodes);
        self.gen(calc_modrm(0b11, opr1.number(), opr2.number()));
    }

    fn gen_oi(&mut self, opcode: u8, opr1: Register, opr2: u32) {
        if opr1.size() == RegSize::QWord {
            self.gen_rex(true, false, false, opr1.only_in_64bit());
        }
        self.gen(opcode + opr1.number());
        self.gen32(opr2);
    }

    fn gen_rex(&mut self, w: bool, r: bool, x: bool, b: bool) {
        self.gen(0b01000000 | (w as u8) << 3 | (r as u8) << 2 | (x as u8) << 1 | (b as u8))
    }

    fn lookup_label(&mut self, name: String, code_addr: u32) -> u32 {
        match self.labels.get(&name) {
            Some(addr) => *addr,
            None => {
                self.unresolved_jumps.push((name, code_addr));
                0
            }
        }
    }

    fn resolve_jump(&mut self) -> Result<(), String> {
        for (name, jump_opr) in &self.unresolved_jumps {
            match self.labels.get(name) {
                Some(label_addr) => {
                    let after_jump_addr = jump_opr + 4;
                    let diff = (*label_addr).wrapping_sub(after_jump_addr);
                    for (i, byte) in diff.to_le_bytes().iter().enumerate() {
                        self.output[*jump_opr as usize + i] = *byte;
                    }
                }
                None => return Err(format!("undefined label: {}", name)),
            }
        }
        Ok(())
    }

    fn gen32(&mut self, bytes: u32) {
        for i in 0..4 {
            let byte = (bytes >> (8 * i)) as u8;
            self.gen(byte);
        }
    }

    fn gen_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.gen(*byte)
        }
    }

    fn gen(&mut self, byte: u8) {
        self.output.push(byte);
    }
}

fn calc_modrm(modval: u8, reg: u8, rm: u8) -> u8 {
    modval << 6 | reg << 3 | rm
}

fn expect_register(operand: Operand) -> Result<Register, String> {
    match operand {
        Operand::Register { reg } => Ok(reg),
        x => Err(format!("unexpected operand: {:?}", x)),
    }
}

fn is_same_reg_size(op1: &Operand, op2: &Operand) -> bool {
    match (op1, op2) {
        (Operand::Register { reg: reg1 }, Operand::Register { reg: reg2 }) => {
            reg1.size() == reg2.size()
        }
        _ => true,
    }
}
