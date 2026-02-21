use crate::const_assert;

const TOKEN_TYPE_COUNT: i32 = 46;
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TokenType {
    Unknown,
    Plus,
    Sub,
    Star,
    Slash,
    Percent,
    SemiColon,
    Arrow,
    Colon,
    DoubleColon,
    Ampersand,
    DoubleQuote,
    SingleQuote,
    Dot,
    Hat,
    Comma,
    Or,
    LogicalOr,
    Equal,
    NotEqual,
    Lt,
    Lte,
    ShortAssign,
    Assign,
    Gt,
    Gte,
    Not,
    OpenParen,
    CloseParen,
    OpenCurly,
    CloseCurly,
    OpenSquare,
    CloseSquare,
    Type,
    Struct,
    Import,
    Defer,
    For,
    In,
    If,
    Else,
    Proc,
    Return,
    Include,
    Word,
    Number,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub x: i32,
    pub y: i32,
    pub tok: String,
    pub tt: TokenType,
}

pub fn is_operator(tt: &TokenType) -> bool {
    match tt {
        TokenType::Plus  => true,
        TokenType::Sub   => true,
        TokenType::Star  => true,
        TokenType::Slash => true,
        TokenType::Assign => true,
        _ => false
    }
}

fn is_whitespace(c: char) -> bool {
    match c {
        ' ' | '\t' | '\n' => true,
        _                 => false,
    }
}

const_assert!(TOKEN_TYPE_COUNT == 46, "Update: TOKEN_TYPE_COUNT has changed");
fn is_delim(c: char) -> bool {
    match c {
        '+' => true,
        '-' => true,
        '*' => true,
        '^' => true,
        '/' => true,
        '%' => true,
        // ' ' => true,
        ';' => true,
        '"' => true,
        ',' => true,
        '!' => true,
        '(' => true,
        ')' => true,
        ':' => true,
        '=' => true,
        '<' => true,
        '>' => true,
        '&' => true,
        '[' => true,
        ']' => true,
        '{' => true,
        '}' => true,
        '|' => true,
        '\'' => true,
        // '\n' => true,
        '.' => true,
        _   => false
    }
}

fn get_delim_tt(c: char) -> TokenType {
    match c {
        '+' => TokenType::Plus,
        '-' => TokenType::Sub,
        '*' => TokenType::Star,
        '^' => TokenType::Hat,
        '/' => TokenType::Slash,
        '%' => TokenType::Percent,
        ';' => TokenType::SemiColon,
        '"' => TokenType::DoubleQuote,
        ',' => TokenType::Comma,
        '!' => TokenType::Not,
        '(' => TokenType::OpenParen,
        ')' => TokenType::CloseParen,
        ':' => TokenType::Colon,
        '=' => TokenType::Assign,
        '<' => TokenType::Lt,
        '>' => TokenType::Gt,
        '&' => TokenType::Ampersand,
        '[' => TokenType::OpenSquare,
        ']' => TokenType::CloseSquare,
        '{' => TokenType::OpenCurly,
        '}' => TokenType::CloseCurly,
        '|' => TokenType::Or,
        '\'' =>TokenType::SingleQuote,
        '.' => TokenType::Dot,
        _   => TokenType::Unknown
    }
}

const_assert!(TOKEN_TYPE_COUNT == 46, "Update: TOKEN_TYPE_COUNT has changed");
fn get_keyword_tt(str: &str) -> TokenType {
    match str {
        "for"     => TokenType::For,
        "if"      => TokenType::If,
        "else"    => TokenType::Else,
        "proc"    => TokenType::Proc,
        "return"  => TokenType::Return,
        "in"      => TokenType::In,
        "include" => TokenType::Include,
        "defer" => TokenType::Defer,
        "type" => TokenType::Type,
        "struct" => TokenType::Struct,
        "import" => TokenType::Import,
        _         => TokenType::Unknown
    }
}

const_assert!(TOKEN_TYPE_COUNT == 46, "Update: TOKEN_TYPE_COUNT has changed");
fn is_double_delim(c: char, p: char) -> bool {
    if c == ':' && p == '=' {
        true
    } else if c == ':' && p == ':' {
        true
    } else if c == '=' && p == '=' {
        true
    } else if c == '-' && p == '>' {
        true
    } else if c == '<' && p == '=' {
        true
    } else if c == '>' && p == '=' {
        true
    } else if c == '!' && p == '=' {
        true
    } else if c == '|' && p == '|' {
        true
    } else {
        false
    }
}

