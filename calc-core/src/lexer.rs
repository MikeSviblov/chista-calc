use crate::error::CalcError;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Int(i128),
    Float(f64),
    Str(String),
    Ident(String),
    True,
    False,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    EqEq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    AndAnd,
    OrOr,
    Bang,
    Eq,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Semicolon,
    Newline,
    KwFn,
    KwAlias,
    KwWhile,
    KwRepeat,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub pos: usize,
}

pub fn tokenize(src: &str) -> crate::error::Result<Vec<Token>> {
    let chars: Vec<char> = src.chars().collect();
    let mut i = 0usize;
    let mut tokens = Vec::new();

    while i < chars.len() {
        let start = i;
        let c = chars[i];

        if c == ' ' || c == '\t' || c == '\r' {
            i += 1;
            continue;
        }

        if c == '\n' {
            tokens.push(Token { kind: TokenKind::Newline, pos: start });
            i += 1;
            continue;
        }

        if c == '#' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        if c.is_ascii_digit() {
            let (kind, next) = lex_number(&chars, i);
            tokens.push(Token { kind, pos: start });
            i = next;
            continue;
        }

        if c == '"' {
            let (s, next) = lex_string(&chars, i)?;
            tokens.push(Token { kind: TokenKind::Str(s), pos: start });
            i = next;
            continue;
        }

        if c.is_alphabetic() || c == '_' {
            let mut j = i + 1;
            while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_') {
                j += 1;
            }
            let word: String = chars[i..j].iter().collect();
            let kind = match word.as_str() {
                "fn" => TokenKind::KwFn,
                "alias" => TokenKind::KwAlias,
                "while" => TokenKind::KwWhile,
                "repeat" => TokenKind::KwRepeat,
                "true" => TokenKind::True,
                "false" => TokenKind::False,
                _ => TokenKind::Ident(word),
            };
            tokens.push(Token { kind, pos: start });
            i = j;
            continue;
        }

        // two-char operators
        if i + 1 < chars.len() {
            let two: String = chars[i..i + 2].iter().collect();
            let two_kind = match two.as_str() {
                "==" => Some(TokenKind::EqEq),
                "!=" => Some(TokenKind::Ne),
                "<=" => Some(TokenKind::Le),
                ">=" => Some(TokenKind::Ge),
                "&&" => Some(TokenKind::AndAnd),
                "||" => Some(TokenKind::OrOr),
                _ => None,
            };
            if let Some(kind) = two_kind {
                tokens.push(Token { kind, pos: start });
                i += 2;
                continue;
            }
        }

        let single_kind = match c {
            '+' => Some(TokenKind::Plus),
            '-' => Some(TokenKind::Minus),
            '*' => Some(TokenKind::Star),
            '/' => Some(TokenKind::Slash),
            '%' => Some(TokenKind::Percent),
            '^' => Some(TokenKind::Caret),
            '=' => Some(TokenKind::Eq),
            '(' => Some(TokenKind::LParen),
            ')' => Some(TokenKind::RParen),
            '{' => Some(TokenKind::LBrace),
            '}' => Some(TokenKind::RBrace),
            ',' => Some(TokenKind::Comma),
            ';' => Some(TokenKind::Semicolon),
            '<' => Some(TokenKind::Lt),
            '>' => Some(TokenKind::Gt),
            '!' => Some(TokenKind::Bang),
            _ => None,
        };

        if let Some(kind) = single_kind {
            tokens.push(Token { kind, pos: start });
            i += 1;
            continue;
        }

        return Err(CalcError::SyntaxError {
            msg: format!("Неизвестный символ '{c}'"),
            pos: start,
        });
    }

    tokens.push(Token { kind: TokenKind::Eof, pos: chars.len() });
    Ok(tokens)
}

