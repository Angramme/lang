use std::{fmt::{self, Display, Formatter}, fs::File, io::{BufRead, BufReader}, iter::Peekable, path::Path};


pub struct Tokenizer {
    inner: Peekable<TokenizerInner>,
}

struct TokenizerInner {
    chars: Peekable<Box<dyn Iterator<Item=char>>>,
}

impl TryFrom<&Path> for TokenizerInner {
    type Error = std::io::Error;
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let file = File::open(path)?;
        let iterator = BufReader::new(file).lines();
        let iterator = iterator.map(|line| line.unwrap().chars().collect::<Vec<_>>()).flatten();
        let iterator: Box<dyn Iterator<Item = char>> = Box::new(iterator);
        Ok(TokenizerInner{chars: iterator.peekable()})
    }
}

impl TryFrom<&'static str> for TokenizerInner {
    type Error = std::io::Error;
    fn try_from(data: &'static str) -> Result<Self, Self::Error> {
        let iterator = data.chars();
        let iterator: Box<dyn Iterator<Item = char>> = Box::new(iterator);
        Ok(TokenizerInner{chars: iterator.peekable()})
    }
}

impl TryFrom<&Path> for Tokenizer {
    type Error = std::io::Error;
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let inner = TokenizerInner::try_from(path)?;
        Ok(Tokenizer { inner: inner.peekable() })
    }
}

impl TryFrom<&'static str> for Tokenizer {
    type Error = std::io::Error;
    fn try_from(data: &'static str) -> Result<Self, Self::Error> {
        let inner = TokenizerInner::try_from(data)?;
        Ok(Tokenizer { inner: inner.peekable() })
    }
}

#[derive(Debug, PartialEq)]
pub enum Token{
    Number(String),
    Operator(char),
    Symbol(String),
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Token::Number(n) => write!(f, "Number({})", n),
            Token::Operator(c) => write!(f, "Operator({})", c),
            Token::Symbol(s) => write!(f, "Symbol({})", s),
        }
    }
}

impl TokenizerInner {
    fn next_number(&mut self) -> Result<Token, String> {
        let mut str = String::new();
        while let Some(c) = self.chars.peek() {
            if !c.is_numeric() { break; }
            str.push(*c);
            self.chars.next();
        }
        if let Some(c) = self.chars.peek() {
            if c.is_alphanumeric() {
                return Err("Number cannot be followed by a letter".to_string());
            }
        } 
        Ok(Token::Number(str))
    }
    fn next_operator(&mut self) -> Result<Token, String> {
        let c = self.chars.next().ok_or("Expected operator but found end of input")?;
        Ok(Token::Operator(c))
    }
    fn next_symbol(&mut self) -> Result<Token, String> {
        let mut str = String::new();
        while let Some(c) = self.chars.peek() {
            if !c.is_alphanumeric() { break; }
            str.push(*c);
            self.chars.next();
        }
        Ok(Token::Symbol(str))
    }
}

impl Iterator for Tokenizer {
    type Item = Result<Token, String>;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl Tokenizer {
    pub fn peek(&mut self) -> Option<&Result<Token, String>> {
        self.inner.peek()
    }

    pub fn expect_symbol(&mut self) -> Result<String, String> {
        match self.next() {
            Some(Ok(Token::Symbol(s))) => Ok(s),
            Some(Ok(Token::Number(n))) => Err(format!("Expected symbol but found number: {}", n)),
            Some(Ok(Token::Operator(c))) => Err(format!("Expected symbol but found operator: {}", c)),
            Some(Err(e)) => Err(e),
            None => Err("Expected symbol but found end of input".to_string()),
        }
    }

    pub fn expect_operator(&mut self) -> Result<char, String> {
        match self.next() {
            Some(Ok(Token::Operator(c))) => Ok(c),
            Some(Ok(Token::Number(n))) => Err(format!("Expected operator but found number: {}", n)),
            Some(Ok(Token::Symbol(s))) => Err(format!("Expected operator but found symbol: {}", s)),
            Some(Err(e)) => Err(e),
            None => Err("Expected operator but found end of input".to_string()),
        }
    }

    pub fn expect_symbol_of(&mut self, expected: &str) -> Result<(), String> {
        match self.expect_symbol() {
            Ok(s) if s == expected => Ok(()),
            Ok(s) => Err(format!("Expected symbol '{}' but found '{}'", expected, s)),
            Err(e) => Err(e),
        }
    }

    pub fn expect_operator_of(&mut self, expected: char) -> Result<(), String> {
        match self.expect_operator() {
            Ok(c) if c == expected => Ok(()),
            Ok(c) => Err(format!("Expected operator '{}' but found '{}'", expected, c)),
            Err(e) => Err(e),
        }
    }
}

impl Iterator for TokenizerInner {
    type Item = Result<Token, String>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.chars.peek() {
            Some(c) if c.is_whitespace() => {self.chars.next(); self.next()},
            Some(c) if c.is_numeric() => Some(self.next_number()),
            Some(c) if c.is_ascii_punctuation() => Some(self.next_operator()),
            Some(c) if c.is_alphanumeric() => Some(self.next_symbol()),
            Some(_) => None,
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer_mixed_input() {
        let data = "3+5 *2-8 /4";
        let mut tokenizer = TokenizerInner::try_from(data).unwrap();
        assert_eq!(tokenizer.next(), Some(Ok(Token::Number("3".to_string()))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::Operator('+'))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::Number("5".to_string()))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::Operator('*'))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::Number("2".to_string()))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::Operator('-'))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::Number("8".to_string()))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::Operator('/'))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::Number("4".to_string()))));
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn test_tokenizer_empty_input() {
        let data = "";
        let mut tokenizer = TokenizerInner::try_from(data).unwrap();
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn test_tokenizer_variable() {
        let data = "abc";
        let mut tokenizer = TokenizerInner::try_from(data).unwrap();
        assert_eq!(tokenizer.next(), Some(Ok(Token::Symbol("abc".to_string()))));
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn test_tokenizer_number() {
        let data = "42";
        let mut tokenizer = TokenizerInner::try_from(data).unwrap();
        assert_eq!(tokenizer.next(), Some(Ok(Token::Number("42".to_string()))));
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn test_tokenizer_mixed_symbols() {
        let data = "var1 + var2";
        let mut tokenizer = TokenizerInner::try_from(data).unwrap();
        assert_eq!(tokenizer.next(), Some(Ok(Token::Symbol("var1".to_string()))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::Operator('+'))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::Symbol("var2".to_string()))));
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn test_tokenizer_symbols_with_numbers() {
        let data = "var123 + 456var";
        let mut tokenizer = TokenizerInner::try_from(data).unwrap();
        assert_eq!(tokenizer.next(), Some(Ok(Token::Symbol("var123".to_string()))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::Operator('+'))));
        let x = tokenizer.next();
        assert!(x.is_some());
        assert!(x.unwrap().is_err());
    }
}
