use crate::script::s2::S2T;

pub use crate::vm::Operator;

#[derive(Clone, PartialEq, Debug)]
pub enum S3T {
    Identifier(String),
    Let(String, Option<Box<S3T>>), // let: name, expr
    If(Vec<(ConditionType, Box<S3T>, Vec<S3T>)>, Option<Vec<S3T>>), // if, unless, else if, else unless, else: ...if/unless, else
    Boolean(bool), // true, false
    Block(Vec<S3T>), // { ...expr/stat }
    Out(Box<S3T>), // out: expr
    Label(String), // #label
    Number(i64),
    Float(f64),
    String(String),
    TryCatchFinally(Vec<S3T>, String, Vec<S3T>, Option<Vec<S3T>>), // try, catch, finally: try { ... } catch name { ... } finally { ... }
    Return(Box<S3T>), // return: expr
    Throw(Box<S3T>), // throw: expr
    Loop(Vec<S3T>), // loop: loop { ... }
    WhileUntil(LoopType, Box<S3T>, Vec<S3T>), // while, until: while/until, condition, { ... }
    Defer(Vec<S3T>), // defer: { ... }
    Bind(String, Box<S3T>), // bind: name, expr
    Function(Option<String>, Vec<String>, Vec<S3T>), // fn: name, ...args, { ... }
    Assign(Box<S3T>, Box<S3T>), // expr = expr
    Plus(Box<S3T>, Box<S3T>), // expr + expr
    Minus(Box<S3T>, Box<S3T>), // expr - expr
    Negate(Box<S3T>), // -expr
    Times(Box<S3T>, Box<S3T>), // expr * expr
    Divide(Box<S3T>, Box<S3T>), // expr / expr
    Modulus(Box<S3T>, Box<S3T>), // expr % expr
    Ternary(Box<S3T>, Box<S3T>, Box<S3T>), // expr ? expr : expr
    EqualTo(Box<S3T>, Box<S3T>), // expr == expr
    Not(Box<S3T>), // !expr
    NotEqualTo(Box<S3T>, Box<S3T>), // expr != expr
    Less(Box<S3T>, Box<S3T>), // expr < expr
    LessOrEqual(Box<S3T>, Box<S3T>), // expr <= expr
    Greater(Box<S3T>, Box<S3T>), // expr > expr
    GreaterOrEqual(Box<S3T>, Box<S3T>), // expr >= expr
    And(Box<S3T>, Box<S3T>), // expr & expr
    Or(Box<S3T>, Box<S3T>), // expr | expr
    Xor(Box<S3T>, Box<S3T>), // expr ^ expr
    Property(Box<S3T>, Box<S3T>), // expr[prop], expr.prop
    Tuple(Vec<S3T>), // expr1, expr2, expr3, ...
    Detuple(Vec<S3T>, Box<S3T>), // (var1, var2, var3, ...) = expr
    LetDetuple(Vec<String>, Box<S3T>), // let (v1, v2, v3, ...) = expr
    FnCall(Box<S3T>, Vec<S3T>), // expr(arg1, arg2, arg3, ...)
    Decorator(Box<S3T>, Box<S3T>), // @expr
    Pipe(Box<S3T>, Box<S3T>), // expr |> f
    Goto(String), // goto #name
    OrThat(Box<S3T>, Box<S3T>), // expr ?? expr
    OperatorFn(Operator), // wrap the operator in brackets
    MultiLet(Vec<String>), // let (v1, v2, v3, ...)
    Null,
    PipeShorthand, // $
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ConditionType {
    If,
    Unless,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LoopType {
    While,
    Until,
}

pub fn s3(input: Vec<S2T>) -> Result<Vec<S3T>, String> {
    let mut i: usize = 0;
    let res = stat_list(&input, &mut i);
    if i < input.len() - 1 {
        return Err("couldn't parse".to_string());
    }
    res.ok_or("couldn't parse".to_string()).map(|v| v.0)
}

fn primary(input: &Vec<S2T>, i: &mut usize) -> Option<(S3T, usize)> {
    match input.get(*i) {
        Some(S2T::Identifier(s)) => {
            *i += 1;
            Some((S3T::Identifier(s.clone()), 1))
        },
        Some(S2T::Int(n)) => {
            *i += 1;
            Some((S3T::Number(*n), 1))
        },
        Some(S2T::Float(f)) => {
            *i += 1;
            Some((S3T::Float(*f), 1))
        },
        Some(S2T::String(s)) => {
            *i += 1;
            Some((S3T::String(s.clone()), 1))
        },
        Some(S2T::True) => {
            *i += 1;
            Some((S3T::Boolean(true), 1))
        },
        Some(S2T::False) => {
            *i += 1;
            Some((S3T::Boolean(false), 1))
        },
        Some(S2T::Null) => {
            *i += 1;
            Some((S3T::Null, 1))
        },
        Some(S2T::PipeShorthand) => {
            *i += 1;
            Some((S3T::PipeShorthand, 1))
        },
        Some(S2T::PlusFn) => {
            *i += 1;
            Some((S3T::OperatorFn(Operator::Plus), 1))
        },
        Some(S2T::MinusFn) => {
            *i += 1;
            Some((S3T::OperatorFn(Operator::Minus), 1))
        },
        Some(S2T::TimesFn) => {
            *i += 1;
            Some((S3T::OperatorFn(Operator::Times), 1))
        },
        Some(S2T::DivideFn) => {
            *i += 1;
            Some((S3T::OperatorFn(Operator::Divide), 1))
        },
        Some(S2T::ModulusFn) => {
            *i += 1;
            Some((S3T::OperatorFn(Operator::Modulus), 1))
        },
        Some(S2T::OrThatFn) => {
            *i += 1;
            Some((S3T::OperatorFn(Operator::OrThat), 1))
        },
        Some(S2T::TernaryFn) => {
            *i += 1;
            Some((S3T::OperatorFn(Operator::Ternary), 1))
        },
        Some(S2T::EqualToFn) => {
            *i += 1;
            Some((S3T::OperatorFn(Operator::EqualTo), 1))
        },
        Some(S2T::NotFn) => {
            *i += 1;
            Some((S3T::OperatorFn(Operator::Not), 1))
        },
        Some(S2T::NotEqualToFn) => {
            *i += 1;
            Some((S3T::OperatorFn(Operator::NotEqualTo), 1))
        },
        Some(S2T::LessFn) => {
            *i += 1;
            Some((S3T::OperatorFn(Operator::Less), 1))
        },
        Some(S2T::LessOrEqualFn) => {
            *i += 1;
            Some((S3T::OperatorFn(Operator::LessOrEqual), 1))
        },
        Some(S2T::GreaterFn) => {
            *i += 1;
            Some((S3T::OperatorFn(Operator::Greater), 1))
        },
        Some(S2T::GreaterOrEqualFn) => {
            *i += 1;
            Some((S3T::OperatorFn(Operator::GreaterOrEqual), 1))
        },
        Some(S2T::AndFn) => {
            *i += 1;
            Some((S3T::OperatorFn(Operator::And), 1))
        },
        Some(S2T::OrFn) => {
            *i += 1;
            Some((S3T::OperatorFn(Operator::Or), 1))
        },
        Some(S2T::XorFn) => {
            *i += 1;
            Some((S3T::OperatorFn(Operator::Xor), 1))
        },
        Some(S2T::PropertyFn) => {
            *i += 1;
            Some((S3T::OperatorFn(Operator::Property), 1))
        },
        Some(S2T::TupleFn) => {
            *i += 1;
            Some((S3T::OperatorFn(Operator::Tuple), 1))
        },
        Some(S2T::PipeFn) => {
            *i += 1;
            Some((S3T::OperatorFn(Operator::Pipe), 1))
        },
        Some(S2T::OpenParen) => {
            let mut n = 0;
            *i += 1;
            n += 1;
            match expr_list(input, i) {
                Some((v, nn)) => {
                    n += nn;
                    match input.get(*i) {
                        Some(S2T::CloseParen) => {
                            *i += 1;
                            n += 1;
                            let v = match v.len() {
                                1 => v[1].clone(),
                                _ => S3T::Tuple(v),
                            };
                            Some((v, n))
                        },
                        _ => {
                            *i -= n;
                            None
                        },
                    }
                },
                _ => {
                    *i -= n;
                    None
                },
            }
        },
        _ => None,
    }
}

fn field(input: &Vec<S2T>, i: &mut usize) -> Option<(S3T, usize)> {
    match primary(input, i) {
        Some((v, n)) => {
            let mut v = v;
            let mut n = n;
            loop {
                v = match input.get(*i) {
                    Some(S2T::Dot) => {
                        *i += 1;
                        n += 1;
                        match input.get(*i) {
                            Some(S2T::Identifier(s)) => {
                                *i += 1;
                                n += 1;
                                S3T::Property(Box::new(v), Box::new(S3T::String(s.clone())))
                            },
                            _ => {
                                *i -= n;
                                return None;
                            },
                        }
                    },
                    Some(S2T::OpenBrack) => {
                        *i += 1;
                        n += 1;
                        match expr(input, i) {
                            Some((vv, nn)) => {
                                n += nn;
                                match input.get(*i) {
                                    Some(S2T::CloseParen) => {
                                        *i += 1;
                                        n += 1;
                                        S3T::Property(Box::new(v), Box::new(vv))
                                    },
                                    _ => {
                                        *i -= n;
                                        return None;
                                    },
                                }
                            },
                            _ => {
                                *i -= n;
                                return None;
                            },
                        }
                    },
                    _ => break,
                };
            }
            Some((v, n))
        },
        _ => None,
    }
}

fn expr_list(input: &Vec<S2T>, i: &mut usize) -> Option<(Vec<S3T>, usize)> {
    let mut res = Vec::new();
    let mut n = 0;
    match stat_expr(input, i) {
        Some((v, nn)) => {
            n += nn;
            res.push(v);
        },
        _ => match expr(input, i) {
            Some((v, nn)) => {
                n += nn;
                res.push(v);
            },
            _ => return None,
        },
    }
    loop {
        match input.get(*i) {
            Some(S2T::Comma) => {
                *i += 1;
                n += 1;
            },
            _ => break,
        };
        let t = match stat_expr(input, i) {
            Some((v, nn)) => {
                n += nn;
                res.push(v);
                Some(())
            },
            _ => {
                *i -= n;
                None
            },
        };
        let t = match t {
            Some(_) => Some(()),
            _ => match expr(input, i) {
                Some((v, nn)) => {
                    n += nn;
                    res.push(v);
                    Some(())
                },
                _ => {
                    *i -= n;
                    None
                },
            },
        };
        match t {
            Some(_) => {},
            _ => {
                return None;
            },
        }
    }
    Some((res, n))
}

fn fn_call(input: &Vec<S2T>, i: &mut usize) -> Option<(S3T, usize)> {
    match field(input, i) {
        Some((v, n)) => {
            let mut v = v;
            let mut n = n;
            loop {
                v = match input.get(*i) {
                    Some(S2T::OpenParen) => {
                        *i += 1;
                        n += 1;
                        match input.get(*i) {
                            Some(S2T::CloseParen) => {
                                *i += 1;
                                n += 1;
                                S3T::FnCall(Box::new(v), vec![])
                            },
                            _ => match expr_list(input, i) {
                                Some((vv, nn)) => {
                                    n += nn;
                                    match input.get(*i) {
                                        Some(S2T::CloseParen) => {
                                            *i += 1;
                                            n += 1;
                                            S3T::FnCall(Box::new(v), vv)
                                        },
                                        _ => {
                                            *i -= n;
                                            return None;
                                        }
                                    }
                                },
                                _ => {
                                    *i -= n;
                                    return None;
                                },
                            },
                        }
                    },
                    _ => break,
                };
            }
            Some((v, n))
        },
        _ => None,
    }
}

#[derive(Clone, Copy, PartialEq)]
enum UnaryOp {
    None,
    Negate,
    Not,
}

fn unary(input: &Vec<S2T>, i: &mut usize) -> Option<(S3T, usize)> {
    let mut n = 0;
    let op = match input.get(*i) {
        Some(S2T::Minus) => {
            *i += 1;
            n += 1;
            UnaryOp::Negate
        },
        Some(S2T::Not) => {
            *i += 1;
            n += 1;
            UnaryOp::Not
        },
        _ => UnaryOp::None,
    };
    match fn_call(input, i) {
        Some((v, nn)) => {
            n += nn;
            let v = match op {
                UnaryOp::None => v,
                UnaryOp::Negate => S3T::Negate(Box::new(v)),
                UnaryOp::Not => S3T::Not(Box::new(v)),
            };
            Some((v, n))
        },
        _ => None,
    }
}

fn times_divide(input: &Vec<S2T>, i: &mut usize) -> Option<(S3T, usize)> {
    match unary(input, i) {
        Some((v, n)) => {
            let mut v = v;
            let mut n = n;
            loop {
                v = match input.get(*i) {
                    Some(S2T::Times) => {
                        *i += 1;
                        n += 1;
                        match unary(input, i) {
                            Some((vv, nn)) => {
                                n += nn;
                                S3T::Times(Box::new(v), Box::new(vv))
                            },
                            _ => {
                                *i -= n;
                                return None;
                            },
                        }
                    },
                    Some(S2T::Divide) => {
                        *i += 1;
                        n += 1;
                        match unary(input, i) {
                            Some((vv, nn)) => {
                                n += nn;
                                S3T::Divide(Box::new(v), Box::new(vv))
                            },
                            _ => {
                                *i -= n;
                                return None;
                            },
                        }
                    },
                    Some(S2T::Modulus) => {
                        *i += 1;
                        n += 1;
                        match unary(input, i) {
                            Some((vv, nn)) => {
                                n += nn;
                                S3T::Modulus(Box::new(v), Box::new(vv))
                            },
                            _ => {
                                *i -= n;
                                return None;
                            },
                        }
                    },
                    _ => break,
                };
            }
            Some((v, n))
        },
        _ => None,
    }
}

fn plus_minus(input: &Vec<S2T>, i: &mut usize) -> Option<(S3T, usize)> {
    match times_divide(input, i) {
        Some((v, n)) => {
            let mut v = v;
            let mut n = n;
            loop {
                v = match input.get(*i) {
                    Some(S2T::Plus) => {
                        *i += 1;
                        n += 1;
                        match times_divide(input, i) {
                            Some((vv, nn)) => {
                                n += nn;
                                S3T::Plus(Box::new(v), Box::new(vv))
                            },
                            _ => {
                                *i -= n;
                                return None;
                            },
                        }
                    },
                    Some(S2T::Minus) => {
                        *i += 1;
                        n += 1;
                        match times_divide(input, i) {
                            Some((vv, nn)) => {
                                n += nn;
                                S3T::Minus(Box::new(v), Box::new(vv))
                            },
                            _ => {
                                *i -= n;
                                return None;
                            },
                        }
                    },
                    _ => break,
                }
            }
            Some((v, n))
        },
        _ => None,
    }
}

#[derive(Clone, Copy, PartialEq)]
enum ComparisonOp {
    EqualTo,
    NotEqualTo,
    Greater,
    Less,
    GreaterOrEqual,
    LessOrEqual,
}

fn comparison(input: &Vec<S2T>, i: &mut usize) -> Option<(S3T, usize)> {
    match plus_minus(input, i) {
        Some((v, n)) => {
            let mut v = v;
            let mut n = n;
            loop {
                let op = match input.get(*i) {
                    Some(S2T::EqualTo) => ComparisonOp::EqualTo,
                    Some(S2T::NotEqualTo) => ComparisonOp::NotEqualTo,
                    Some(S2T::Greater) => ComparisonOp::Greater,
                    Some(S2T::Less) => ComparisonOp::Less,
                    Some(S2T::GreaterOrEqual) => ComparisonOp::GreaterOrEqual,
                    Some(S2T::LessOrEqual) => ComparisonOp::LessOrEqual,
                    _ => break,
                };
                *i += 1;
                n += 1;
                let vv = match plus_minus(input, i) {
                    Some((vv, nn)) => {
                        n += nn;
                        vv
                    },
                    _ => {
                        *i -= n;
                        return None;
                    },
                };
                v = match op {
                    ComparisonOp::EqualTo => S3T::EqualTo(Box::new(v), Box::new(vv)),
                    ComparisonOp::NotEqualTo => S3T::NotEqualTo(Box::new(v), Box::new(vv)),
                    ComparisonOp::Greater => S3T::Greater(Box::new(v), Box::new(vv)),
                    ComparisonOp::Less => S3T::Less(Box::new(v), Box::new(vv)),
                    ComparisonOp::GreaterOrEqual => S3T::GreaterOrEqual(Box::new(v), Box::new(vv)),
                    ComparisonOp::LessOrEqual => S3T::LessOrEqual(Box::new(v), Box::new(vv)),
                };
            }
            Some((v, n))
        },
        _ => None,
    }
}

fn and(input: &Vec<S2T>, i: &mut usize) -> Option<(S3T, usize)> {
    match comparison(input, i) {
        Some((v, n)) => {
            let mut v = v;
            let mut n = n;
            loop {
                v = match input.get(*i) {
                    Some(S2T::And) => {
                        *i += 1;
                        n += 1;
                        match comparison(input, i) {
                            Some((vv, nn)) => {
                                n += nn;
                                S3T::And(Box::new(v), Box::new(vv))
                            },
                            _ => {
                                *i -= n;
                                return None;
                            },
                        }
                    },
                    _ => break,
                };
            }
            Some((v, n))
        },
        _ => None,
    }
}

fn xor(input: &Vec<S2T>, i: &mut usize) -> Option<(S3T, usize)> {
    match and(input, i) {
        Some((v, n)) => {
            let mut v = v;
            let mut n = n;
            loop {
                v = match input.get(*i) {
                    Some(S2T::Xor) => {
                        *i += 1;
                        n += 1;
                        match and(input, i) {
                            Some((vv, nn)) => {
                                n += nn;
                                S3T::Xor(Box::new(v), Box::new(vv))
                            },
                            _ => {
                                *i -= n;
                                return None;
                            },
                        }
                    },
                    _ => break,
                };
            }
            Some((v, n))
        },
        _ => None,
    }
}

fn or(input: &Vec<S2T>, i: &mut usize) -> Option<(S3T, usize)> {
    match xor(input, i) {
        Some((v, n)) => {
            let mut v = v;
            let mut n = n;
            loop {
                v = match input.get(*i) {
                    Some(S2T::Or) => {
                        *i += 1;
                        n += 1;
                        match xor(input, i) {
                            Some((vv, nn)) => {
                                n += nn;
                                S3T::Or(Box::new(v), Box::new(vv))
                            },
                            _ => {
                                *i -= n;
                                return None;
                            },
                        }
                    },
                    _ => break,
                };
            }
            Some((v, n))
        },
        _ => None,
    }
}

fn or_that(input: &Vec<S2T>, i: &mut usize) -> Option<(S3T, usize)> {
    match or(input, i) {
        Some((v, n)) => {
            let mut v = v;
            let mut n = n;
            loop {
                v = match input.get(*i) {
                    Some(S2T::OrThat) => {
                        *i += 1;
                        n += 1;
                        match or(input, i) {
                            Some((vv, nn)) => {
                                n += nn;
                                S3T::OrThat(Box::new(v), Box::new(vv))
                            },
                            _ => {
                                *i -= n;
                                return None;
                            },
                        }
                    },
                    _ => break,
                };
            }
            Some((v, n))
        },
        _ => None,
    }
}

fn ternary(input: &Vec<S2T>, i: &mut usize) -> Option<(S3T, usize)> {
    match or_that(input, i) {
        Some((v1, n)) => {
            let mut n = n;
            match input.get(*i) {
                Some(S2T::Question) => {
                    *i += 1;
                    n += 1;
                    let v2 = match or_that(input, i) {
                        Some((v2, nn)) => {
                            n += nn;
                            v2
                        },
                        _ => {
                            *i -= n;
                            return None;
                        },
                    };
                    match input.get(*i) {
                        Some(S2T::Colon) => {
                            *i += 1;
                            n += 1;
                        },
                        _ => {
                            *i -= n;
                            return None;
                        },
                    }
                    let v3 = match or_that(input, i) {
                        Some((v3, nn)) => {
                            n += nn;
                            v3
                        },
                        _ => {
                            *i -= n;
                            return None;
                        },
                    };
                    Some((S3T::Ternary(Box::new(v1), Box::new(v2), Box::new(v3)), n))
                },
                _ => Some((v1, n))
            }
        },
        _ => None,
    }
}

fn pipe(input: &Vec<S2T>, i: &mut usize) -> Option<(S3T, usize)> {
    match ternary(input, i) {
        Some((v, n)) => {
            let mut v = v;
            let mut n = n;
            loop {
                v = match input.get(*i) {
                    Some(S2T::Pipe) => {
                        *i += 1;
                        n += 1;
                        match ternary(input, i) {
                            Some((vv, nn)) => {
                                n += nn;
                                S3T::Pipe(Box::new(v), Box::new(vv))
                            },
                            _ => {
                                *i -= n;
                                return None;
                            },
                        }
                    },
                    _ => break,
                };
            }
            Some((v, n))
        },
        _ => None,
    }
}

fn expr(input: &Vec<S2T>, i: &mut usize) -> Option<(S3T, usize)> {
    let t = pipe(input, i);
    let t = match t {
        Some(t) => Some(t),
        _ => match input.get(*i) {
            Some(S2T::Or) => {
                *i += 1;
                let mut n = 1;
                let args = match ident_list(input, i) {
                    Some((args, nn)) => {
                        n += nn;
                        args
                    },
                    _ => vec![],
                };
                match input.get(*i) {
                    Some(S2T::Or) => {},
                    _ => {
                        *i -= n;
                        return None;
                    },
                }
                *i += 1;
                n += 1;
                let body = match expr(input, i) {
                    Some((v, nn)) => {
                        n += nn;
                        vec![S3T::Return(Box::new(v))]
                    },
                    _ => match input.get(*i) {
                        Some(S2T::OpenBrace) => {
                            *i += 1;
                            n += 1;
                            match stat_list(input, i) {
                                Some((v, nn)) => {
                                    n += nn;
                                    match input.get(*i) {
                                        Some(S2T::CloseBrace) => {
                                            *i += 1;
                                            n += 1;
                                            v
                                        },
                                        _ => {
                                            *i -= n;
                                            return None;
                                        },
                                    }
                                },
                                _ => {
                                    *i -= n;
                                    return None;
                                },
                            }
                        },
                        _ => {
                            *i -= n;
                            return None;
                        },
                    },
                };
                Some((S3T::Function(None, args, body), n))
            },
            _ => None,
        },
    };
    t
}

#[derive(Clone, Copy)]
enum AssignOp {
    Assign,
    Plus,
    Minus,
    Times,
    Divide,
    Modulus,
}

fn stat(input: &Vec<S2T>, i: &mut usize) -> Option<(S3T, usize)> {
    let t = match stat_expr(input, i) {
        Some(t) => Some(t),
        _ => None,
    };
    let t = match t {
        Some(t) => Some(t),
        _ => match expr(input, i) {
            Some((v, n)) => {
                let mut n = n;
                match input.get(*i) {
                    Some(S2T::Semicolon) => {
                        *i += 1;
                        n += 1;
                        Some((v, n))
                    },
                    _ => {
                        *i -= n;
                        return None;
                    },
                }
            },
            _ => None,
        },
    };
    let t = match t {
        Some(t) => Some(t),
        _ => match input.get(*i) {
            Some(S2T::OpenParen) => {
                let mut n = 1;
                *i += 1;
                match expr_list(input, i) {
                    Some((v, nn)) => {
                        n += nn;
                        match input.get(*i) {
                            Some(S2T::CloseParen) => {
                                *i += 1;
                                n += 1;
                                match input.get(*i) {
                                    Some(S2T::Assign) => {
                                        *i += 1;
                                        n += 1;
                                        match expr(input, i) {
                                            Some((r, nn)) => {
                                                n += nn;
                                                match input.get(*i) {
                                                    Some(S2T::Semicolon) => {
                                                        *i += 1;
                                                        n += 1;
                                                        Some((S3T::Detuple(v, Box::new(r)), n))
                                                    },
                                                    _ => {
                                                        *i -= n;
                                                        None
                                                    },
                                                }
                                            },
                                            _ => {
                                                *i -= n;
                                                None
                                            },
                                        }
                                    },
                                    _ => {
                                        *i -= n;
                                        None
                                    },
                                }
                            },
                            _ => {
                                *i -= n;
                                None
                            },
                        }
                    },
                    _ => {
                        *i -= n;
                        None
                    },
                }
            },
            _ => None,
        },
    };
    let t = match t {
        Some(t) => Some(t),
        _ => match expr(input, i) {
            Some((left, nn)) => {
                let mut n = nn;
                let op = match input.get(*i) {
                    Some(S2T::Assign) => Some(AssignOp::Assign),
                    Some(S2T::PlusAssign) => Some(AssignOp::Plus),
                    Some(S2T::MinusAssign) => Some(AssignOp::Minus),
                    Some(S2T::TimesAssign) => Some(AssignOp::Times),
                    Some(S2T::DivideAssign) => Some(AssignOp::Divide),
                    Some(S2T::ModulusAssign) => Some(AssignOp::Modulus),
                    _ => None,
                };
                if let Some(op) = op {
                    *i += 1;
                    n += 1;
                    match expr(input, i) {
                        Some((right, nn)) => {
                            n += nn;
                            match input.get(*i) {
                                Some(S2T::Semicolon) => {
                                    *i += 1;
                                    n += 1;
                                    match op {
                                        AssignOp::Assign => Some((S3T::Assign(Box::new(left), Box::new(right)), n)),
                                        AssignOp::Plus => Some((S3T::Assign(Box::new(left.clone()), Box::new(S3T::Plus(Box::new(left.clone()), Box::new(right)))), n)),
                                        AssignOp::Minus => Some((S3T::Assign(Box::new(left.clone()), Box::new(S3T::Minus(Box::new(left.clone()), Box::new(right)))), n)),
                                        AssignOp::Times => Some((S3T::Assign(Box::new(left.clone()), Box::new(S3T::Times(Box::new(left.clone()), Box::new(right)))), n)),
                                        AssignOp::Divide => Some((S3T::Assign(Box::new(left.clone()), Box::new(S3T::Divide(Box::new(left.clone()), Box::new(right)))), n)),
                                        AssignOp::Modulus => Some((S3T::Assign(Box::new(left.clone()), Box::new(S3T::Modulus(Box::new(left.clone()), Box::new(right)))), n)),
                                    }
                                },
                                _ => {
                                    *i -= n;
                                    None
                                },
                            }
                        },
                        _ => {
                            *i -= n;
                            None
                        },
                    }
                } else {
                    match input.get(*i) {
                        Some(S2T::PlusPlus) => {
                            *i += 1;
                            n += 1;
                            match input.get(*i) {
                                Some(S2T::Semicolon) => Some((S3T::Assign(Box::new(left.clone()), Box::new(S3T::Plus(Box::new(left.clone()), Box::new(S3T::Number(1))))), n)),
                                _ => {
                                    *i -= n;
                                    None
                                },
                            }
                        },
                        Some(S2T::MinusMinus) => {
                            *i += 1;
                            n += 1;
                            match input.get(*i) {
                                Some(S2T::Semicolon) => Some((S3T::Assign(Box::new(left.clone()), Box::new(S3T::Minus(Box::new(left.clone()), Box::new(S3T::Number(1))))), n)),
                                _ => {
                                    *i -= n;
                                    None
                                },
                            }
                        },
                        _ => {
                            *i -= n;
                            None
                        },
                    }
                }
            },
            _ => None,
        },
    };
    let t = match t {
        Some(t) => Some(t),
        _ => match input.get(*i) {
            Some(S2T::Let) => {
                let mut n = 1;
                *i += 1;
                match input.get(*i) {
                    Some(S2T::Identifier(s)) => {
                        n += 1;
                        *i += 1;
                        match input.get(*i) {
                            Some(S2T::Semicolon) => {
                                n += 1;
                                *i += 1;
                                Some((S3T::Let(s.clone(), None), n))
                            },
                            Some(S2T::Assign) => {
                                n += 1;
                                *i += 1;
                                let t = match stat_expr(input, i) {
                                    Some((v, nn)) => {
                                        n += nn;
                                        Some((S3T::Let(s.clone(), Some(Box::new(v))), n))
                                    },
                                    _ => None,
                                };
                                let t = match t {
                                    Some(t) => Some(t),
                                    _ => match expr(input, i) {
                                        Some((v, nn)) => {
                                            n += nn;
                                            match input.get(*i) {
                                                Some(S2T::Semicolon) => Some((S3T::Let(s.clone(), Some(Box::new(v))), nn + n + 1)),
                                                _ => {
                                                    *i -= nn;
                                                    None
                                                },
                                            }
                                        },
                                        _ => None,
                                    },
                                };
                                match t {
                                    Some(t) => Some(t),
                                    None => {
                                        *i -= n;
                                        None
                                    },
                                }
                            },
                            _ => {
                                *i -= n;
                                None
                            }
                        }
                    },
                    Some(S2T::OpenParen) => {
                        n += 1;
                        *i += 1;
                        match ident_list(input, i) {
                            Some((ident, nn)) => {
                                n += nn;
                                match input.get(*i) {
                                    Some(S2T::CloseParen) => {
                                        *i += 1;
                                        n += 1;
                                        match input.get(*i) {
                                            Some(S2T::Assign) => {
                                                *i += 1;
                                                n += 1;
                                                let t = match stat_expr(input, i) {
                                                    Some((v, nn)) => {
                                                        *i += 1;
                                                        match input.get(*i) {
                                                            Some(S2T::Semicolon) => Some((S3T::LetDetuple(ident.clone(), Box::new(v)), nn + n + 1)),
                                                            _ => {
                                                                *i -= nn;
                                                                *i -= 1;
                                                                None
                                                            },
                                                        }
                                                    },
                                                    _ => None,
                                                };
                                                let t = match t {
                                                    Some(t) => Some(t),
                                                    _ => match expr(input, i) {
                                                        Some((v, nn)) => {
                                                            *i += 1;
                                                            match input.get(*i) {
                                                                Some(S2T::Semicolon) => Some((S3T::LetDetuple(ident.clone(), Box::new(v)), nn + n + 1)),
                                                                _ => {
                                                                    *i -= nn;
                                                                    *i -= 1;
                                                                    None
                                                                },
                                                            }
                                                        },
                                                        _ => None,
                                                    },
                                                };
                                                t
                                            },
                                            Some(S2T::Semicolon) => {
                                                *i += 1;
                                                n += 1;
                                                Some((S3T::MultiLet(ident), n))
                                            },
                                            _ => {
                                                *i -= n;
                                                None
                                            },
                                        }
                                    },
                                    _ => {
                                        *i -= n;
                                        None
                                    },
                                }
                            },
                            _ => {
                                *i -= n;
                                None
                            },
                        }
                    },
                    _ => {
                        *i -= n;
                        None
                    },
                }
            },
            _ => None,
        },
    };
    let t = match t {
        Some(t) => Some(t),
        _ => match input.get(*i) {
            Some(S2T::Out) => {
                let mut n = 1;
                *i += 1;
                let e = match expr(input, i) {
                    Some((v, nn)) => {
                        n += nn;
                        match input.get(*i) {
                            Some(S2T::Semicolon) => {
                                *i += 1;
                                n += 1;
                                v
                            },
                            _ => {
                                *i -= n;
                                return None;
                            },
                        }
                    },
                    _ => match stat_expr(input, i) {
                        Some((v, nn)) => {
                            n += nn;
                            match input.get(*i) {
                                Some(S2T::Semicolon) => {
                                    *i += 1;
                                    n += 1;
                                    v
                                },
                                _ => {
                                    *i -= n;
                                    return None;
                                },
                            }
                        },
                        _ => {
                            *i -= n;
                            return None;
                        },
                    },
                };
                Some((S3T::Out(Box::new(e)), n))
            },
            _ => None,
        },
    };
    let t = match t {
        Some(t) => Some(t),
        _ => match input.get(*i) {
            Some(S2T::Label(s)) => {
                *i += 1;
                Some((S3T::Label(s.clone()), 1))
            },
            _ => None,
        },
    };
    let t = match t {
        Some(t) => Some(t),
        _ => match input.get(*i) {
            Some(S2T::Return) => {
                let mut n = 1;
                *i += 1;
                let e = match expr(input, i) {
                    Some((v, nn)) => {
                        n += nn;
                        match input.get(*i) {
                            Some(S2T::Semicolon) => {
                                *i += 1;
                                n += 1;
                                v
                            },
                            _ => {
                                *i -= n;
                                return None;
                            },
                        }
                    },
                    _ => match stat_expr(input, i) {
                        Some((v, nn)) => {
                            n += nn;
                            match input.get(*i) {
                                Some(S2T::Semicolon) => {
                                    *i += 1;
                                    n += 1;
                                    v
                                },
                                _ => {
                                    *i -= n;
                                    return None;
                                },
                            }
                        },
                        _ => {
                            *i -= n;
                            return None;
                        },
                    },
                };
                Some((S3T::Return(Box::new(e)), n))
            },
            _ => None,
        },
    };
    let t = match t {
        Some(t) => Some(t),
        _ => match input.get(*i) {
            Some(S2T::Throw) => {
                let mut n = 1;
                *i += 1;
                let e = match expr(input, i) {
                    Some((v, nn)) => {
                        n += nn;
                        match input.get(*i) {
                            Some(S2T::Semicolon) => {
                                *i += 1;
                                n += 1;
                                v
                            },
                            _ => {
                                *i -= n;
                                return None;
                            },
                        }
                    },
                    _ => match stat_expr(input, i) {
                        Some((v, nn)) => {
                            n += nn;
                            match input.get(*i) {
                                Some(S2T::Semicolon) => {
                                    *i += 1;
                                    n += 1;
                                    v
                                },
                                _ => {
                                    *i -= n;
                                    return None;
                                },
                            }
                        },
                        _ => {
                            *i -= n;
                            return None;
                        },
                    },
                };
                Some((S3T::Throw(Box::new(e)), n))
            },
            _ => None,
        },
    };
    let t = match t {
        Some(t) => Some(t),
        _ => {
            let looptype = match input.get(*i) {
                Some(S2T::While) => Some(LoopType::While),
                Some(S2T::Until) => Some(LoopType::Until),
                _ => None,
            };
            match looptype {
                Some(looptype) => {
                    let mut n = 1;
                    *i += 1;
                    let cond = match expr(input, i) {
                        Some((v, nn)) => {
                            n += nn;
                            v
                        },
                        _ => {
                            *i -= n;
                            return None;
                        },
                    };
                    let body = match input.get(*i) {
                        Some(S2T::OpenBrace) => {
                            n += 1;
                            *i += 1;
                            match stat_list(input, i) {
                                Some((v, nn)) => {
                                    n += nn;
                                    match input.get(*i) {
                                        Some(S2T::CloseBrace) => {
                                            n += 1;
                                            *i += 1;
                                            v
                                        },
                                        _ => {
                                            *i -= n;
                                            return None;
                                        },
                                    }
                                },
                                _ => {
                                    *i -= n;
                                    return None;
                                },
                            }
                        },
                        _ => {
                            *i -= n;
                            return None;
                        },
                    };
                    Some((S3T::WhileUntil(looptype, Box::new(cond), body), n))
                },
                _ => None,
            }
        },
    };
    let t = match t {
        Some(t) => Some(t),
        _ => match input.get(*i) {
            Some(S2T::Defer) => {
                let mut n = 1;
                *i += 1;
                let e = match input.get(*i) {
                    Some(S2T::OpenBrace) => {
                        n += 1;
                        *i += 1;
                        match stat_list(input, i) {
                            Some((v, nn)) => {
                                n += nn;
                                match input.get(*i) {
                                    Some(S2T::CloseBrace) => {
                                        n += 1;
                                        *i += 1;
                                        Some(v)
                                    },
                                    _ => {
                                        *i -= n;
                                        return None;
                                    },
                                }
                            },
                            _ => {
                                *i -= n;
                                return None;
                            },
                        }
                    },
                    _ => None,
                };
                let e = match e {
                    Some(e) => Some(e),
                    _ => match expr(input, i) {
                        Some((v, nn)) => {
                            n += nn;
                            match input.get(*i) {
                                Some(S2T::Semicolon) => {
                                    *i += 1;
                                    n += 1;
                                    Some(vec![v])
                                },
                                _ => {
                                    *i -= n;
                                    return None;
                                },
                            }
                        },
                        _ => None,
                    },
                };
                let e = match e {
                    Some(e) => Some(e),
                    _ => match stat_expr(input, i) {
                        Some((v, nn)) => {
                            n += nn;
                            match input.get(*i) {
                                Some(S2T::Semicolon) => {
                                    *i += 1;
                                    n += 1;
                                    Some(vec![v])
                                },
                                _ => {
                                    *i -= n;
                                    return None;
                                },
                            }
                        },
                        _ => None,
                    },
                };
                match e {
                    Some(e) => {
                        Some((S3T::Defer(e), n))
                    },
                    _ => {
                        *i -= n;
                        return None;
                    },
                }
            },
            _ => None,
        },
    };
    let t = match t {
        Some(t) => Some(t),
        _ => match input.get(*i) {
            Some(S2T::Bind) => {
                let mut n = 1;
                *i += 1;
                let name = match input.get(*i) {
                    Some(S2T::Identifier(s)) => s.clone(),
                    _ => {
                        *i -= n;
                        return None;
                    },
                };
                *i += 1;
                n += 1;
                match input.get(*i) {
                    Some(S2T::Assign) => {},
                    _ => {
                        *i -= n;
                        return None;
                    },
                }
                *i += 1;
                n += 1;
                let e = match expr(input, i) {
                    Some((v, nn)) => {
                        n += nn;
                        match input.get(*i) {
                            Some(S2T::Semicolon) => {
                                *i += 1;
                                n += 1;
                                Some(v)
                            },
                            _ => {
                                *i -= n;
                                return None;
                            },
                        }
                    },
                    _ => None,
                };
                let e = match e {
                    Some(e) => Some(e),
                    _ => match stat_expr(input, i) {
                        Some((v, nn)) => {
                            n += nn;
                            match input.get(*i) {
                                Some(S2T::Semicolon) => {
                                    *i += 1;
                                    n += 1;
                                    Some(v)
                                },
                                _ => {
                                    *i -= n;
                                    return None;
                                },
                            }
                        },
                        _ => None,
                    },
                };
                match e {
                    Some(e) => {
                        Some((S3T::Bind(name, Box::new(e)), n))
                    },
                    _ => {
                        *i -= n;
                        return None;
                    },
                }
            },
            _ => None, 
        },
    };
    let t = match t {
        Some(t) => Some(t),
        _ => match input.get(*i) {
            Some(S2T::At) => {
                let mut n = 1;
                *i += 1;
                let d = match expr(input, i) {
                    Some((v, nn)) => {
                        n += nn;
                        v
                    },
                    _ => {
                        *i -= n;
                        return None;
                    },
                };
                let f = match function(input, i) {
                    Some((v, nn)) => {
                        n += nn;
                        v
                    },
                    _ => {
                        *i -= n;
                        return None;
                    },
                };
                Some((S3T::Decorator(Box::new(d), Box::new(f)), n))
            },
            _ => None,
        },
    };
    let t = match t {
        Some(t) => Some(t),
        _ => match input.get(*i) {
            Some(S2T::Goto) => {
                let mut n = 1;
                *i += 1;
                match input.get(*i) {
                    Some(S2T::Label(s)) => {
                        *i += 1;
                        n += 1;
                        match input.get(*i) {
                            Some(S2T::Semicolon) => {
                                *i += 1;
                                n += 1;
                                Some((S3T::Goto(s.clone()), n))
                            },
                            _ => {
                                *i -= n;
                                None
                            },
                        }
                    },
                    _ => {
                        *i -= n;
                        None
                    },
                }
            },
            _ => None,
        },
    };
    t
}

fn stat_expr(input: &Vec<S2T>, i: &mut usize) -> Option<(S3T, usize)> {
    let t = match if_list(input, i) {
        Some((v, n)) => {
            let mut n = n;
            let else_ = match input.get(*i) {
                Some(S2T::Else) => {
                    n += 1;
                    *i += 1;
                    match input.get(*i) {
                        Some(S2T::OpenBrace) => {
                            n += 1;
                            *i += 1;
                            let stats = match stat_list(input, i) {
                                Some((v, nn)) => {
                                    n += nn;
                                    match input.get(*i) {
                                        Some(S2T::CloseBrace) => {
                                            *i += 1;
                                            n += 1;
                                            v
                                        },
                                        _ => {
                                            *i -= n;
                                            return None;
                                        },
                                    }
                                },
                                _ => {
                                    *i -= n;
                                    return None;
                                },
                            };
                            Some(stats)
                        },
                        _ => {
                            *i -= n;
                            return None;
                        },
                    }
                },
                _ => None,
            };
            Some((S3T::If(v, else_), n))
        },
        _ => None,
    };
    let t = match t {
        Some(t) => Some(t),
        _ => match input.get(*i) {
            Some(S2T::Try) => {
                let mut n = 1;
                *i += 1;
                let try_ = match input.get(*i) {
                    Some(S2T::OpenBrace) => {
                        n += 1;
                        *i += 1;
                        match stat_list(input, i) {
                            Some((v, nn)) => {
                                n += nn;
                                match input.get(*i) {
                                    Some(S2T::CloseBrace) => {
                                        n += 1;
                                        *i += 1;
                                        v
                                    },
                                    _ => {
                                        *i -= n;
                                        return None;
                                    },
                                }
                            },
                            _ => {
                                *i -= n;
                                return None;
                            },
                        }
                    },
                    _ => {
                        *i -= n;
                        return None;
                    },
                };
                let catch = match input.get(*i) {
                    Some(S2T::Catch) => {
                        *i += 1;
                        n += 1;
                        match input.get(*i) {
                            Some(S2T::Identifier(s)) => {
                                *i += 1;
                                n += 1;
                                match input.get(*i) {
                                    Some(S2T::OpenBrace) => {
                                        *i += 1;
                                        n += 1;
                                        match stat_list(input, i) {
                                            Some((v, nn)) => {
                                                n += nn;
                                                match input.get(*i) {
                                                    Some(S2T::CloseBrace) => {
                                                        n += 1;
                                                        *i += 1;
                                                        (s.clone(), v)
                                                    },
                                                    _ => {
                                                        *i -= n;
                                                        return None;
                                                    }
                                                }
                                            },
                                            _ => {
                                                *i -= n;
                                                return None;
                                            },
                                        }
                                    },
                                    _ => {
                                        *i -= n;
                                        return None;
                                    },
                                }
                            },
                            _ => {
                                *i -= n;
                                return None;
                            },
                        }
                    },
                    _ => {
                        *i -= n;
                        return None;
                    },
                };
                let finally = match input.get(*i) {
                    Some(S2T::Finally) => {
                        *i += 1;
                        n += 1;
                        match input.get(*i) {
                            Some(S2T::OpenBrace) => {
                                *i += 1;
                                n += 1;
                                match stat_list(input, i) {
                                    Some((v, nn)) => {
                                        n += nn;
                                        match input.get(*i) {
                                            Some(S2T::CloseBrace) => {
                                                *i += 1;
                                                n += 1;
                                                Some(v)
                                            },
                                            _ => {
                                                *i -= n;
                                                return None;
                                            },
                                        }
                                    },
                                    _ => {
                                        *i -= n;
                                        return None;
                                    },
                                }
                            },
                            _ => {
                                *i -= n;
                                return None;
                            },
                        }
                    },
                    _ => None,
                };
                Some((S3T::TryCatchFinally(try_, catch.0, catch.1, finally), n))
            },
            _ => None,
        },
    };
    let t = match t {
        Some(t) => Some(t),
        _ => match input.get(*i) {
            Some(S2T::Loop) => {
                let mut n = 1;
                *i += 1;
                let loop_ = match input.get(*i) {
                    Some(S2T::OpenBrace) => {
                        n += 1;
                        *i += 1;
                        match stat_list(input, i) {
                            Some((v, nn)) => {
                                n += nn;
                                match input.get(*i) {
                                    Some(S2T::CloseBrace) => {
                                        n += 1;
                                        *i += 1;
                                        v
                                    },
                                    _ => {
                                        *i -= n;
                                        return None;
                                    },
                                }
                            },
                            _ => {
                                *i -= n;
                                return None;
                            },
                        }
                    },
                    _ => {
                        *i -= n;
                        return None;
                    },
                };
                Some((S3T::Loop(loop_), n))
            },
            _ => None,
        },
    };
    let t = match t {
        Some(t) => Some(t),
        _ => match function(input, i) {
            Some(t) => Some(t),
            _ => None,
        },
    };
    let t = match t {
        Some(t) => Some(t),
        _ => match input.get(*i) {
            Some(S2T::OpenBrace) => {
                let mut n = 1;
                *i += 1;
                let stats = match stat_list(input, i) {
                    Some((s, nn)) => {
                        n += nn;
                        match input.get(*i) {
                            Some(S2T::CloseBrace) => {
                                *i += 1;
                                n += 1;
                                s
                            },
                            _ => {
                                *i -= n;
                                return None;
                            },
                        }
                    },
                    _ => {
                        *i -= n;
                        return None;
                    },
                };
                Some((S3T::Block(stats), n))
            },
            _ => None,
        },
    };
    t
}

fn function(input: &Vec<S2T>, i: &mut usize) -> Option<(S3T, usize)> {
    match input.get(*i) {
        Some(S2T::Fn) => {},
        _ => return None,
    }
    *i += 1;
    let mut n = 1;
    let name = match input.get(*i) {
        Some(S2T::Identifier(s)) => {
            *i += 1;
            n += 1;
            Some(s.clone())
        },
        _ => None,
    };
    match input.get(*i) {
        Some(S2T::OpenParen) => {},
        _ => {
            *i -= n;
            return None;
        },
    }
    n += 1;
    *i += 1;
    let args = match ident_list(input, i) {
        Some((args, nn)) => {
            n += nn;
            args
        },
        _ => vec![],
    };
    match input.get(*i) {
        Some(S2T::CloseParen) => {},
        _ => {
            *i -= n;
            return None;
        },
    }
    n += 1;
    *i += 1;
    match input.get(*i) {
        Some(S2T::OpenBrace) => {},
        _ => {
            *i -= n;
            return None;
        },
    }
    n += 1;
    *i += 1;
    let body = match stat_list(input, i) {
        Some((body, nn)) => {
            n += nn;
            body
        },
        _ => {
            *i -= n;
            return None;
        },
    };
    match input.get(*i) {
        Some(S2T::CloseBrace) => {},
        _ => {
            *i -= n;
            return None;
        },
    }
    n += 1;
    *i += 1;
    Some((S3T::Function(name, args, body), n))
}

fn ident_list(input: &Vec<S2T>, i: &mut usize) -> Option<(Vec<String>, usize)> {
    let mut res = Vec::new();
    let mut n = 0;
    match input.get(*i) {
        Some(S2T::Identifier(s)) => {
            res.push(s.clone());
            n += 1;
            *i += 1;
        },
        _ => return None,
    }
    loop {
        match input.get(*i) {
            Some(S2T::Comma) => {},
            _ => break,
        }
        *i += 1;
        n += 1;
        match input.get(*i) {
            Some(S2T::Identifier(s)) => {
                res.push(s.clone());
                n += 1;
                *i += 1;
            },
            _ => {
                *i -= n;
                return None;
            },
        }
    }
    Some((res, n))
}

fn if_list(input: &Vec<S2T>, i: &mut usize) -> Option<(Vec<(ConditionType, Box<S3T>, Vec<S3T>)>, usize)> {
    let mut res = Vec::new();
    let mut n = 0;
    {
        let op = match input.get(*i) {
            Some(S2T::If) => ConditionType::If,
            Some(S2T::Unless) => ConditionType::Unless,
            _ => return None,
        };
        n += 1;
        *i += 1;
        let cond = match expr(input, i) {
            Some((v, nn)) => {
                n += nn;
                v
            },
            _ => {
                *i -= n;
                return None;
            },
        };
        match input.get(*i) {
            Some(S2T::OpenBrace) => {},
            _ => {
                *i -= n;
                return None;
            },
        }
        n += 1;
        *i += 1;
        let stats = match stat_list(input, i) {
            Some((v, nn)) => {
                n += nn;
                v
            },
            _ => {
                *i -= n;
                return None;
            },
        };
        match input.get(*i) {
            Some(S2T::CloseBrace) => {},
            _ => {
                *i -= n;
                return None;
            },
        }
        *i += 1;
        n += 1;
        res.push((op, Box::new(cond), stats));
    }
    loop {
        let op = match input.get(*i) {
            Some(S2T::ElseIf) => ConditionType::If,
            Some(S2T::ElseUnless) => ConditionType::Unless,
            _ => break,
        };
        n += 1;
        *i += 1;
        let cond = match expr(input, i) {
            Some((v, nn)) => {
                n += nn;
                v
            },
            _ => {
                *i -= n;
                return None;
            },
        };
        match input.get(*i) {
            Some(S2T::OpenBrace) => {},
            _ => {
                *i -= n;
                return None;
            },
        }
        n += 1;
        *i += 1;
        let stats = match stat_list(input, i) {
            Some((v, nn)) => {
                n += nn;
                v
            },
            _ => {
                *i -= n;
                return None;
            },
        };
        match input.get(*i) {
            Some(S2T::CloseBrace) => {},
            _ => {
                *i -= n;
                return None;
            },
        }
        *i += 1;
        n += 1;
        res.push((op, Box::new(cond), stats));
    }
    Some((res, n))
}

fn stat_list(input: &Vec<S2T>, i: &mut usize) -> Option<(Vec<S3T>, usize)> {
    let mut res = Vec::new();
    let mut n = 0;
    match stat(input, i) {
        Some((v, nn)) => {
            n += nn;
            res.push(v);
        },
        _ => return Some((res, n)),
    }
    loop {
        match stat(input, i) {
            Some((v, nn)) => {
                n += nn;
                res.push(v);
            },
            _ => break,
        }
    }
    Some((res, n))
}