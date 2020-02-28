use bct::{Tryte, Word};
use crate::memory::{Memory, MemError};
use crate::instructions::{Instr, ISA};
use crate::decoder::{decode_instruction, DecodeError, DecodeResult};

pub struct Cpu<Memory> {
  isa: ISA,
  pc: i32,
  mem: Memory,
  estack: Vec<Tryte>,
  cstack: Vec<i32>,
  cy: usize
}

impl<M: Memory> Cpu<M> {
  pub fn debug(&self) {
    let mut result = Vec::new();
    for t in &self.estack {
      match t.as_nonary_string() {
        Some(s) => { result.push(s) },
        None => {},
      }
    }
    let pc = Word::from(self.pc);
    println!("{}{}: {}", pc.hi_tryte().as_nonary_string().unwrap(), pc.lo_tryte().as_nonary_string().unwrap(), result.join(" "));
  }

  fn _load_tryte_pc(&mut self) -> Result<Tryte, MemError> {
    let pc = self.pc;
    let val = self.mem.load_tryte(pc);
    self.pc += 1;
    val
  }

  fn drop_t(&mut self) {
    self.estack.pop();
  }

  fn dup_t(&mut self) {
    let val = self.estack.last().unwrap().clone();
    self.estack.push(val);
  }
  
  fn swap_t(&mut self) {
    let (val1, val2) = self.pop2();
    self.estack.push(val1);
    self.estack.push(val2);
  }

  fn rot_t(&mut self) {
    let (a, b, c) = self.pop3();
    self.estack.push(b);
    self.estack.push(a);
    self.estack.push(c);
  }

  fn _over_t(&mut self) {
    let (a, b) = self.pop2();
    self.estack.push(b.clone());
    self.estack.push(a);
    self.estack.push(b);
  }

  fn add_t(&mut self) {
    let (val1, val2) = self.pop2();
    let (result, _carry) = Tryte::add(&val1, &val2);
    println!("ADD - {} + {} = {}", val1.to_integer(), val2.to_integer(), result.to_integer());
    self.estack.push(result);
  }

  fn neg_t(&mut self) {
    let val = self.pop();
    println!("NEG - value before: {}", val.to_integer());
    let result = Tryte::from(-val.to_integer());
    println!("NEG - value after: {:?} ({})", result, result.to_integer());
    self.estack.push(result);
  }
  
  fn _inc_t(&mut self) {
    let val = self.pop();
    let (result, _carry) = Tryte::add(&val, &Tryte::from(1));
    self.estack.push(result);
  }

  fn _sub_t(&mut self) {
    let (val1, val2) = self.pop2();
    let (result, _carry) = Tryte::sub(&val2, &val1);
    self.estack.push(result);
  }

  fn _dec_t(&mut self) {
    let val = self.pop();
    let (result, _carry) = Tryte::sub(&val, &Tryte::from(1));
    self.estack.push(result);
  }

  // Pops one tryte and uses it as offset
  // fn load_near_t(&mut self) {
  //   let offset = self.estack.pop().unwrap();
  //   let val = self.mem.load_tryte(self.pc + offset.0 as i32).unwrap();
  //   self.estack.push(val);
  // }

  /// Pops two trytes and uses them as address
  fn load_abs_t(&mut self) {
    let (lo, hi) = self.pop2();
    let addr = Word::from_trytes(hi, lo);
    let val = self.mem.load_tryte(addr.into()).unwrap();
    self.estack.push(val);
  }

  /// Pops two trytes and uses them as address
  fn store_abs_t(&mut self) {
    let (lo, hi, val) = self.pop3();
    let addr = Word::from_trytes(hi, lo);
    self.mem.store_tryte(addr.into(), val).unwrap();
  }

  fn jmp_abs(&mut self) {
    let (lo, hi) = self.pop2();
    let addr = Word::from_trytes(hi, lo);
    self.jump(addr.into());
  }

  // fn jmp_near(&mut self) {
  //   let offset = self.pop();
  //   let addr = self.pc + offset.to_integer() as i32;
  //   self.jump(addr.into());
  // }

  fn jsr_abs(&mut self) {
    let (lo, hi) = self.pop2();
    let addr = Word::from_trytes(hi, lo);
    self.cstack.push(self.pc);
    self.jump(addr.into());
  }

  // fn jsr_near(&mut self) {
  //   let offset = self.pop();
  //   let addr = self.pc + offset.to_integer() as i32;
  //   self.cstack.push(self.pc);
  //   self.jump(addr.into());
  // }

  fn rts(&mut self) {
    let addr = self.cstack.pop().unwrap();
    self.jump(addr);
  }

