use crate::block::Block as BlockO;
use crate::tokenizer::Token;
use crate::parser::{Parsable, Parser};

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(f64),
    Variable(String),
    Block(BlockO),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
}

impl Expression {
    fn precedence(c: char) -> i8 {
        match c {
            '*' | '/' => 1,
            '+' | '-' => 2,
            ')' => 100,
            _ => 100,
        }
    }

    fn parse_prec(parser: &mut Parser, prec: i8) -> Result<Expression, String> {  
        use Token::*;       
        use Expression::*;       
        let left = if prec == 0 {
            if parser.tokens.peek() == Some(&Ok(Operator('{'))) {
                let block = BlockO::parse(parser)?;
                if !block.has_value() {
                    return Err("Expected block with return value".to_string());
                }
                Block(block)
            } else {
                match parser
                    .tokens
                    .next()
                    .ok_or("Expected expression but found end of input")??
                {
                    Number(n) => Ok(Literal(n.parse().map_err(|e| format!("Failed to parse number: {}", e))?)),
                    Symbol(s) => Ok(Variable(s)),
                    Operator('(') => {
                        let inside = Expression::parse(parser)?;
                        parser.tokens.expect_operator_of(')')?;
                        Ok(inside)
                    }
                    x => Err(format!("Expected number or symbol but found {}", x)),
                }?
            }
        } else {
            Expression::parse_prec(parser, prec - 1)?
        };

        match parser.tokens.peek() {
            None => return Ok(left),
            Some(Err(err)) => return Err(err.clone()),
            Some(Ok(Operator(c))) if Expression::precedence(*c) <= prec => (),
            Some(Ok(_)) => return Ok(left),
        }

        let operator = match parser
            .tokens
            .next()
            .ok_or("Expected operator but found end of input")??
        {
            Operator(c) => Ok(c),
            x => Err(format!("Expected operator but found {}", x)),
        }?;
        let right = Expression::parse_prec(parser, prec)?;
        match operator {
            '*' => Ok(Mul(Box::new(left), Box::new(right))),
            '/' => Ok(Div(Box::new(left), Box::new(right))),
            '+' => Ok(Add(Box::new(left), Box::new(right))),
            '-' => Ok(Sub(Box::new(left), Box::new(right))),
            x => Err(format!("Expected valid operator but found {}", x)),
        }
    }
}

impl Parsable for Expression {
    fn parse(parser: &mut Parser) -> Result<Self, String> {
        Expression::parse_prec(parser, 3)
    }
}


#[cfg(test)]
mod tests {
    use crate::block::Line;
    use crate::parser::Ast;
    use std::fs::File;
    use std::io::Write;
    use std::env::temp_dir;

    use super::*;

    fn test(data: &'static str) -> Option<Result<Ast, String>> {
        let mut parser = Parser::try_from(data).expect("Failed to create parser");
        parser.next()
    }

    #[test]
    fn test_parse_literal() {
        match test("42").unwrap().unwrap() {
            Ast::Expression(Expression::Literal(42.)) => (),
            x => panic!("Expected literal 42 ; got {:?}", x),
        }
    }

    #[test]
    fn test_parse_variable() {
        match test("x").unwrap().unwrap() {
            Ast::Expression(Expression::Variable(ref s)) if s == "x" => (),
            x => panic!("Expected variable x ; got {:?}", x),
        }
    }

    #[test]
    fn test_parse_addition() {
        match test("1 + 2").unwrap().unwrap() {
            Ast::Expression(Expression::Add(
                box Expression::Literal(1.),
                box Expression::Literal(2.),
            )) => (),
            x => panic!("Expected addition of 1 and 2 ; got {:?}", x),
        }
    }

    #[test]
    fn test_parse_subtraction() {
        match test("3 - 1").unwrap().unwrap() {
            Ast::Expression(Expression::Sub(
                box Expression::Literal(3.),
                box Expression::Literal(1.),
            )) => (),
            x => panic!("Expected subtraction of 3 and 1 ; got {:?}", x),
        }
    }

    #[test]
    fn test_parse_multiplication() {
        match test("4 * 2").unwrap().unwrap() {
            Ast::Expression(Expression::Mul(
                box Expression::Literal(4.),
                box Expression::Literal(2.),
            )) => (),
            x => panic!("Expected multiplication of 4 and 2 ; got {:?}", x),
        }
    }

    #[test]
    fn test_parse_division() {
        match test("8 / 4").unwrap().unwrap() {
            Ast::Expression(Expression::Div(
                box Expression::Literal(8.),
                box Expression::Literal(4.),
            )) => (),
            x => panic!("Expected division of 8 by 4 ; got {:?}", x),
        }
    }