const_assert!(TOKEN_TYPE_COUNT == 46, "Update: TOKEN_TYPE_COUNT has changed");
fn get_double_delim_tt(c: char, p: char) -> TokenType {
    if c == ':' && p == '=' {
        TokenType::ShortAssign
    } else if c == ':' && p == ':' {
        TokenType::DoubleColon
    } else if c == '=' && p == '=' {
        TokenType::Equal
    } else if c == '-' && p == '>' {
        TokenType::Arrow
    } else if c == '<' && p == '=' {
        TokenType::Lte
    } else if c == '>' && p == '=' {
        TokenType::Gte
    } else if c == '!' && p == '=' {
        TokenType::NotEqual
    } else if c == '|' && p == '|' {
        TokenType::LogicalOr
    } else {
        TokenType::Unknown
    }
}

fn is_number(tok: &str) -> bool {
    match tok.parse::<i64>() {
        Ok(_) => true,
        Err(_) => false
    }
}

fn is_valid_unicode(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes).is_ok()
}

fn get_tt(tok: &str) -> TokenType {
    if tok.len() == 1 {
        let tt = get_delim_tt(tok.chars().nth(0).unwrap());
        if tt != TokenType::Unknown {
            return tt;
        }
    } else if tok.len() == 2 {
        let c = tok.chars().nth(0).unwrap();
        let p = tok.chars().nth(1).unwrap();
        let tt = get_double_delim_tt(c, p);
        if tt != TokenType::Unknown {
            return tt;
        }
    }

    if is_number(tok) {
        return TokenType::Number;
    } else if is_valid_unicode(tok.as_bytes()) {
        let tt = get_keyword_tt(tok);
        if tt != TokenType::Unknown {
            return tt;        
        } else {
            return TokenType::Word;
        }
    } else {
        return TokenType::Unknown;
    }
}

fn create_token(x: i32, y: i32, tok: String) -> Token {
    Token {
        x,
        y,
        tok: tok.clone(),
        tt: get_tt(&tok)
    }
}

pub fn tokenize_start(src: &str) -> Vec<Token> {
    let (mut x, mut y) = (1, 1);
    let mut ret = Vec::<Token>::new();
    let mut iterator = src.chars().enumerate().peekable();
    let mut current_string = String::new();
    while let Some((_, c)) = iterator.next() {
        let (_, p) = match iterator.peek() {
            Some(p) => p,
            None => {
                if current_string.len() > 0 {
                    let token = create_token(x, y, current_string.clone());
                    ret.push(token);
                    current_string.clear();
                }
                if is_delim(c) {
                    let token = create_token(x, y, c.to_string());
                    ret.push(token);
                }
                break;
            },
        };
        if c == '\n' {
            x = 1;
            y += 1;
        }
        
        if is_whitespace(c) {
            // save token
            if current_string.len() > 0 {
                let token = create_token(x, y, current_string.clone());
                ret.push(token);
                current_string.clear();
            }
        } else if is_delim(c) {
            if current_string.len() > 0 {
                // save the token first
                let token = create_token(x, y, current_string.clone());
                ret.push(token);
                current_string.clear();
            }

            // save the delim as a token
            if is_double_delim(c, *p){
                x += 1;
                let mut tok = String::new();
                tok.push(c);
                tok.push(*p);
                let token = create_token(x, y, tok);
                ret.push(token);
                iterator.next();
            } else if c == '"' {
                let mut c = iterator.next().unwrap().1;
                let mut tok = String::new();
                while c != '"' {
                    tok.push(c);
                    c = iterator.next().unwrap().1;
                }
                ret.push(Token{x, y, tok, tt: TokenType::DoubleQuote});
            } else {
                let token = create_token(x, y, c.to_string());
                ret.push(token);
            }
        } else {
            current_string.push(c);
            x += 1;
        }
    }
    return ret;
}

pub fn print_tokens(tokens: &Vec<Token>) {
    for token in tokens {
        println!("Token: {}[{:?}]({}:{})",token.tok, token.tt, token.x, token.y);
    }
}
