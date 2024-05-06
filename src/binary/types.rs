#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct FuncType {
    pub params: Vec<ValueType>,
    pub results: Vec<ValueType>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ValueType {
    I32,
    I64,
}

impl From<u8> for ValueType {
    fn from(value: u8) -> Self {
        match value {
            0x7F => ValueType::I32,
            0x7E => ValueType::I64,
            _ => panic!("unexpected value type"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FunctionLocal {
    pub type_count: u32,
    pub value_type: ValueType,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ExportDesc {
    Func(u32),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Export {
    pub name: String,
    pub desc: ExportDesc,
}
