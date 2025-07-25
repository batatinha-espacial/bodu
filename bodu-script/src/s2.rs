use std::{iter::Peekable, slice::Iter};

use crate::s1::{Base, S1T};

#[derive(Clone, PartialEq, Debug)]
pub enum S2T {
    Identifier(String),
    Let, // let
    If, // if
    True, // true
    False, // false
    Else, // else
    ElseIf, // else if
    Unless, // unless
    ElseUnless, // else unless
    OpenBrace, // {
    CloseBrace, // }
    Out, // out
    Label(String),
    Int(i64),
    Float(f64),
    String(String),
    Try, // try
    Catch, // catch
    Return, // return
    Throw, // throw
    Loop, // loop
    While, // while
    Until, // until
    Defer, // defer
    Bind, // bind
    Plus, // +
    PlusAssign, // +=
    PlusPlus, // ++
    Minus, // -
    MinusAssign, // -=
    MinusMinus, // --
    Times, // *
    TimesAssign, // *=
    Divide, // /
    DivideAssign, // /=
    Modulus, // %
    ModulusAssign, // %=
    Question, // ?
    OrThat, // ??
    Assign, // =
    EqualTo, // ==
    Not, // !
    NotEqualTo, // !=
    Less, // <
    LessOrEqual, // <=
    Greater, // >
    GreaterOrEqual, // >=
    And, // &
    Or, // |
    Pipe, // |>
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
    Goto, // goto
    Fn, // fn
    PlusFn, // [+]
    MinusFn, // [-]
    TimesFn, // [*]
    DivideFn, // [/]
    ModulusFn, // [%]
    OrThatFn, // [??]
    TernaryFn, // [?:]
    EqualToFn, // [==]
    NotFn, // [!]
    NotEqualToFn, // [!=]
    LessFn, // [<]
    LessOrEqualFn, // [<=]
    GreaterFn, // [>]
    GreaterOrEqualFn, // [>=]
    AndFn, // [&]
    OrFn, // [|]
    XorFn, // [^]
    PropertyFn, // [.]
    TupleFn, // [,]
    PipeFn, // [|>]
    Null, // null
    PipeShorthand, // $
    Debug, // debug
    Release, // release
    Maybe, // maybe
    For, // for
    In, // in
    Before, // before
    Again, // again
    After, // after
    Continue, // continue
    Break, // break
    Probably, // probably
    Possibly, // possibly
    IsntNullFn, // [?]
}

