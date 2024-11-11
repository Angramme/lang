#![feature(box_patterns)]

use std::{error::Error, path::PathBuf};

use inkwell::{context::Context, OptimizationLevel};
use crate::codegen::CodeGen;

pub mod tokenizer;
pub mod parser;
pub mod expression;
pub mod codegen;
pub mod error;
pub mod block;

use clap::Parser;

/// A simple compiler for a simple language
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The path to the file to compile and run
    #[arg()]
    path: PathBuf,
}


fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    
    let mut parser = parser::Parser::try_from(args.path.as_path()).expect("Failed to create parser");
    let ast = parser.next().unwrap()?;


    let context = Context::create();
    let module = context.create_module("sum");
    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None)?;
    let mut codegen = CodeGen {
        context: &context,
        module,
        builder: context.create_builder(),
        execution_engine,
    };

    let main = codegen.compile_main(&ast)?;

    let x = 0u64;
    let y = 0u64;

    unsafe {
        println!("output: {}", main.call(x, y));
    }

    Ok(())
}