  fn bz_t(&mut self) {
    let (offset, val) = self.pop2();
    if val.to_integer() == 0 {
      self.jump_relative(&offset);
    }
  }

  fn _bnz_t(&mut self) {
    let (offset, val) = self.pop2();
    if val.to_integer() != 0 {
      self.jump_relative(&offset);
    }
  }

  fn bpl_t(&mut self) {
    let (offset, val) = self.pop2();
    if val.to_integer() > 0 {
      self.jump_relative(&offset);
    }
  }

  fn _bnpl_t(&mut self) {
    let (offset, val) = self.pop2();
    if val.to_integer() <= 0 {
      self.jump_relative(&offset);
    }
  }

  fn bmi_t(&mut self) {
    let (offset, val) = self.pop2();
    if val.to_integer() < 0 {
      self.jump_relative(&offset);
    }
  }

  fn _bnmi_t(&mut self) {
    let (offset, val) = self.pop2();
    if val.to_integer() >= 0 {
      self.jump_relative(&offset);
    }
  }

  //
  // Helpers
  //
  fn push_tryte(&mut self, tryte: Tryte) {
    self.estack.push(tryte);
  }

  fn push_word(&mut self, word: Word) {
    self.estack.push(word.hi_tryte().clone());
    self.estack.push(word.lo_tryte().clone());
  }

  fn jump(&mut self, addr: i32) {
    self.pc = addr;
  }

  fn jump_relative(&mut self, offset: &Tryte) {
    self.pc += offset.to_integer() as i32;
  }

  fn pop(&mut self) -> Tryte {
    let tryte = self.estack.pop().unwrap();
    tryte
  }

  fn pop2(&mut self) -> (Tryte, Tryte) {
    let t1 = self.pop();
    let t2 = self.pop();
    (t1, t2)
  }

  fn pop3(&mut self) -> (Tryte, Tryte, Tryte) {
    let t1 = self.pop();
    let t2 = self.pop();
    let t3 = self.pop();
    (t1, t2, t3)
  }

  fn execute_op(&mut self, op: Instr) -> Result<(), String> {
    match op {
      // Instr::HALT => Err(format!("\n\nExited cleanly, performed {} operations", self.cy)),
      Instr::IRQ => Err(format!("\n\nExited cleanly, performed {} operations", self.cy)),
      Instr::NOP => Ok(()), // do nothing
      Instr::LIT(tryte) => Ok(self.push_tryte(tryte)),
      Instr::WORD(word) => Ok(self.push_word(word)),
      // Tryte stack management
      Instr::DROP => Ok(self.drop_t()),
      Instr::DUP => Ok(self.dup_t()),
      Instr::SWAP => Ok(self.swap_t()),
      Instr::ROT => Ok(self.rot_t()),
      // Arithmetics
      Instr::ADD => Ok(self.add_t()),
      Instr::NEG => Ok(self.neg_t()),
      // Memory management
      Instr::LOAD => Ok(self.load_abs_t()),
      Instr::STOR => Ok(self.store_abs_t()),
      // Flow control
      Instr::JMP => Ok(self.jmp_abs()),
      Instr::CALL => Ok(self.jsr_abs()),
      Instr::RET => Ok(self.rts()),
      // Conditional branches
      Instr::BZ => Ok(self.bz_t()),
      Instr::BMI => Ok(self.bmi_t()),
      Instr::BPL => Ok(self.bpl_t()),
      // Unimplemented
      _ => { panic!("Unimplemented instruction: {:?}", op) },
    }
  }

  pub fn step(&mut self) -> Result<usize, String> {
    let (decode_res, pc) = decode_instruction(&self.isa, &self.mem, self.pc);
    let (first, second) = match decode_res {
      Ok(DecodeResult(fst, snd)) => (fst, snd),
      Err(err) => match err {
        DecodeError::IllegalInstruction { pos, op } => { panic!("Illegal instruction {} at {}", op, pos) },
        DecodeError::MemoryError(_) => { panic!("Illegal memory access") },
        _ => { panic!("Unknown error") },
      },
    };

    self.pc = pc;

    self.cy += 1;
    if let Err(err) = self.execute_op(first) {
      return Err(err);
    }

    self.cy += 1;
    if let Err(err) = self.execute_op(second) {
      return Err(err);
    }
    
    Ok(self.cy)
  }

  pub fn set_cycle_count(&mut self, cycles: usize) {
    self.cy = cycles;
  }

  pub fn new(isa: ISA, mem: M, pc: i32) -> Cpu<M> {
    Cpu {
      isa,
      pc,
      mem,
      estack: Vec::new(),
      cstack: Vec::new(),
      cy: 0,
    }
  }
}