pub fn s2(s1: Vec<S1T>) -> Result<Vec<S2T>, String> {
    let mut res = Vec::new();
    let mut iterr = s1.iter().peekable();
    while let Some(i) = iterr.next() {
        match i {
            S1T::Identifier(s) => res.push(S2T::Identifier(s.clone())),
            S1T::KeywordLet => res.push(S2T::Let),
            S1T::KeywordIf => res.push(S2T::If),
            S1T::True => res.push(S2T::True),
            S1T::False => res.push(S2T::False),
            S1T::KeywordElse => {
                if let Some(s) = iterr.next_if(|t| {
                    match t {
                        S1T::KeywordIf => true,
                        S1T::KeywordUnless => true,
                        _ => false,
                    }
                }) {
                    let t = match s {
                        S1T::KeywordIf => S2T::ElseIf,
                        S1T::KeywordUnless => S2T::ElseUnless,
                        _ => S2T::Else, // should never happen
                    };
                    res.push(t);
                    continue;
                }
                res.push(S2T::Else);
            },
            S1T::KeywordUnless => res.push(S2T::Unless),
            S1T::OpenBrace => res.push(S2T::OpenBrace),
            S1T::CloseBrace => res.push(S2T::CloseBrace),
            S1T::KeywordOut => res.push(S2T::Out),
            S1T::Label(s) => res.push(S2T::Label(s.clone())),
            S1T::IntLiteral(s, b) => res.push(S2T::Int(parse_int(s.clone(), *b))),
            S1T::FloatLiteral(s, b) => res.push(S2T::Float(parse_float(s.clone(), *b))),
            S1T::String(s) => res.push(S2T::String(s.clone())),
            S1T::KeywordTry => res.push(S2T::Try),
            S1T::KeywordCatch => res.push(S2T::Catch),
            S1T::KeywordReturn => res.push(S2T::Return),
            S1T::KeywordThrow => res.push(S2T::Throw),
            S1T::KeywordLoop => res.push(S2T::Loop),
            S1T::KeywordWhile => res.push(S2T::While),
            S1T::KeywordUntil => res.push(S2T::Until),
            S1T::KeywordDefer => res.push(S2T::Defer),
            S1T::KeywordBind => res.push(S2T::Bind),
            S1T::Plus => {
                if let Some(s) = iterr.next_if(|t| {
                    match t {
                        S1T::Equals => true,
                        S1T::Plus => true,
                        _ => false,
                    }
                }) {
                    let t = match s {
                        S1T::Equals => S2T::PlusAssign,
                        S1T::Plus => S2T::PlusPlus,
                        _ => S2T::Plus, // should never happen
                    };
                    res.push(t);
                    continue;
                }
                res.push(S2T::Plus);
            },
            S1T::Minus => {
                if let Some(s) = iterr.next_if(|t| {
                    match t {
                        S1T::Equals => true,
                        S1T::Minus => true,
                        _ => false,
                    }
                }) {
                    let t = match s {
                        S1T::Equals => S2T::MinusAssign,
                        S1T::Minus => S2T::MinusMinus,
                        _ => S2T::Minus, // should never happen
                    };
                    res.push(t);
                    continue;
                }
                res.push(S2T::Minus);
            },
            S1T::Times => {
                if let Some(_) = iterr.next_if(|t| {
                    match t {
                        S1T::Equals => true,
                        _ => false,
                    }
                }) {
                    res.push(S2T::TimesAssign);
                    continue;
                }
                res.push(S2T::Times);
            },
            S1T::Divide => {
                if let Some(_) = iterr.next_if(|t| {
                    match t {
                        S1T::Equals => true,
                        _ => false,
                    }
                }) {
                    res.push(S2T::DivideAssign);
                    continue;
                }
                res.push(S2T::Divide);
            },
            S1T::Modulus => {
                if let Some(_) = iterr.next_if(|t| {
                    match t {
                        S1T::Equals => true,
                        _ => false,
                    }
                }) {
                    res.push(S2T::ModulusAssign);
                    continue;
                }
                res.push(S2T::Modulus);
            },
            S1T::Question => {
                if let Some(_) = iterr.next_if(|t| {
                    match t {
                        S1T::Question => true,
                        _ => false,
                    }
                }) {
                    res.push(S2T::OrThat);
                    continue;
                }
                res.push(S2T::Question);
            },
            S1T::Equals => {
                if let Some(_) = iterr.next_if(|t| {
                    match t {
                        S1T::Equals => true,
                        _ => false,
                    }
                }) {
                    res.push(S2T::EqualTo);
                    continue;
                }
                res.push(S2T::Assign);
            },
            S1T::Not => {
                if let Some(_) = iterr.next_if(|t| {
                    match t {
                        S1T::Equals => true,
                        _ => false,
                    }
                }) {
                    res.push(S2T::NotEqualTo);
                    continue;
                }
                res.push(S2T::Not);
            },
            S1T::Less => {
                if let Some(_) = iterr.next_if(|t| {
                    match t {
                        S1T::Equals => true,
                        _ => false,
                    }
                }) {
                    res.push(S2T::LessOrEqual);
                    continue;
                }
                res.push(S2T::Less);
            },
            S1T::Greater => {
                if let Some(_) = iterr.next_if(|t| {
                    match t {
                        S1T::Equals => true,
                        _ => false,
                    }
                }) {
                    res.push(S2T::GreaterOrEqual);
                    continue;
                }
                res.push(S2T::Greater);
            },
            S1T::And => {
                if let Some(_) = iterr.next_if(|t| {
                    match t {
                        S1T::And => true,
                        _ => false,
                    }
                }) {
                    res.push(S2T::And);
                    continue;
                }
                res.push(S2T::And);
            },
            S1T::Or => {
                if let Some(s) = iterr.next_if(|t| {
                    match t {
                        S1T::Or => true,
                        S1T::Greater => true,
                        _ => false,
                    }
                }) {
                    let t = match s {
                        S1T::Or => S2T::Or,
                        S1T::Greater => S2T::Pipe,
                        _ => S2T::Or, // shoul never happen
                    };
                    res.push(t);
                    continue;
                }
                res.push(S2T::Or);
            },
            S1T::Xor => {
                if let Some(_) = iterr.next_if(|t| {
                    match t {
                        S1T::Xor => true,
                        _ => false,
                    }
                }) {
                    res.push(S2T::Xor);
                    continue;
                }
                res.push(S2T::Xor);
            },
            S1T::Semicolon => {
                loop {
                    if let None = iterr.next_if(|t| {
                        match t {
                            S1T::Semicolon => true,
                            _ => false
                        }
                    }) {
                        break;
                    }
                }
                res.push(S2T::Semicolon);
            },
            S1T::At => res.push(S2T::At),
            S1T::OpenParen => res.push(S2T::OpenParen),
            S1T::CloseParen => res.push(S2T::CloseParen),
            S1T::OpenBrack => res.push(S2T::OpenBrack),
            S1T::CloseBrack => res.push(S2T::CloseBrack),
            S1T::Dot => res.push(S2T::Dot),
            S1T::Colon => res.push(S2T::Colon),
            S1T::Comma => res.push(S2T::Comma),
            S1T::KeywordGoto => res.push(S2T::Goto),
            S1T::KeywordFn => res.push(S2T::Fn),
            S1T::KeywordNull => res.push(S2T::Null),
            S1T::Dollar => res.push(S2T::PipeShorthand),
            S1T::KeywordDebug => res.push(S2T::Debug),
            S1T::KeywordRelease => res.push(S2T::Release),
            S1T::KeywordMaybe => res.push(S2T::Maybe),
            S1T::KeywordFor => res.push(S2T::For),
            S1T::KeywordIn => res.push(S2T::In),
            S1T::KeywordBefore => res.push(S2T::Before),
            S1T::KeywordAgain => res.push(S2T::Again),
            S1T::KeywordAfter => res.push(S2T::After),
            S1T::KeywordContinue => res.push(S2T::Continue),
            S1T::KeywordBreak => res.push(S2T::Break),
            S1T::KeywordProbably => res.push(S2T::Probably),
            S1T::KeywordPossibly => res.push(S2T::Possibly),
        }
    }
    let s2 = res;
    let mut res = Vec::new();
    let mut iterr = s2.iter().peekable();
    while let Some(i) = iterr.next() {
        match i {
            S2T::OpenBrack => {
                if let Some(s) = iterr.next_if(|t| {
                    match t {
                        S2T::Plus => true,
                        S2T::Minus => true,
                        S2T::Times => true,
                        S2T::Divide => true,
                        S2T::Modulus => true,
                        S2T::Question => true,
                        S2T::OrThat => true,
                        S2T::EqualTo => true,
                        S2T::Not => true,
                        S2T::NotEqualTo => true,
                        S2T::Less => true,
                        S2T::LessOrEqual => true,
                        S2T::Greater => true,
                        S2T::GreaterOrEqual => true,
                        S2T::And => true,
                        S2T::Or => true,
                        S2T::Xor => true,
                        S2T::Dot => true,
                        S2T::Comma => true,
                        S2T::Pipe => true,
                        _ => false,
                    }
                }) {
                    match s {
                        S2T::Plus => {
                            opfn_helper(&mut iterr, S2T::PlusFn, S2T::Plus, &mut res);
                            continue;
                        },
                        S2T::Minus => {
                            opfn_helper(&mut iterr, S2T::MinusFn, S2T::Minus, &mut res);
                            continue;
                        },
                        S2T::Times => {
                            opfn_helper(&mut iterr, S2T::TimesFn, S2T::Times, &mut res);
                            continue;
                        },
                        S2T::Divide => {
                            opfn_helper(&mut iterr, S2T::DivideFn, S2T::Divide, &mut res);
                            continue;
                        },
                        S2T::Modulus => {
                            opfn_helper(&mut iterr, S2T::ModulusFn, S2T::Modulus, &mut res);
                            continue;
                        },
                        S2T::Question => {
                            if let Some(t) = iterr.next_if(|t| {
                                match t {
                                    S2T::Colon => true,
                                    S2T::Question => true,
                                    S2T::CloseBrack => true,
                                    _ => false,
                                }
                            }) {
                                match t {
                                    S2T::Colon => {
                                        if let Some(S2T::CloseBrack) = iterr.peek() {
                                            iterr.next();
                                            res.push(S2T::TernaryFn);
                                            continue;
                                        }
                                        res.push(S2T::OpenBrack);
                                        res.push(S2T::Question);
                                        res.push(S2T::Colon);
                                        continue;
                                    },
                                    S2T::Question => {
                                        if let Some(S2T::CloseBrack) = iterr.peek() {
                                            iterr.next();
                                            res.push(S2T::OrThatFn);
                                            continue;
                                        }
                                        res.push(S2T::OpenBrack);
                                        res.push(S2T::Question);
                                        res.push(S2T::Question);
                                        continue;
                                    },
                                    S2T::CloseBrack => {
                                        res.push(S2T::IsntNullFn);
                                        continue;
                                    },
                                    _ => {}, // should never happen
                                }
                            }
                            res.push(S2T::OpenBrack);
                            res.push(S2T::Question);
                            continue;
                        },
                        S2T::OrThat => {
                            opfn_helper(&mut iterr, S2T::OrThatFn, S2T::OrThat, &mut res);
                            continue;
                        },
                        S2T::EqualTo => {
                            opfn_helper(&mut iterr, S2T::EqualToFn, S2T::EqualTo, &mut res);
                            continue;
                        },
                        S2T::Not => {
                            opfn_helper(&mut iterr, S2T::NotFn, S2T::Not, &mut res);
                            continue;
                        },
                        S2T::NotEqualTo => {
                            opfn_helper(&mut iterr, S2T::NotEqualToFn, S2T::NotEqualTo, &mut res);
                            continue;
                        },
                        S2T::Less => {
                            opfn_helper(&mut iterr, S2T::LessFn, S2T::Less, &mut res);
                            continue;
                        },
                        S2T::LessOrEqual => {
                            opfn_helper(&mut iterr, S2T::LessOrEqualFn, S2T::LessOrEqual, &mut res);
                            continue;
                        },
                        S2T::Greater => {
                            opfn_helper(&mut iterr, S2T::GreaterFn, S2T::Greater, &mut res);
                            continue;
                        },
                        S2T::GreaterOrEqual => {
                            opfn_helper(&mut iterr, S2T::GreaterOrEqualFn, S2T::GreaterOrEqual, &mut res);
                            continue;
                        },
                        S2T::And => {
                            opfn_helper(&mut iterr, S2T::AndFn, S2T::And, &mut res);
                            continue;
                        },
                        S2T::Or => {
                            opfn_helper(&mut iterr, S2T::OrFn, S2T::Or, &mut res);
                            continue;
                        },
                        S2T::Xor => {
                            opfn_helper(&mut iterr, S2T::XorFn, S2T::Xor, &mut res);
                            continue;
                        },
                        S2T::Dot => {
                            opfn_helper(&mut iterr, S2T::PropertyFn, S2T::Dot, &mut res);
                            continue;
                        },
                        S2T::Comma => {
                            opfn_helper(&mut iterr, S2T::TupleFn, S2T::Comma, &mut res);
                            continue;
                        },
                        S2T::Pipe => {
                            opfn_helper(&mut iterr, S2T::PipeFn, S2T::Pipe, &mut res);
                            continue;
                        },
                        _ => {}, // should never happen
                    }
                }
                res.push(S2T::OpenBrack);
            },
            i => res.push(i.clone()),
        }
    }
    Ok(res)
}