fn lex_number(chars: &[char], start: usize) -> (TokenKind, usize) {
    if chars[start] == '0' && start + 1 < chars.len() {
        let base_char = chars[start + 1];
        let (radix, is_base_prefix) = match base_char {
            'x' | 'X' => (16, true),
            'b' | 'B' => (2, true),
            'o' | 'O' => (8, true),
            _ => (10, false),
        };
        if is_base_prefix {
            let mut j = start + 2;
            while j < chars.len() && chars[j].is_digit(radix) {
                j += 1;
            }
            let digits: String = chars[start + 2..j].iter().collect();
            let value = i128::from_str_radix(&digits, radix).unwrap();
            return (TokenKind::Int(value), j);
        }
    }

    let mut j = start;
    while j < chars.len() && chars[j].is_ascii_digit() {
        j += 1;
    }

    let mut is_float = false;

    if j < chars.len() && chars[j] == '.' && j + 1 < chars.len() && chars[j + 1].is_ascii_digit() {
        is_float = true;
        j += 1;
        while j < chars.len() && chars[j].is_ascii_digit() {
            j += 1;
        }
    }

    if j < chars.len() && (chars[j] == 'e' || chars[j] == 'E') {
        let mut k = j + 1;
        if k < chars.len() && (chars[k] == '+' || chars[k] == '-') {
            k += 1;
        }
        if k < chars.len() && chars[k].is_ascii_digit() {
            is_float = true;
            k += 1;
            while k < chars.len() && chars[k].is_ascii_digit() {
                k += 1;
            }
            j = k;
        }
    }

    let text: String = chars[start..j].iter().collect();
    if is_float {
        (TokenKind::Float(text.parse::<f64>().unwrap()), j)
    } else {
        (TokenKind::Int(text.parse::<i128>().unwrap()), j)
    }
}

fn lex_string(chars: &[char], start: usize) -> crate::error::Result<(String, usize)> {
    let mut i = start + 1;
    let mut s = String::new();
    loop {
        if i >= chars.len() {
            return Err(CalcError::SyntaxError {
                msg: "Незакрытая строка".into(),
                pos: start,
            });
        }
        match chars[i] {
            '"' => {
                i += 1;
                break;
            }
            '\\' => {
                i += 1;
                if i >= chars.len() {
                    return Err(CalcError::SyntaxError {
                        msg: "Незакрытая строка".into(),
                        pos: start,
                    });
                }
                let escaped = match chars[i] {
                    'n' => '\n',
                    't' => '\t',
                    '"' => '"',
                    '\\' => '\\',
                    other => other,
                };
                s.push(escaped);
                i += 1;
            }
            c => {
                s.push(c);
                i += 1;
            }
        }
    }
    Ok((s, i))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn kinds(src: &str) -> Vec<TokenKind> {
        tokenize(src).unwrap().into_iter().map(|t| t.kind).collect()
    }
    #[test]
    fn numbers_in_all_bases() {
        assert_eq!(kinds("10"), vec![TokenKind::Int(10), TokenKind::Eof]);
        assert_eq!(kinds("0x1F"), vec![TokenKind::Int(31), TokenKind::Eof]);
        assert_eq!(kinds("0b1010"), vec![TokenKind::Int(10), TokenKind::Eof]);
        assert_eq!(kinds("0o17"), vec![TokenKind::Int(15), TokenKind::Eof]);
        assert_eq!(kinds("1.5"), vec![TokenKind::Float(1.5), TokenKind::Eof]);
        assert_eq!(kinds("1e3"), vec![TokenKind::Float(1000.0), TokenKind::Eof]);
    }
    #[test]
    fn string_with_escapes() {
        assert_eq!(kinds("\"a\\nb\""), vec![TokenKind::Str("a\nb".into()), TokenKind::Eof]);
    }
    #[test]
    fn operators_idents_comments() {
        assert_eq!(kinds("x = f(1) # hi"), vec![
            TokenKind::Ident("x".into()), TokenKind::Eq, TokenKind::Ident("f".into()),
            TokenKind::LParen, TokenKind::Int(1), TokenKind::RParen, TokenKind::Eof,
        ]);
    }
    #[test]
    fn multi_char_operators() {
        assert_eq!(kinds("=="), vec![TokenKind::EqEq, TokenKind::Eof]);
        assert_eq!(kinds("&&"), vec![TokenKind::AndAnd, TokenKind::Eof]);
        assert_eq!(kinds("<="), vec![TokenKind::Le, TokenKind::Eof]);
    }
    #[test]
    fn newline_is_emitted() {
        assert_eq!(kinds("1\n2"), vec![TokenKind::Int(1), TokenKind::Newline, TokenKind::Int(2), TokenKind::Eof]);
    }
}
