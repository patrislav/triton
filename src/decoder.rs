use bct::{Tryte, Word};
use bct::translate::i8_trybble_to_bct;
use crate::memory::{Memory, MemError};
use crate::instructions::{Instr, ISA};

pub enum DecodeError {
  IllegalInstruction { pos: i32, op: i8 },
  MemoryError(MemError),
  Unknown,
}

pub struct DecodeResult(pub Instr, pub Instr);

fn decode_simple_instruction_trybble(value: i8, err: DecodeError) -> Result<Instr, DecodeError> {
  match value {
    -10 => Ok(Instr::STOR),
    -9 => Ok(Instr::ISU),
    -8 => Ok(Instr::LOAD),
    -7 => Ok(Instr::INC),
    -6 => Ok(Instr::MAX),
    -5 => Ok(Instr::IST),
    -1 => Ok(Instr::NEG),
    0 => Ok(Instr::NOP),
    1 => Ok(Instr::ADD),
    2 => Ok(Instr::ROT),
    3 => Ok(Instr::SWAP),
    4 => Ok(Instr::DUP),
    5 => Ok(Instr::SHL),
    6 => Ok(Instr::DROP),
    7 => Ok(Instr::SHR),
    _ => Err(err),
  }
}

fn decode_first_trybble(value: i8, next: i8, pc: &mut i32) -> Result<Instr, DecodeError> {
  match value {
    // Load immediate trybble
    -12 => Ok(Instr::LIT(Tryte::from_bct_trybbles(0, i8_trybble_to_bct(next)))),
    _ => decode_simple_instruction_trybble(value, DecodeError::IllegalInstruction { pos: *pc, op: value }),
  }
}

fn decode_second_trybble(value: i8, mem: &dyn Memory, pc: &mut i32) -> Result<Instr, DecodeError> {
  match value {
    // Load next tryte
    -12 => {
      *pc += 1;
      match mem.load_tryte(*pc) {
        Ok(tryte) => Ok(Instr::LIT(tryte)),
        Err(err) => Err(DecodeError::MemoryError(err)),
      }
    },
    // Load next word
    -11 => {
      *pc += 1;
      match mem.load_tryte(*pc) {
        Ok(hi) => {
          *pc += 1;
          match mem.load_tryte(*pc) {
            Ok(lo) => {
              Ok(Instr::WORD(Word::from_trytes(hi, lo)))
            },
            Err(err) => Err(DecodeError::MemoryError(err)),
          }
        },
        Err(err) => Err(DecodeError::MemoryError(err)),
      }
    },
    // Support-only instructions
    -13 => Ok(Instr::IRQ),
    8 => Ok(Instr::RET),
    9 => Ok(Instr::JMP),
    10 => Ok(Instr::CALL),
    11 => Ok(Instr::BMI),
    12 => Ok(Instr::BZ),
    13 => Ok(Instr::BPL),
    _ => decode_simple_instruction_trybble(value, DecodeError::IllegalInstruction { pos: *pc, op: value }),
  }
}

pub fn decode_instruction(_isa: &ISA, mem: &dyn Memory, pc: i32) -> (Result<DecodeResult, DecodeError>, i32) {
  let mut next_pc = pc;
  let val = mem.load_tryte(next_pc);

  let res = match val {
    Err(err) => Err(DecodeError::MemoryError(err)),
    Ok(op) => {
      match decode_first_trybble(op.hi_value(), op.lo_value(), &mut next_pc) {
        Err(err) => Err(err),
        Ok(Instr::LIT(x)) => Ok(DecodeResult(Instr::LIT(x), Instr::NOP)),
        Ok(first) => {
          match decode_second_trybble(op.lo_value(), mem, &mut next_pc) {
            Err(err) => Err(err),
            Ok(second) => Ok(DecodeResult(first, second)),
          }
        },
      }
    },
  };
  (res, next_pc + 1)
}

