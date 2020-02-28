use bct::{Tryte, Word};

#[derive(Debug)]
pub enum ISA {
  T6010, // 6-trit instructions, 10-trit addressable
}

#[derive(Debug)]
pub enum Instr {
  NOP,

  // Push
  LIT(Tryte),
  WORD(Word),

  // Stack operations
  DROP,
  DUP,
  SWAP,
  ROT,

  // Arithmetics
  ADD,
  NEG,

  // Logical
  MAX,
  INC,
  IST,
  ISU,
  SHL,
  SHR,

  // Memory management
  LOAD,
  STOR,

  // Flow control
  JMP,
  CALL,
  RET,
  IRQ,

  // Conditional branches
  BZ,
  BPL,
  BMI,
}
