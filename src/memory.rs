use bct::{Tryte};

#[derive(Debug)]
pub enum MemError {
  Unknown,
}

pub trait Memory {
  fn load_tryte(&self, addr: i32) -> Result<Tryte, MemError>;
  fn store_tryte(&mut self, addr: i32, val: Tryte) -> Result<(), MemError>;

  // fn load_word(&mut self, addr: i32) -> Result<Ternary, MemError>;
  // fn store_word(&mut self, addr: i32, val: Ternary);
}
