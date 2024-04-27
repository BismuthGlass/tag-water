use std::collections::HashMap;
use std::collections::HashSet;
use std::iter::Peekable;

use tokenizer::{Token, TokenInfo};

pub struct File {
    pub file: String,
    pub tags: HashSet<String>,
}

pub struct Set {
    pub files: Vec<usize>,
    pub tags: HashSet<String>,
}

pub struct Data {
    directives: HashMap<String, String>,
    variables: HashMap<String, Vec<Token>>,
    pub files: Vec<File>,
    pub sets: Vec<Set>,
    pub tags: HashSet<String>,
    pub file_set: HashSet<String>,
}

impl Data {
    fn new() -> Self {
        Data {
            directives: HashMap::new(),
            variables: HashMap::new(),
            files: Vec::new(),
            sets: Vec::new(),
            tags: HashSet::new(),
            file_set: HashSet::new(),
        }
    }

    pub fn from_tokens(tokens: &[tokenizer::TokenInfo]) -> Result<Data, String> {
        let mut data = Self::new();
        let mut tokens = tokens.iter().peekable();
        while let Some(tkn) = tokens.peek() {
            match tkn.token {
                Token::Variable(_) => data.handle_variable(&mut tokens)?,
                Token::String(_) => data.handle_file(&mut tokens)?,
                Token::Directive(_) => data.handle_directive(&mut tokens)?,
                Token::SetOpen => data.handle_set(&mut tokens)?,
                _ => {
                    tokens.next();
                }
            };
        }
        Ok(data)
    }

    fn handle_tag_list<'a, T>(
        &mut self,
        tokens: &mut Peekable<T>,
    ) -> Result<HashSet<String>, String>
    where
        T: Iterator<Item = &'a TokenInfo>,
    {
        let mut tag_list = HashSet::new();
        while let Some(tkn) = tokens.peek() {
            match &tkn.token {
                Token::Variable(v) => {
                    if !self.variables.contains_key(v) {
                        return Err(format!("Line {}: Undefined variable {}", tkn.line, v));
                    }
                    for tkn in self.variables[v].iter() {
                        match tkn {
                            Token::AddTag(t) => tag_list.insert(t.clone()),
                            Token::RemoveTag(t) => tag_list.remove(t),
                            _ => panic!("Wrong usage"),
                        };
                    }
                    tokens.next();
                }
                Token::AddTag(t) => {
                    tag_list.insert(t.clone());
                    tokens.next();
                }
                Token::RemoveTag(t) => {
                    tag_list.remove(t);
                    tokens.next();
                }
                Token::LineBreak => {
                    tokens.next();
                }
                _ => break,
            }
        }

        for tag in &tag_list {
            self.tags.insert(tag.clone());
        }

        Ok(tag_list)
    }

    fn handle_set<'a, T>(&mut self, tokens: &mut Peekable<T>) -> Result<(), String>
    where
        T: Iterator<Item = &'a TokenInfo>,
    {
        let token = tokens.next().unwrap();
        if token.token != Token::SetOpen {
            panic!("Invalid token");
        }

        let mut file_indices = Vec::new();
        let mut tag_list = HashSet::new();
        let mut store_file_tags = false;
        while let Some(tkn) = tokens.peek() {
            match &tkn.token {
                Token::Directive(d) => {
                    if d != "@autotag" {
                        return Err(format!("Line {}: Unexpected directive {}", tkn.line, d));
                    }
                    store_file_tags = true;
                    tokens.next();
                }
                Token::SetClose => {
                    tokens.next();
                    break;
                }
                Token::String(_) => {
                    file_indices.push(self.files.len());
                    self.handle_file(tokens)?;
                    if store_file_tags {
                        let file_tags = &self.files[self.files.len() - 1].tags;
                        for tag in file_tags {
                            tag_list.insert(tag.clone());
                        }
                    }
                }
                Token::LineBreak => {
                    tokens.next();
                }
                _ => {
                    return Err(format!(
                        "Line {}: Unexpected token {:?}",
                        token.line, tkn.token
                    ));
                }
            }
        }

        for tag in self.handle_tag_list(tokens)? {
            tag_list.insert(tag);
        }

        self.sets.push(Set {
            files: file_indices,
            tags: tag_list,
        });
        Ok(())
    }

    fn handle_file<'a, T>(&mut self, tokens: &mut Peekable<T>) -> Result<(), String>
    where
        T: Iterator<Item = &'a TokenInfo>,
    {
        let token_info = tokens.next().unwrap();
        let file = if let Token::String(v) = &token_info.token {
            v
        } else {
            panic!("Invalid token")
        };

        if self.file_set.contains(file) {
            return Err(format!("Line {}: Repeated file {}", token_info.line, file));
        }
        self.file_set.insert(file.clone());

        let file = File {
            file: file.to_string(),
            tags: self.handle_tag_list(tokens)?,
        };
        self.files.push(file);
        Ok(())
    }

    fn handle_variable<'a, T>(&mut self, tokens: &mut T) -> Result<(), String>
    where
        T: Iterator<Item = &'a TokenInfo>,
    {
        let token_info = tokens.next().unwrap();
        let variable_name = if let Token::Variable(name) = &token_info.token {
            name
        } else {
            panic!("Invalid token")
        };

        let mut token_vec = Vec::new();
        while let Some(tkn) = tokens.next() {
            match &tkn.token {
                Token::AddTag(_) | Token::RemoveTag(_) => token_vec.push(tkn.token.clone()),
                Token::Variable(v) => {
                    if !self.variables.contains_key(v) {
                        return Err(format!("Line {}: Undefined variable {}", tkn.line, v));
                    }
                    self.variables[v].iter().for_each(|tkn| {
                        token_vec.push(tkn.clone());
                    });
                }
                Token::StatementEnd => break,
                _ => break,
            }
        }
        self.variables.insert(variable_name.clone(), token_vec);
        Ok(())
    }

    fn handle_directive<'a, T>(&mut self, tokens: &mut T) -> Result<(), String>
    where
        T: Iterator<Item = &'a TokenInfo>,
    {
        let token_info = tokens.next().unwrap();
        let directive = if let Token::Directive(v) = &token_info.token {
            v.clone()
        } else {
            panic!("Invalid token")
        };

        let value = if let Some(TokenInfo {
            line: _,
            token: Token::String(v),
        }) = tokens.next()
        {
            v.clone()
        } else {
            return Err(format!(
                "Line {}: Empty directive {}",
                token_info.line, directive
            ));
        };

        self.directives.insert(directive, value);
        Ok(())
    }
}