    #[test]
    fn test_parse_complex_expression() {
        match test("1 + 2 * 3").unwrap().unwrap() {
            Ast::Expression(Expression::Add(
                box Expression::Literal(1.),
                box Expression::Mul(box Expression::Literal(2.), box Expression::Literal(3.)),
            )) => (),
            x => panic!("Expected complex expression 1 + 2 * 3 ; got {:?}", x),
        }
    }

    #[test]
    fn test_parse_precedence() {
        match test("1 * 2 + 3").unwrap().unwrap() {
            Ast::Expression(Expression::Add(
                box Expression::Mul(box Expression::Literal(1.), box Expression::Literal(2.)),
                box Expression::Literal(3.),
            )) => (),
            x => panic!("Expected complex expression 1 * 2 + 3 ; got {:?}", x),
        }
    }

    #[test]
    fn test_parse_big() {
        match test("1 + 2 * 3 * 4 + 5").unwrap().unwrap() {
            Ast::Expression(Expression::Add(
                box Expression::Literal(1.),
                box Expression::Add(
                    box Expression::Mul(
                        box Expression::Literal(2.),
                        box Expression::Mul(box Expression::Literal(3.), box Expression::Literal(4.)),
                    ),
                    box Expression::Literal(5.),
                )
            )) => (),
            x => panic!("Expected complex expression 1 + 2 * 3 * 4 + 5 ; got {:?}", x),
        }
    }

    #[test]
    fn test_on_file() {
        let data = "1 + 2   ";
        let mut temp_file_path = temp_dir();
        temp_file_path.push("test_expression.txt");

        let mut file = File::create(&temp_file_path).expect("Failed to create temporary file");
        file.write_all(data.as_bytes()).expect("Failed to write to temporary file");

        let mut parser = Parser::try_from(temp_file_path.as_path()).expect("Failed to create parser");
        let ast = parser.next().unwrap().unwrap();
        match ast {
            Ast::Expression(Expression::Add(
                box Expression::Literal(1.),
                box Expression::Literal(2.),
            )) => (),
            x => panic!("Expected addition of 1 and 2 ; got {:?}", x),
        }
    }

    #[test]
    fn test_parantheses() {
        match test("(1 + 2 ) * 3").unwrap().unwrap() {
            Ast::Expression(Expression::Mul(
                box Expression::Add(
                    box Expression::Literal(1.),
                    box Expression::Literal(2.),
                ),
                box Expression::Literal(3.),
            )) => (),
            x => panic!("Expected parantheses (1 + 2) * 3 ; got {:?}", x),
        }
    }

    #[test]
    fn test_parantheses2() {
        match test("1 * (2 + 3)").unwrap().unwrap() {
            Ast::Expression(Expression::Mul(
                box Expression::Literal(1.),
                box Expression::Add(
                    box Expression::Literal(2.),
                    box Expression::Literal(3.),
                ),
            )) => (),
            x => panic!("Expected parantheses 1 * (2 + 3) ; got {:?}", x),
        }
    }

    #[test]
    fn test_parantheses3() {
        match test("(1 * 2) + (3 * 4)").unwrap().unwrap() {
            Ast::Expression(Expression::Add(
                box Expression::Mul(
                    box Expression::Literal(1.),
                    box Expression::Literal(2.),
                ),
                box Expression::Mul(
                    box Expression::Literal(3.),
                    box Expression::Literal(4.),
                ),
            )) => (),
            x => panic!("Expected parantheses (1 * 2) + (3 * 4) ; got {:?}", x),
        }
    }

    #[test]
    fn test_block() {
        match test("12 + { 30 }").unwrap().unwrap() {
            Ast::Expression(Expression::Add(
                box Expression::Literal(12.),
                box Expression::Block(block),
            )) => {
                assert!(!block.lines.is_empty());
                assert!(block.has_value());
                match &block.lines[0] {
                    Line::ReturnStatement(expr) => match expr {
                        Expression::Literal(30.) => (),
                        x => panic!("Expected block 12 + {{ 30 }} ; got {:?}", x),
                    },
                    x => panic!("Expected block 12 + {{ 30 }} ; got {:?}", x),
                }
            },
            x => panic!("Expected block 12 + {{ 30 }} ; got {:?}", x),
        }
    }

    #[test]
    fn test_block2() {
        match test("{ 30; } + 12").unwrap() {
            Err(_) => (),
            x => panic!("Expected error ; got {:?}", x),
        }
    }
}