fn opfn_helper(iterr: &mut Peekable<Iter<'_, S2T>>, tok: S2T, tokfail: S2T, res: &mut Vec<S2T>) {
    if let Some(_) = iterr.next_if(|t| {
        match t {
            S2T::CloseBrack => true,
            _ => false,
        }
    }) {
        res.push(tok);
        return
    }
    res.push(S2T::OpenBrack);
    res.push(tokfail);
}

fn parse_int(s: String, base: Base) -> i64 {
    match base {
        Base::B10 => i64::from_str_radix(&s, 10).unwrap(),
        Base::B2 => i64::from_str_radix(&s, 2).unwrap(),
        Base::B8 => i64::from_str_radix(&s, 8).unwrap(),
        Base::B16 => i64::from_str_radix(&s, 16).unwrap(),
    }
}

fn parse_float(s: String, base: Base) -> f64 {
    let mut iterr = s.chars();
    let mut r = 0.0;
    let b: f64 = match base {
        Base::B10 => 10.0,
        Base::B2 => 2.0,
        Base::B8 => 8.0,
        Base::B16 => 16.0,
    };
    let mut q: f64 = b;
    let mut dot = false;
    while let Some(ch) = iterr.next() {
        if ch == '.' {
            dot = true;
            continue;
        }
        let cd: f64 = match base {
            Base::B10 => match ch {
                '0' => 0.0,
                '1' => 1.0,
                '2' => 2.0,
                '3' => 3.0,
                '4' => 4.0,
                '5' => 5.0,
                '6' => 6.0,
                '7' => 7.0,
                '8' => 8.0,
                '9' => 9.0,
                _ => 0.0, // should never happen
            },
            Base::B2 => match ch {
                '0' => 0.0,
                '1' => 1.0,
                _ => 0.0, // should never happen
            },
            Base::B8 => match ch {
                '0' => 0.0,
                '1' => 1.0,
                '2' => 2.0,
                '3' => 3.0,
                '4' => 4.0,
                '5' => 5.0,
                '6' => 6.0,
                '7' => 7.0,
                _ => 0.0, // should never happen
            },
            Base::B16 => match ch {
                '0' => 0.0,
                '1' => 1.0,
                '2' => 2.0,
                '3' => 3.0,
                '4' => 4.0,
                '5' => 5.0,
                '6' => 6.0,
                '7' => 7.0,
                '8' => 8.0,
                '9' => 9.0,
                'a' => 10.0,
                'b' => 11.0,
                'c' => 12.0,
                'd' => 13.0,
                'e' => 14.0,
                'f' => 15.0,
                _ => 0.0, // should never happen
            },
        };
        if !dot {
            r = r * b + cd;
            continue;
        }
        r += cd / q;
        q *= b;
    }
    r
}