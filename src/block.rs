use crate::tokenizer::Token;
use crate::{expression::Expression, parser::Parsable};

#[derive(Debug, Clone)]
pub enum Line {
    Expression(Expression),
    LetStatement {
        name: String,
        value: Expression,
        type_: Option<String>,
    },
    ReturnStatement(Expression),
}

#[derive(Debug, Clone)]
pub struct Block {
    pub lines: Vec<Line>,
}

impl Block {
    pub fn has_value(&self) -> bool {
        self.lines.last().map_or(false, |line| match line {
            Line::ReturnStatement(_) => true,
            _ => false,
        })
    }

    fn parse_let(parser: &mut crate::parser::Parser) -> Result<Line, String> {
        use Token as T;
        parser.tokens.next();
        let name = parser.tokens.expect_symbol()?;
        parser.tokens.expect_operator_of(':')?;
        let type_ = match parser.tokens.next() {
            Some(Ok(T::Symbol(type_))) => Some(type_),
            Some(Ok(T::Operator('='))) => None,
            Some(Err(e)) => return Err(e),
            _ => return Err(format!("Expected type or '=' after let {}:", name)),
        };
        if type_.is_some() {
            parser.tokens.expect_operator_of('=')?;
        }
        let value = Expression::parse(parser)?;
        Ok(Line::LetStatement { name, value, type_ })
    }

    fn parse_return(parser: &mut crate::parser::Parser) -> Result<Line, String> {
        parser.tokens.next();
        let value = Expression::parse(parser)?;
        Ok(Line::ReturnStatement(value))
    }
}

impl Parsable for Block {
    fn parse(parser: &mut crate::parser::Parser) -> Result<Self, String> {
        use Token as T;
        let mut lines = Vec::new();
        parser.tokens.expect_operator_of('{')?;
        while let Some(token) = parser.tokens.peek() {
            let token = (token.as_ref())?;
            lines.push(if *token == T::Symbol("let".to_string()) {
                Block::parse_let(parser)?
            } else if *token == T::Symbol("return".to_string()) {
                Block::parse_return(parser)?
            } else if *token == T::Operator('}') {
                break;
            } else {
                let value = Expression::parse(parser)?;
                Line::Expression(value)
            });
            match parser.tokens.next() {
                Some(Err(e)) => return Err(e),
                Some(Ok(T::Operator(';'))) => (),
                Some(Ok(T::Operator('}'))) => match lines.last() {
                    Some(Line::Expression(expr)) => {
                        *lines.last_mut().unwrap() = Line::ReturnStatement(expr.clone());
                        break
                    },
                    _ => return Err("Expected expression before '}' or ';' operator".to_string()),
                },
                Some(Ok(t)) => return Err(format!("Expected ';' but found '{}'", t)),
                None => return Err("Expected ';' but found end of input".to_string()),
            }
            if Some(&Ok(T::Operator('}'))) == parser.tokens.peek() {
                parser.tokens.next();
                break;
            }
        }
        Ok(Block { lines: lines })
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::Parser;

    use super::*;

    fn parse_block(input: &'static str) -> Result<Block, String> {
        let mut parser = Parser::try_from(input as &'static str).map_err(|e| e.to_string())?;
        Block::parse(&mut parser)
    }

    #[test]
    fn test_empty_block() {
        let input = "{}";
        let result = parse_block(input);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert!(block.lines.is_empty());
    }

    #[test]
    fn test_single_expression() {
        let input = "{ 42; }";
        let result = parse_block(input);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(block.lines.len(), 1);
        match &block.lines[0] {
            Line::Expression(_) => (),
            _ => panic!("Expected an expression"),
        }
    }

    #[test]
    fn test_let_statement() {
        let input = "{ let x: i32 = 42; }";
        let result = parse_block(input);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(block.lines.len(), 1);
        match &block.lines[0] {
            Line::LetStatement { name, value, type_ } => {
                assert_eq!(name, "x");
                assert!(type_.is_some());
                assert_eq!(type_.as_ref().unwrap(), "i32");
                match value {
                    Expression::Literal(literal) => assert_eq!(*literal, 42.),
                    _ => panic!("Expected a literal expression with value 42"),
                }
            }
            _ => panic!("Expected a let statement"),
        }
    }

    #[test]
    fn test_let_statement2() {
        let input = "{ let x := 42; }";
        let result = parse_block(input);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(block.lines.len(), 1);
        match &block.lines[0] {
            Line::LetStatement { name, value, type_ } => {
                assert_eq!(name, "x");
                assert!(type_.is_none());
                match value {
                    Expression::Literal(literal) => assert_eq!(*literal, 42.),
                    _ => panic!("Expected a literal expression with value 42"),
                }
            }
            _ => panic!("Expected a let statement"),
        }
    }

    #[test]
    fn test_return_statement() {
        let input = "{ return 42; }";
        let result = parse_block(input);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(block.lines.len(), 1);
        match &block.lines[0] {
            Line::ReturnStatement(_) => (),
            _ => panic!("Expected a return statement"),
        }
    }

    #[test]
    fn test_multiple_lines() {
        let input = "{ let x: i32 = 42; return x; }";
        let result = parse_block(input);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(block.lines.len(), 2);
        match &block.lines[0] {
            Line::LetStatement { name, value, type_ } => {
                assert_eq!(name, "x");
                assert!(type_.is_some());
                assert_eq!(type_.as_ref().unwrap(), "i32");
                match value {
                    Expression::Literal(literal) => assert_eq!(*literal, 42.),
                    _ => panic!("Expected a literal expression with value 42"),
                }
            }
            _ => panic!("Expected a let statement"),
        }
        match &block.lines[1] {
            Line::ReturnStatement(_) => (),
            _ => panic!("Expected a return statement"),
        }
    }
    
    #[test]
    fn test_implicit_return() {
        let input = "{ 42 }";
        let result = parse_block(input);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(block.lines.len(), 1);
        match &block.lines[0] {
            Line::ReturnStatement(Expression::Literal(42.)) => (),
            _ => panic!("Expected a return statement"),
        }
    }
}
