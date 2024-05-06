use super::{instruction::{self, Instruction}, opcode::Opcode, section::{Function, SectionCode}, types::{FuncType, FunctionLocal, ValueType}};
use nom::{IResult, number::complete::{le_u32, le_u8}, bytes::complete::{tag, take}, sequence::pair, multi::many0};
use nom_leb128::leb128_u32;
use num_traits::FromPrimitive as _;

#[derive(Debug, PartialEq, Eq)]
pub struct Module {
    pub magic: String,
    pub version: u32,
    pub type_section: Option<Vec<FuncType>>,
    pub function_section: Option<Vec<u32>>,
    pub code_section: Option<Vec<Function>>,
}

impl Default for Module {
    fn default() -> Self {
        Self {
            magic: "\0asm".to_string(),
            version: 1,
            type_section: None,
            function_section: None,
            code_section: None,
        }
    }
}

impl Module {
    pub fn new(input: &[u8]) -> anyhow::Result<Module> {
        let (_, module) = Module::decode(input).map_err(|_| anyhow::anyhow!("Failed to decode module"))?;
        Ok(module)
    }

    fn decode(input: &[u8]) -> IResult<&[u8], Module> {
        let (input, _) = tag(b"\0asm")(input)?;
        let (input, version) = le_u32(input)?;

        let mut module = Module {
            magic: "\0asm".into(),
            version,
            ..Default::default()
        };

        let mut remaining = input;

        while !remaining.is_empty() {
            match decode_section_header(remaining) {
                Ok((input, (code, size))) => {
                    let (rest, section_contents) = take(size)(input)?;

                    match code {
                       SectionCode::Type => {
                            let (_, types) = decode_type_section(section_contents)?;
                            module.type_section = Some(types);
                        }
                        SectionCode::Function => {
                            let (_, func_idx_list) = decode_function_section(section_contents)?;
                            module.function_section = Some(func_idx_list);
                        }
                        SectionCode::Code => {
                            let (_, functions) = decode_code_section(section_contents)?;
                            module.code_section = Some(functions);
                        }
                        _ => todo!(),
                    };
                    remaining = rest;
                }
                Err(err) => return Err(err),
            }
        }
        Ok((input, module))
    }
}

fn decode_section_header(input: &[u8]) -> IResult<&[u8], (SectionCode, u32)> {
    let (input, (code, size)) = pair(le_u8, leb128_u32)(input)?;
    Ok((
        input,
        (
            SectionCode::from_u8(code).expect("unexpected section code"),
            size,
        ),
    ))
}

fn decode_value_type(input: &[u8]) -> IResult<&[u8], ValueType> {
    let (input, value_type) = le_u8(input)?;
    Ok((input, value_type.into()))
}

fn decode_type_section(input: &[u8]) -> IResult<&[u8], Vec<FuncType>> {
    let mut func_types: Vec<FuncType> = vec![];

    let (mut input, count) = leb128_u32(input)?; // 1

    for _ in 0..count {
        let (rest, _) = le_u8(input)?; // 2
        let mut func = FuncType::default();

        let (rest, size) = leb128_u32(rest)?; // 3
        let (rest, types) = take(size)(rest)?;
        let (_, types) = many0(decode_value_type)(types)?; // 4
        func.params = types;

        let (rest, size) = leb128_u32(rest)?; // 5
        let (rest, types) = take(size)(rest)?;
        let (_, types) = many0(decode_value_type)(types)?; // 6
        func.results = types;

    // TODO: 引数と戻り値のデコード
        func_types.push(func);
        input = rest;
    }

    Ok((&[], func_types))
}

fn decode_function_section(input: &[u8]) -> IResult<&[u8], Vec<u32>> {
    let mut func_idx_list = vec![];
    let (mut input, count) = leb128_u32(input)?;

    for _ in 0..count {
        let (rest, func_idx) = leb128_u32(input)?;
        func_idx_list.push(func_idx);
        input = rest;
    }

    Ok((input, func_idx_list))
}

fn decode_code_section(input: &[u8]) -> IResult<&[u8], Vec<Function>> {
    let mut functions = vec![];
    let (mut input, count) = leb128_u32(input)?;

    for _ in 0..count {
        let (rest, size) = leb128_u32(input)?;
        let (rest, body) = take(size)(rest)?;
        let (_, body) = decode_function_body(body)?;
        functions.push(body);
        input = rest;
    }
    Ok((&[], functions))
}

