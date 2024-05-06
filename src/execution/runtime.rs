use super::{store::Store, value::Value};
use crate::binary::{instruction::Instruction, module::Module};
use anyhow::Result;

#[derive(Default)]
pub struct Frame {
    pub pc: isize,               
    pub sp: usize,               
    pub insts: Vec<Instruction>, 
    pub arity: usize,            
    pub locals: Vec<Value>,      
}

#[derive(Default)]
pub struct Runtime {
    pub store: Store,
    pub stack: Vec<Value>,
    pub call_stack: Vec<Frame>,
}

impl Runtime {
    pub fn instantiate(wasm: impl AsRef<[u8]>) -> Result<Self> {
        let module = Module::new(wasm.as_ref())?;
        let store = Store::new(module)?;
        Ok(Self {
            store,
            ..Default::default()
        })
    }
}