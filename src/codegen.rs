use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::values::AnyValueEnum;

use crate::block::Block;
use crate::expression::Expression;
use crate::parser::Ast;

type JitMain = unsafe extern "C" fn(u64, u64) -> f64;

pub struct CodeGen<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub execution_engine: ExecutionEngine<'ctx>,
}


impl<'ctx> CodeGen<'ctx> {
    pub fn compile<T: Compilable>(&mut self, obj: &T) -> Result<inkwell::values::AnyValueEnum<'ctx>, String> 
    {
        obj.compile(self)
    }

    pub fn compile_main<T: Compilable>(&mut self, obj: &T) -> Result<JitFunction<JitMain>, String> {
        let f64_type = self.context.f64_type();
        let i64_type = self.context.i64_type();
        let fn_type = f64_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let function = self.module.add_function("sum", fn_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");

        self.builder.position_at_end(basic_block);
        
        let compiled = self.compile(obj)?;

        self.builder.build_return(Some(&compiled.into_float_value())).unwrap();

        unsafe { self.execution_engine.get_function("sum").map_err(|e| e.to_string()) }
    }
}

pub trait Compilable {
    fn compile<'ctx>(&self, code_gen: &CodeGen<'ctx>) -> Result<inkwell::values::AnyValueEnum<'ctx>, String>;
}

impl Compilable for Expression {
    fn compile<'ctx>(&self, code_gen: &CodeGen<'ctx>) -> Result<AnyValueEnum<'ctx>, String> {
        Ok(match self {
            Expression::Literal(x) => {
                let i64_type = code_gen.context.f64_type();
                i64_type.const_float(*x as f64).into()
            },
            Expression::Variable(_) => todo!(),
            Expression::Add(a, b) => {
                let x = a.compile(code_gen)?;
                let y = b.compile(code_gen)?;
                code_gen.builder.build_float_add(x.into_float_value(), y.into_float_value(), "sum").unwrap().into()
            },
            Expression::Sub(a, b) => {
                let x = a.compile(code_gen)?;
                let y = b.compile(code_gen)?;
                code_gen.builder.build_float_sub(x.into_float_value(), y.into_float_value(), "sub").unwrap().into()
            },
            Expression::Mul(a, b) => {
                let x = a.compile(code_gen)?;
                let y = b.compile(code_gen)?;
                code_gen.builder.build_float_mul(x.into_float_value(), y.into_float_value(), "mul").unwrap().into()
            },
            Expression::Div(a, b) => {
                let x = a.compile(code_gen)?;
                let y = b.compile(code_gen)?;
                code_gen.builder.build_float_div(x.into_float_value(), y.into_float_value(), "div").unwrap().into()
            },
            Expression::Block(b) => b.compile(code_gen)?,
        })
    }
}

impl Compilable for Ast {
    fn compile<'ctx>(&self, code_gen: &CodeGen<'ctx>) -> Result<AnyValueEnum<'ctx>, String> {
        match self {
            Ast::Expression(expr) => expr.compile(code_gen),
        }
    }
}

impl Compilable for Block {
    fn compile<'ctx>(&self, code_gen: &CodeGen<'ctx>) -> Result<AnyValueEnum<'ctx>, String> {
        todo!()
    }
}