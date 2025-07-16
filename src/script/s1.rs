use std::{iter::{self, Peekable}, str::Chars};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Base {
    B10,
    B2,
    B8,
    B16,
}

#[derive(Clone, PartialEq, Eq)]
pub enum S1T {
    Identifier(String),
    KeywordLet, // let
    KeywordIf, // if
    True, // true
    False, // false
    KeywordElse, // else
    KeywordUnless, // unless
    OpenBrace, // {
    CloseBrace, // }
    KeywordOut, // out
    Label(String),
    IntLiteral(String, Base),
    FloatLiteral(String, Base),
    String(String),
    KeywordTry, // try
    KeywordCatch, // catch
    KeywordReturn, // return
    KeywordThrow, // throw
    KeywordFinally, // finally
    KeywordLoop, // loop
    KeywordWhile, // while
    KeywordUntil, // until
    KeywordDefer, // defer
    KeywordBind, // bind
    Plus, // +
    Minus, // -
    Times, // *
    Divide, // /
    Modulus, // %
    Question, // ?
    Equals, // =
    Not, // ! ~
    Less, // <
    Greater, // >
    And, // &
    Or, // |
    Xor, // ^
    Semicolon, // ;
    At, // @
    OpenParen, // (
    CloseParen, // )
    OpenBrack, // [
    CloseBrack, // ]
    Dot, // .
    Colon, // :
    Comma, // ,
    KeywordGoto, // goto
    KeywordFn, // fn
    KeywordNull, // null
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum InsideComment {
    No,
    Singleline, // //
    Multiline, // /**/
}

pub fn s1(contents: String) -> Result<Vec<S1T>, String> {
    let mut iterr = contents.chars().peekable();
    let mut res: Vec<S1T> = Vec::new();
    let invalid_idents = "{}+-*/%?=!~<>&|^;@()[].:,\"\'";
    let mut inside_comment: InsideComment = InsideComment::No;
    while let Some(ch) = iterr.next() {
        match ch {
            ch if inside_comment == InsideComment::Singleline => {
                if ch == '\n' {
                    inside_comment = InsideComment::No;
                }
                continue;
            },
            ch if inside_comment == InsideComment::Multiline => {
                if ch == '*' {
                    if let Some(_) = iterr.next_if(|s| *s == '/') {
                        inside_comment = InsideComment::No;
                    }
                }
                continue;
            },
            ch if ch.is_whitespace() => {},
            ';' => res.push(S1T::Semicolon),
            '=' => res.push(S1T::Equals),
            '0' => {
                if let Some(s) = iterr.next_if(|s| {
                    "xbo".chars().any(|ch| *s == ch)
                }) {
                    let base = match s {
                        'x' => Base::B16,
                        'b' => Base::B2,
                        'o' => Base::B8,
                        _ => Base::B10, // should never happen
                    };
                    if let Some(ch2) = iterr.next() {
                        match base {
                            Base::B16 => {
                                if !"0123456789abcdef".chars().any(|chh| {
                                    ch2.to_lowercase().to_string().chars().nth(0).unwrap() == chh
                                }) {
                                    return Err("invalid number".to_string())
                                }
                            },
                            Base::B2 => {
                                if !"01".chars().any(|chh| {
                                    chh == ch2
                                }) {
                                    return Err("invalid number".to_string())
                                }
                            },
                            Base::B8 => {
                                if !"01234567".chars().any(|chh| {
                                    chh == ch2
                                }) {
                                    return Err("invalid number".to_string())
                                }
                            },
                            Base::B10 => {}, // should never happen
                        }
                        res.push(parse_number(&mut iterr, base, ch2));
                        continue;
                    }
                    return Err("invalid number".to_string())
                }
                res.push(parse_number(&mut iterr, Base::B10, ch));
            },
            '1'..='9' => res.push(parse_number(&mut iterr, Base::B10, ch)),
            '.' => res.push(S1T::Dot),
            '(' => res.push(S1T::OpenParen),
            ')' => res.push(S1T::CloseParen),
            ':' => res.push(S1T::Colon),
            '{' => res.push(S1T::OpenBrace),
            '}' => res.push(S1T::CloseBrace),
            '!' | '~' => res.push(S1T::Not),
            '%' => res.push(S1T::Modulus),
            '+' => res.push(S1T::Plus),
            '-' => res.push(S1T::Minus),
            '*' => res.push(S1T::Times),
            '/' => {
                if let Some(s) = iterr.next_if(|s| {
                    *s == '/' || *s == '*'
                }) {
                    if s == '/' {
                        inside_comment = InsideComment::Singleline;
                    } else {
                        inside_comment = InsideComment::Multiline;
                    }
                    continue;
                }
                res.push(S1T::Divide);
            },
            '<' => res.push(S1T::Less),
            '>' => res.push(S1T::Greater),
            '&' => res.push(S1T::And),
            '|' => res.push(S1T::Or),
            '^' => res.push(S1T::Xor),
            '#' => {
                let str = from_fn_variable_len(|| {
                    iterr.next_if(|s| {
                        if s.is_whitespace() {
                            return false
                        }
                        !invalid_idents.chars().any(|chh| chh == *s)
                    })
                }).iter().collect::<String>();
                res.push(S1T::Label(str));
            },
            '\"' => {
                let mut str = String::new();
                loop {
                    if let Some(chh) = parse_char(&mut iterr, true)? {
                        str.push(chh);
                    } else {
                        break;
                    }
                }
                res.push(S1T::String(str));
            },
            '\'' => {
                let mut str = String::new();
                loop {
                    if let Some(chh) = parse_char(&mut iterr, false)? {
                        str.push(chh);
                    } else {
                        break;
                    }
                }
                res.push(S1T::String(str));
            },
            '?' => res.push(S1T::Question),
            '[' => res.push(S1T::OpenBrack),
            ']' => res.push(S1T::CloseBrack),
            '@' => res.push(S1T::At),
            ',' => res.push(S1T::Comma),
            _ => {
                let str = iter::once(ch)
                    .chain(from_fn_variable_len(|| {
                        iterr.next_if(|s| {
                            if s.is_whitespace() {
                                return false;
                            }
                            !invalid_idents.chars().any(|chh| *s == chh)
                        })
                    }))
                    .collect::<String>();
                if str == "elif" || str == "elsif" {
                    res.push(S1T::KeywordElse);
                    res.push(S1T::KeywordIf);
                    continue;
                }
                res.push(match str.as_str() {
                    "let" => S1T::KeywordLet,
                    "if" => S1T::KeywordIf,
                    "true" => S1T::True,
                    "false" => S1T::False,
                    "else" => S1T::KeywordElse,
                    "unless" => S1T::KeywordUnless,
                    "out" => S1T::KeywordOut,
                    "try" => S1T::KeywordTry,
                    "catch" => S1T::KeywordCatch,
                    "return" => S1T::KeywordReturn,
                    "throw" => S1T::KeywordThrow,
                    "finally" => S1T::KeywordFinally,
                    "loop" => S1T::KeywordLoop,
                    "while" => S1T::KeywordWhile,
                    "until" => S1T::KeywordUntil,
                    "defer" => S1T::KeywordDefer,
                    "bind" => S1T::KeywordBind,
                    "goto" => S1T::KeywordGoto,
                    "fn" | "func" | "function" => S1T::KeywordFn,
                    "null" => S1T::KeywordNull,
                    _ => S1T::Identifier(str)
                });
            }
        }
    }
    Ok(res)
}

// don't touch this!
pub fn from_fn_variable_len<T, F>(mut cb: F) -> Vec<T> where F: FnMut() -> Option<T> {
    let mut vec = Vec::new();
    while let Some(i) = cb() {
        vec.push(i);
    }
    vec
}

fn parse_number(iterr: &mut Peekable<Chars<'_>>, base: Base, ch: char) -> S1T {
    let chars = match base {
        Base::B10 => "0123456789",
        Base::B2 => "01",
        Base::B8 => "01234567",
        Base::B16 => "0123456789abcdef",
    };
    let mut dot = false;
    let n = iter::once(ch)
        .chain(from_fn_variable_len(|| {
            iterr.next_if(|s| {
                if *s == '.' && !dot {
                    dot = true;
                    return true;
                }
                if *s == '_' {
                    return true;
                }
                chars.chars().any(|chh| chh == s.to_lowercase().to_string().chars().nth(0).unwrap())
            })
        }))
        .filter(|s| *s != '_')
        .collect::<String>().to_lowercase();
    if dot {
        S1T::FloatLiteral(n, base)
    } else {
        S1T::IntLiteral(n, base)
    }
}

pub fn parse_char(iterr: &mut Peekable<Chars<'_>>, doublequotes: bool) -> Result<Option<char>, String> {
    let ch = iterr.next();
    if let Some(ch) = ch {
        match ch {
            '\"' => {
                if doublequotes {
                    return Ok(None);
                }
                return Ok(Some(ch))
            },
            '\'' => {
                if !doublequotes {
                    return Ok(None);
                }
                return Ok(Some(ch));
            },
            '\\' => {
                let ch2 = iterr.next();
                if let Some(ch2) = ch2 {
                    match ch2 {
                        '\\' => return Ok(Some('\\')),
                        '\'' => return Ok(Some('\'')),
                        '\"' => return Ok(Some('\"')),
                        '0' => return Ok(Some('\0')),
                        't' => return Ok(Some('\t')),
                        'r' => return Ok(Some('\r')),
                        'n' => return Ok(Some('\n')),
                        _ => return Err("invalid escape sequence".to_string())
                    }
                }
                return Err("unfinished string literal".to_string())
            },
            _ => return Ok(Some(ch)),
        }
    }
    Err("unfinished string literal".to_string())
}