pub mod tokenizer {
    use std::iter::Peekable;
    use std::str::Bytes;

    #[derive(Debug, Clone, PartialEq)]
    pub enum Token {
        SetOpen,
        SetClose,
        String(String),
        Colon,
        Variable(String),
        Directive(String),
        AddTag(String),
        RemoveTag(String),
        Assignment,
        LineBreak,
        StatementEnd,
    }

    #[derive(Debug)]
    pub struct TokenInfo {
        pub line: i64,
        pub token: Token,
    }

    impl TokenInfo {
        pub fn new(line: i64, token: Token) -> Self {
            TokenInfo { line, token }
        }
    }

    fn is_whitespace(b: u8) -> bool {
        b == b' ' || b == b'\r' || b == b'\n' || b == b'\t'
    }

    fn parse_quoted_string<T>(bytes: &mut T) -> String
    where
        T: Iterator<Item = u8> + Clone,
    {
        bytes.next();
        let start = bytes.clone();
        let mut str_len = 0;
        while let Some(b) = bytes.next() {
            if b != b'"' {
                str_len += 1;
            } else {
                return String::from_utf8(start.take(str_len).collect()).unwrap();
            }
        }
        panic!("Unmatched `\"`");
    }

    fn parse_whitespace_terminated<T>(bytes: &mut Peekable<T>) -> String
    where
        T: Iterator<Item = u8> + Clone,
    {
        let start = bytes.clone();
        let mut str_len = 0;
        while let Some(b) = bytes.peek() {
            if !is_whitespace(*b) && *b != b';' {
                str_len += 1;
                bytes.next();
            } else {
                break;
            }
        }
        String::from_utf8(start.take(str_len).collect()).unwrap()
    }

    fn skip_comment(bytes: &mut Peekable<impl Iterator<Item = u8> + Clone>) {
        while let Some(b) = bytes.peek() {
            if *b == b'\n' {
                break;
            } else {
                bytes.next();
            }
        }
    }

    pub fn tokenize(bytes: Bytes) -> Result<Vec<TokenInfo>, String> {
        let mut bytes = bytes.peekable();
        let mut tokens = Vec::new();
        let mut cur_line = 1;

        while let Some(byte) = bytes.peek() {
            match byte {
                b'"' => tokens.push(TokenInfo::new(
                    cur_line,
                    Token::String(parse_quoted_string(&mut bytes)),
                )),
                b'@' => tokens.push(TokenInfo::new(
                    cur_line,
                    Token::Directive(parse_whitespace_terminated(&mut bytes)),
                )),
                b'$' => tokens.push(TokenInfo::new(
                    cur_line,
                    Token::Variable(parse_whitespace_terminated(&mut bytes)),
                )),
                b'-' => {
                    bytes.next();
                    tokens.push(TokenInfo::new(
                        cur_line,
                        Token::RemoveTag(parse_whitespace_terminated(&mut bytes)),
                    ));
                }
                b'=' => {
                    tokens.push(TokenInfo::new(cur_line, Token::Assignment));
                    bytes.next();
                }
                b'{' => {
                    tokens.push(TokenInfo::new(cur_line, Token::SetOpen));
                    bytes.next();
                }
                b'}' => {
                    tokens.push(TokenInfo::new(cur_line, Token::SetClose));
                    bytes.next();
                }
                b';' => {
                    tokens.push(TokenInfo::new(cur_line, Token::StatementEnd));
                    bytes.next();
                }
                b'#' => skip_comment(&mut bytes),
                b'\n' => {
                    tokens.push(TokenInfo::new(cur_line, Token::LineBreak));
                    bytes.next();
                    cur_line += 1;
                }
                _ if *byte == b':' || is_whitespace(*byte) => {
                    bytes.next();
                }
                _ => tokens.push(TokenInfo::new(
                    cur_line,
                    Token::AddTag(parse_whitespace_terminated(&mut bytes)),
                )),
            }
        }

        Ok(tokens)
    }

    #[cfg(test)]
    mod tokenizer_tests {
        use super::*;

        #[test]
        #[ignore]
        fn test_whitespace() {
            let string = "This is my string test2 is good!".to_string();
            let mut iter = string.bytes().skip("This is my ".len()).peekable();
            let result = parse_whitespace_terminated(&mut iter);
            assert_eq!(&result, "string");
            assert_eq!(*iter.peek().unwrap(), b' ');
        }

        #[test]
        #[ignore]
        fn test_quoted() {
            let string = "This is my \"string\" test2 is good!".to_string();
            let mut iter = string.bytes().skip("This is my ".len()).peekable();
            let result = parse_quoted_string(&mut iter);
            assert_eq!(&result, "string");
            assert_eq!(*iter.peek().unwrap(), b' ');
        }
    }
}