fn decode_function_body(input: &[u8]) -> IResult<&[u8], Function> {
    let mut body = Function::default();
    let (mut input, count) = leb128_u32(input)?;

    for _ in 0..count {
        let (rest, type_count) = leb128_u32(input)?;
        let (rest, value_type) = le_u8(rest)?;
        body.locals.push(FunctionLocal {
            type_count,
            value_type: value_type.into(),
        });
        input = rest;
    }

    let mut remaining = input;

    while !remaining.is_empty() {
        let (rest, instruction) = decode_instructions(remaining)?;
        body.code.push(instruction);
        remaining = rest;
    }

    Ok((&[], body))
}

fn decode_instructions(input: &[u8]) -> IResult<&[u8], Instruction> {
    let (input, byte) = le_u8(input)?;
    let opcode = Opcode::from_u8(byte).unwrap_or_else(|| panic!("unexpected opcode: {}", byte));
    let (rest, instruction) = match opcode {
        Opcode::LocalGet => {
            let (rest, index) = leb128_u32(input)?;
            (rest, Instruction::LocalGet(index))
        }
        Opcode::I32Add => (input, Instruction::I32Add),
        Opcode::End => (input, Instruction::End),
    };
    Ok((rest, instruction))
}

#[cfg(test)]
mod tests {
    use crate::binary::{module::Module, instruction::Instruction, section::Function, types::{FuncType, ValueType, FunctionLocal}};
    use anyhow::Result;

    #[test]
    fn decode_module() -> Result<()> {
        let wasm = wat::parse_str("(module)")?;
        let module = Module::new(&wasm)?;
        assert_eq!(module, Module::default());
        Ok(())
    }

    #[test]
    fn decode_func() -> Result<()> {
        let wasm = wat::parse_str("(module (func))")?;
        let module = Module::new(&wasm)?;
        assert_eq!(
            module,
            Module {
                type_section: Some(vec![FuncType::default()]),
                function_section: Some(vec![0]),
                code_section: Some(vec![Function {locals:vec![], code:vec![Instruction::End] }]),
                ..Default::default()
            }
        );
        Ok(())
    }

    #[test]
    fn decode_func_param() -> Result<()> {
        let wasm = wat::parse_str("(module (func (param i32)))")?;
        let module = Module::new(&wasm)?;
        assert_eq!(
            module,
            Module {
                type_section: Some(vec![FuncType {
                    params: vec![ValueType::I32],
                    results: vec![],
                }]),
                function_section: Some(vec![0]),
                code_section: Some(vec![Function {locals:vec![], code:vec![Instruction::End] }]),
                ..Default::default()
            }
        );
        Ok(())
    }

    #[test]
    fn decode_func_local() -> Result<()> {
        let wasm = wat::parse_file("src/fixtures/func_local.wat")?;
        let module = Module::new(&wasm)?;
        assert_eq!(
            module,
            Module {
                type_section: Some(vec![FuncType::default()]),
                function_section: Some(vec![0]),
                code_section: Some(vec![Function {
                    locals: vec![
                        FunctionLocal {
                            type_count: 1,
                            value_type: ValueType::I32,
                        },
                        FunctionLocal {
                            type_count: 2,
                            value_type: ValueType::I64,
                        },
                    ],
                    code: vec![Instruction::End],
                }]),
                ..Default::default()
            }
        );
        Ok(())
    }

    #[test]
    fn decode_func_add() -> Result<()> {
        let wasm = wat::parse_file("src/fixtures/func_add.wat")?;
        let module = Module::new(&wasm)?;
        assert_eq!(
            module,
            Module {
                type_section: Some(vec![FuncType {
                    params: vec![ValueType::I32, ValueType::I32],
                    results: vec![ValueType::I32],
                }]),
                function_section: Some(vec![0]),
                code_section: Some(vec![Function {
                    locals: vec![],
                    code: vec![
                        Instruction::LocalGet(0),
                        Instruction::LocalGet(1),
                        Instruction::I32Add,
                        Instruction::End
                    ],
                }]),
                ..Default::default()
            }
        );
        Ok(())
    }
}