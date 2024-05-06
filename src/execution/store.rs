use crate::binary::{
    instruction::Instruction,
    types::{FuncType, ValueType},
};

#[derive(Clone)]
pub struct Func {
    pub locals: Vec<ValueType>,
    pub body: Vec<Instruction>,
}

#[derive(Clone)]
pub struct InternalFuncInst {
    pub func_type: FuncType,
    pub code: Func,
}

#[derive(Clone)]
pub enum FuncInst {
    Internal(InternalFuncInst),
}

#[derive(Default)]
pub struct Store {
    pub funcs: Vec<FuncInst>,
}