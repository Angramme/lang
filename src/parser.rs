use crate::tokenizer::Tokenizer;
use crate::expression::Expression;
use core::str;
use std::path::Path;

pub struct Parser {
    pub tokens: Tokenizer,
}

impl TryFrom<&Path> for Parser {
    type Error = std::io::Error;
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let tokenizer = Tokenizer::try_from(path)?;
        Ok(Parser {
            tokens: tokenizer,
        })
    }
}

impl TryFrom<&'static str> for Parser {
    type Error = std::io::Error;
    fn try_from(path: &'static str) -> Result<Self, Self::Error> {
        let tokenizer = Tokenizer::try_from(path)?;
        Ok(Parser {
            tokens: tokenizer,
        })
    }
}

impl Parser {
    fn next_of<T: Parsable>(&mut self) -> Result<T, String> {
        T::parse(self)
    }
}

impl Iterator for Parser {
    type Item = Result<Ast, String>;
    fn next(&mut self) -> Option<Self::Item> {
        fn nexxt(this: &mut Parser) -> <Parser as IntoIterator>::Item {
            Ok(Ast::Expression(this.next_of()?))
        }

        match self.tokens.peek() {
            Some(_) => Some(nexxt(self)),
            _ => None,
        }
    }
}

pub trait Parsable: Sized {
    fn parse(parser: &mut Parser) -> Result<Self, String>;
}

#[derive(Debug)]
pub enum Ast {
    Expression(Expression),
}
