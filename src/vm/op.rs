use std::{collections::HashMap, sync::{Arc, Mutex}};

use crate::vm::{make_container, make_err, Container, Function, Instruction, Label, Object, ObjectProp, State, StateContainer, Value, VarIndex};

// TODO: add comments

pub fn resolve_bind(state: StateContainer, v: Container) -> Result<Container, Container> {
    let b = v.lock().unwrap().clone();
    match b {
        Value::Bind(b) => {
            let b = call(state.clone(), b, Vec::new())?;
            resolve_bind(state, b)
        },
        _ => Ok(v)
    }
}

pub fn add(state: StateContainer, a: Container, b: Container) -> Result<Container, Container> {
    let x = resolve_bind(state.clone(), a)?;
    let y = resolve_bind(state.clone(), b)?;
    let a = x.clone().lock().unwrap().clone();
    let b = y.clone().lock().unwrap().clone();
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => Ok(make_container(Value::Number(a+b))),
        (Value::Number(a), Value::Float(b)) => Ok(make_container(Value::Float((a as f64)+b))),
        (Value::Float(a), Value::Number(b)) => Ok(make_container(Value::Float(a+(b as f64)))),
        (Value::Float(a), Value::Float(b)) => Ok(make_container(Value::Float(a+b))),
        (Value::String(a), Value::String(b)) => Ok(make_container(Value::String(a+&b))),
        (Value::Function(_), Value::Function(_)) => {
            let h = {
                let mut internals = HashMap::new();
                internals.insert(0, x);
                internals.insert(1, y);
                Function {
                    internals,
                    call: |state, args, gi| {
                        let f = gi(0).unwrap();
                        let g = gi(1).unwrap();
                        let r = call(state.clone(), f, args)?;
                        call(state.clone(), g, vec![r])
                    },
                    state: state.clone(),

                }
            };
            Ok(make_container(Value::Function(h)))
        },
        (Value::Boolean(a), Value::Boolean(b)) => Ok(make_container(Value::Boolean(a&b))),
        (Value::Object(obj), _) => {
            call_metaprop(state.clone(), obj, vec![x, y], "add".to_string())
        },
        (_, Value::Object(obj)) => {
            call_metaprop(state.clone(), obj, vec![x, y], "add".to_string())
        },
        _ => Err(make_err("can't add a with b")),
    }
}

pub fn call(state: StateContainer, f: Container, args: Vec<Container>) -> Result<Container, Container> {
    let g = resolve_bind(state.clone(), f)?;
    let f = g.clone().lock().unwrap().clone();
    match f {
        Value::Function(f) => {
            let gi = {
                let i = f.internals.clone();
                move |v: u64| {i.get(&v).map(|s| s.clone())}
            };
            (f.call)(f.state.clone(), args, Arc::new(gi))
        },
        Value::Object(obj) => {
            call_metaprop(state.clone(), obj, vec![g].iter().chain(args.iter()).map(|h| h.clone()).collect::<Vec<Container>>(), "call".to_string())
        },
        _ => Err(make_err("can't call f")),
    }
}

pub fn to_string_base(state: StateContainer, v: Container) -> Result<String, Container> {
    let x = resolve_bind(state.clone(), v)?;
    let v = x.clone().lock().unwrap().clone();
    match v {
        Value::String(v) => Ok(v),
        Value::Number(v) => Ok(v.to_string()),
        Value::Float(v) => Ok(v.to_string()),
        Value::Null => Ok("null".to_string()),
        Value::Boolean(v) => Ok(v.to_string()),
        Value::Object(obj) => {
            let tmp = call_metaprop(state.clone(), obj, vec![x], "to_string".to_string())?;
            to_string_base(state.clone(), tmp)
        },
        Value::Tuple(t) => {
            let mut s = Vec::new();
            for i in t.iter() {
                s.push(to_string_base(state.clone(), i.clone())?);
            }
            Ok("(".to_string()+&s.join(", ")+")")
        },
        _ => Err(make_err("can't convert v to string")),
    }
}

pub fn to_string(state: StateContainer, v: Container) -> Result<Container, Container> {
    Ok(make_container(Value::String(to_string_base(state, v)?)))
}

pub fn get_base(state: StateContainer, obj: Container, prop: String) -> Result<Container, Container> {
    let o = resolve_bind(state.clone(), obj)?;
    let obj = o.lock().unwrap().clone();
    match obj {
        Value::Object(obj) => {
            let p = obj.props.get(&prop);
            match p {
                None => match call_metaprop(state.clone(), obj, vec![o, make_container(Value::String(prop.clone()))], "get".to_string()) {
                    Ok(v) => Ok(v),
                    Err(_) => Ok(make_container(Value::Null)),
                },
                Some(prop) => {
                    match prop {
                        ObjectProp::Value(prop) => Ok(prop.clone()),
                        ObjectProp::GetSet(getter, _) => {
                            call(state, getter.clone(), Vec::new())
                        },
                    }
                },
            }
        },
        _ => Err(make_err("can't get property prop on object obj")),
    }
}

pub fn get(state: StateContainer, obj: Container, prop: Container) -> Result<Container, Container> {
    get_base(state.clone(), obj, to_string_base(state.clone(), prop)?)
}

pub fn call_metaprop(state: StateContainer, obj: Object, args: Vec<Container>, prop: String) -> Result<Container, Container> {
    let metaobj = obj.metaobj;
    let f = get_base(state.clone(), metaobj, prop)?;
    call(state, f, args)
}

pub fn multiply(state: StateContainer, a: Container, b: Container) -> Result<Container, Container> {
    let x = resolve_bind(state.clone(), a)?;
    let y = resolve_bind(state.clone(), b)?;
    let a = x.clone().lock().unwrap().clone();
    let b = y.clone().lock().unwrap().clone();
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => Ok(make_container(Value::Number(a*b))),
        (Value::Number(a), Value::Float(b)) => Ok(make_container(Value::Float((a as f64)*b))),
        (Value::Float(a), Value::Number(b)) => Ok(make_container(Value::Float(a*(b as f64)))),
        (Value::Float(a), Value::Float(b)) => Ok(make_container(Value::Float(a*b))),
        (Value::String(s), Value::Number(n)) => {
            let s = if n == 1 {
                s.clone()
            } else if n == 0 {
                "".to_string()
            } else if n == -1 {
                s.chars().rev().collect::<String>()
            } else if n > 1 {
                let mut a = "".to_string();
                let mut i = n;
                while i > 0 {
                    a += &s;
                    i -= 1;
                }
                a.clone()
            } else {
                let mut a = "".to_string();
                let mut i = -n;
                while i > 0 {
                    a += &s;
                    i -= 1;
                }
                a.chars().rev().collect::<String>()
            };
            Ok(make_container(Value::String(s)))
        },
        (Value::Number(n), Value::String(s)) => {
            let s = if n == 1 {
                s.clone()
            } else if n == 0 {
                "".to_string()
            } else if n == -1 {
                s.chars().rev().collect::<String>()
            } else if n > 1 {
                let mut a = "".to_string();
                let mut i = n;
                while i > 0 {
                    a += &s;
                    i -= 1;
                }
                a.clone()
            } else {
                let mut a = "".to_string();
                let mut i = -n;
                while i > 0 {
                    a += &s;
                    i -= 1;
                }
                a.chars().rev().collect::<String>()
            };
            Ok(make_container(Value::String(s)))
        },
        (Value::Function(_), Value::Number(n)) => {
            if n < 1 {
                return Err(make_err("can't multiply a with b"))
            }
            let g = {
                let mut internals = HashMap::new();
                internals.insert(0, x);
                internals.insert(1, y);
                Function {
                    internals,
                    call: |state, args, gi| {
                        let arg = if args.len() == 0 {
                            return Err(make_err("less than 1 argument"));
                        } else {
                            args[0].clone()
                        };
                        let f = gi(0).unwrap();
                        let n = match gi(1).unwrap().lock().unwrap().clone() {
                            Value::Number(n) => {
                                n
                            },
                            _ => return Err(make_err("data corrupted")),
                        };
                        if n < 1 {
                            return Err(make_err("data corrupted"))
                        }
                        let mut r = make_container(Value::Null);
                        let mut i = n;
                        while i > 0 {
                            r = call(state.clone(), f.clone(), vec![arg.clone()])?;
                            i -= 1;
                        }
                        Ok(r)
                    },
                    state: state.clone(),
                }
            };
            Ok(make_container(Value::Function(g)))
        },
        (Value::Number(n), Value::Function(_)) => {
            if n < 1 {
                return Err(make_err("can't multiply a with b"))
            }
            let g = {
                let mut internals = HashMap::new();
                internals.insert(0, y);
                internals.insert(1, x);
                Function {
                    internals,
                    call: |state, args, gi| {
                        let arg = if args.len() == 0 {
                            return Err(make_err("less than 1 argument"));
                        } else {
                            args[0].clone()
                        };
                        let f = gi(0).unwrap();
                        let n = match gi(1).unwrap().lock().unwrap().clone() {
                            Value::Number(n) => {
                                n
                            },
                            _ => return Err(make_err("data corrupted")),
                        };
                        if n < 1 {
                            return Err(make_err("data corrupted"))
                        }
                        let mut r = make_container(Value::Null);
                        let mut i = n;
                        while i > 0 {
                            r = call(state.clone(), f.clone(), vec![arg.clone()])?;
                            i -= 1;
                        }
                        Ok(r)
                    },
                    state: state.clone(),
                }
            };
            Ok(make_container(Value::Function(g)))
        },
        (Value::Boolean(a), Value::Boolean(b)) => Ok(make_container(Value::Boolean(a|b))),
        (Value::Object(obj), _) => call_metaprop(state.clone(), obj, vec![x, y], "multiply".to_string()),
        (_, Value::Object(obj)) => call_metaprop(state.clone(), obj, vec![x, y], "multiply".to_string()),
        _ => Err(make_err("can't multiply a with b")),
    }
}

pub fn negate(state: StateContainer, v: Container) -> Result<Container, Container> {
    let x = resolve_bind(state.clone(), v)?;
    let v = x.clone().lock().unwrap().clone();
    match v {
        Value::Number(n) => Ok(make_container(Value::Number(-n))),
        Value::Float(n) => Ok(make_container(Value::Float(-n))),
        Value::String(s) => Ok(make_container(Value::String(s.chars().rev().collect()))),
        Value::Boolean(b) => Ok(make_container(Value::Boolean(!b))),
        Value::Object(obj) => call_metaprop(state.clone(), obj, vec![x], "negate".to_string()),
        _ => Err(make_err("can't negate v")),
    }
}

pub fn subtract(state: StateContainer, a: Container, b: Container) -> Result<Container, Container> {
    let x = resolve_bind(state.clone(), a)?;
    let y = resolve_bind(state.clone(), b)?;
    let a = x.lock().unwrap().clone();
    let b = y.lock().unwrap().clone();
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => Ok(make_container(Value::Number(a-b))),
        (Value::Number(a), Value::Float(b)) => Ok(make_container(Value::Float((a as f64)-b))),
        (Value::Float(a), Value::Number(b)) => Ok(make_container(Value::Float(a-(b as f64)))),
        (Value::Float(a), Value::Float(b)) => Ok(make_container(Value::Float(a-b))),
        (Value::Boolean(a), Value::Boolean(b)) => Ok(make_container(Value::Boolean(a^b))),
        (Value::Object(obj), _) => call_metaprop(state.clone(), obj, vec![x, y], "subtract".to_string()),
        (_, Value::Object(obj)) => call_metaprop(state.clone(), obj, vec![x, y], "subtract".to_string()),
        _ => Err(make_err("can't subtract a and b")),
    }
}

pub fn has_base(state: StateContainer, obj: Container, prop: String) -> Result<bool, Container> {
    let obj = resolve_bind(state.clone(), obj)?.lock().unwrap().clone();
    match obj {
        Value::Object(obj) => {
            Ok(match obj.props.get(&prop) {
                Some(_) => true,
                None => false,
            })
        },
        _ => Err(make_err("can't see if obj has property prop")),
    }
}

pub fn has(state: StateContainer, obj: Container, prop: Container) -> Result<Container, Container> {
    has_base(state.clone(), obj, to_string_base(state.clone(), prop.clone())?).map(|b| make_container(Value::Boolean(b)))
}

pub fn set_base(state: StateContainer, obj: Container, prop: String, v: Container) -> Result<(), Container> {
    let a = if let Value::Object(o) = &mut *resolve_bind(state.clone(), obj)?.lock().unwrap() {
        match o.props.get(&prop) {
            Some(ObjectProp::GetSet(_, setter)) => {
                Ok(Some(setter.clone()))
            },
            _ => {
                o.props.insert(prop.to_string(), ObjectProp::Value(v.clone()));
                Ok(None)
            },
        }
    } else {
        Err(make_err("can't set property of non-object"))
    };
    let a = a?;
    if let Some(setter) = a {
        call(state.clone(), setter, vec![v]).map(|_| ())
    } else {
        Ok(())
    }
}

pub fn set(state: StateContainer, obj: Container, prop: Container, v: Container) -> Result<Container, Container> {
    set_base(state.clone(), obj, to_string_base(state.clone(), prop.clone())?, v.clone()).map(|_| make_container(Value::Null))
}

pub fn eql_base(state: StateContainer, a: Container, b: Container) -> Result<bool, Container> {
    let x = resolve_bind(state.clone(), a)?;
    let y = resolve_bind(state.clone(), b)?;
    let a = x.lock().unwrap().clone();
    let b = y.lock().unwrap().clone();
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => Ok(a==b),
        (Value::Number(a), Value::Float(b)) => Ok((a as f64)==b),
        (Value::Float(a), Value::Number(b)) => Ok(a==(b as f64)),
        (Value::Float(a), Value::Float(b)) => Ok(a==b),
        (Value::Null, Value::Null) => Ok(true),
        (Value::String(a), Value::String(b)) => Ok(a==b),
        (Value::Boolean(a), Value::Boolean(b)) => Ok(a==b),
        (Value::Object(_), Value::Object(_)) => Ok(Arc::ptr_eq(&x, &y)),
        (Value::Tuple(a), Value::Tuple(b)) => {
            let c = a.iter().zip(b.iter()).map(|(a, b)| {
                eql_base(state.clone(), a.clone(), b.clone())
            }).collect::<Vec<_>>();
            let mut d = Vec::new();
            for i in c {
                d.push(i?);
            }
            Ok(d.iter().all(|z| *z))
        },
        (Value::Function(_), Value::Function(_)) => Ok(Arc::ptr_eq(&x, &y)),
        _ => Ok(false),
    }
}

pub fn eql(state: StateContainer, a: Container, b: Container) -> Result<Container, Container> {
    Ok(make_container(Value::Boolean(eql_base(state, a, b)?)))
}

pub fn neql_base(state: StateContainer, a: Container, b: Container) -> Result<bool, Container> {
    Ok(!eql_base(state, a, b)?)
}

pub fn neql(state: StateContainer, a: Container, b: Container) -> Result<Container, Container> {
    Ok(make_container(Value::Boolean(neql_base(state, a, b)?)))
}

pub fn to_boolean_base(state: StateContainer, v: Container) -> Result<bool, Container> {
    let v = resolve_bind(state.clone(), v)?.lock().unwrap().clone();
    match v {
        Value::Number(n) => Ok(n != 0),
        Value::Float(f) => Ok(f != 0.0),
        Value::Null => Ok(false),
        Value::String(s) => Ok(s.len() != 0),
        Value::Boolean(b) => Ok(b),
        _ => Ok(true),
    }
}

pub fn to_boolean(state: StateContainer, v: Container) -> Result<Container, Container> {
    Ok(make_container(Value::Boolean(to_boolean_base(state, v)?)))
}

pub fn divide(state: StateContainer, a: Container, b: Container) -> Result<Container, Container> {
    let x = resolve_bind(state.clone(), a)?;
    let y = resolve_bind(state.clone(), b)?;
    let a = x.lock().unwrap().clone();
    let b = y.lock().unwrap().clone();
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => {
            if b == 0 {
                Err(make_err("cannot divide by 0"))
            } else {
                Ok(make_container(Value::Number(a/b)))
            }
        },
        (Value::Number(a), Value::Float(b)) => Ok(make_container(Value::Float((a as f64)/b))),
        (Value::Float(a), Value::Number(b)) => Ok(make_container(Value::Float(a/(b as f64)))),
        (Value::Float(a), Value::Float(b)) => Ok(make_container(Value::Float(a/b))),
        (Value::Boolean(a), Value::Boolean(b)) => Ok(make_container(Value::Boolean(!(a|b)))),
        (Value::Object(obj), _) => call_metaprop(state.clone(), obj, vec![x, y], "divide".to_string()),
        (_, Value::Object(obj)) => call_metaprop(state.clone(), obj, vec![x, y], "divide".to_string()),
        _ => Err(make_err("cannot divide a by b")),
    }
}

pub fn remainder(state: StateContainer, a: Container, b: Container) -> Result<Container, Container> {
    let x = resolve_bind(state.clone(), a)?;
    let y = resolve_bind(state.clone(), b)?;
    let a = x.lock().unwrap().clone();
    let b = y.lock().unwrap().clone();
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => Ok(make_container(Value::Number(a%b))),
        (Value::Number(a), Value::Float(b)) => Ok(make_container(Value::Float((a as f64)%b))),
        (Value::Float(a), Value::Number(b)) => Ok(make_container(Value::Float(a%(b as f64)))),
        (Value::Float(a), Value::Float(b)) => Ok(make_container(Value::Float(a%b))),
        (Value::Number(n), Value::Function(_)) => {
            if n < 1 {
                return Err(make_container(Value::String("cannot take the remainder of a by b".to_string())))
            }
            let g = {
                let mut internals = HashMap::new();
                internals.insert(0, y);
                internals.insert(1, x);
                Function {
                    internals,
                    call: |state, args, gi| {
                        let f = gi(0).unwrap();
                        let n = match gi(1).unwrap().lock().unwrap().clone() {
                            Value::Number(n) => n,
                            _ => return Err(make_container(Value::String("data corrupted".to_string())))
                        };
                        for _ in 0..n {
                            call(state.clone(), f.clone(), args.clone())?;
                        }
                        Ok(make_container(Value::Null))
                    },
                    state: state.clone()
                }
            };
            Ok(make_container(Value::Function(g)))
        },
        (Value::Function(_), Value::Number(n)) => {
            if n < 1 {
                return Err(make_container(Value::String("cannot take the remainder of a by b".to_string())))
            }
            let g = {
                let mut internals = HashMap::new();
                internals.insert(0, x);
                internals.insert(1, y);
                Function {
                    internals,
                    call: |state, args, gi| {
                        let f = gi(0).unwrap();
                        let n = match gi(1).unwrap().lock().unwrap().clone() {
                            Value::Number(n) => n,
                            _ => return Err(make_container(Value::String("data corrupted".to_string())))
                        };
                        for _ in 0..n {
                            call(state.clone(), f.clone(), args.clone())?;
                        }
                        Ok(make_container(Value::Null))
                    },
                    state: state.clone()
                }
            };
            Ok(make_container(Value::Function(g)))
        },
        (Value::Object(obj), _) => call_metaprop(state.clone(), obj, vec![x, y], "remainder".to_string()),
        (_, Value::Object(obj)) => call_metaprop(state.clone(), obj, vec![x, y], "remainder".to_string()),
        _ => Err(make_container(Value::String("cannot take the remainder of a by b".to_string()))),
    }
}

pub fn to_number_base(state: StateContainer, v: Container) -> Result<i64, Container> {
    let v = resolve_bind(state.clone(), v)?.lock().unwrap().clone();
    match v {
        Value::Number(n) => Ok(n),
        Value::Float(f) => Ok(f as i64),
        Value::Null => Ok(0),
        Value::String(s) => s.parse().map_err(|_| make_container(Value::String("cannot convert v to number".to_string()))),
        Value::Boolean(b) => Ok(if b {1} else {0}),
        _ => Err(make_container(Value::String("cannot convert v to number".to_string())))
    }
}

pub fn make_object_base() -> Object {
    Object {
        props: HashMap::new(),
        internals: HashMap::new(),
        metaobj: make_container(Value::Null),
        externals: HashMap::new(),
    }
}

pub fn make_object() -> Container {
    make_container(Value::Object(make_object_base()))
}

pub fn call_prop(state: StateContainer, obj: Container, args: Vec<Container>, prop: String) -> Result<Container, Container> {
    let f = get_base(state.clone(), obj, prop)?;
    call(state.clone(), f, args)
}

pub fn not(state: StateContainer, v: Container) -> Result<Container, Container> {
    let x = resolve_bind(state.clone(), v)?;
    let v = x.clone().lock().unwrap().clone();
    match v {
        Value::Number(n) => Ok(make_container(Value::Number(!n))),
        Value::Float(n) => Ok(make_container(Value::Number(!(n as i64)))),
        Value::Boolean(b) => Ok(make_container(Value::Boolean(!b))),
        Value::Object(obj) => call_metaprop(state.clone(), obj, vec![x], "not".to_string()),
        _ => Err(make_err("can't not v")),
    }
}

pub fn gt(state: StateContainer, a: Container, b: Container) -> Result<Container, Container> {
    let x = resolve_bind(state.clone(), a)?;
    let y = resolve_bind(state.clone(), b)?;
    let a = x.lock().unwrap().clone();
    let b = y.lock().unwrap().clone();
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => Ok(make_container(Value::Boolean(a > b))),
        (Value::Number(a), Value::Float(b)) => Ok(make_container(Value::Boolean((a as f64) > b))),
        (Value::Float(a), Value::Number(b)) => Ok(make_container(Value::Boolean(a > (b as f64)))),
        (Value::Float(a), Value::Float(b)) => Ok(make_container(Value::Boolean(a > b))),
        (Value::Object(obj), _) => call_metaprop(state.clone(), obj, vec![x, y], "gt".to_string()),
        (_, Value::Object(obj)) => call_metaprop(state.clone(), obj, vec![x, y], "gt".to_string()),
        _ => Err(make_err("cannot compare a and b")),
    }
}

pub fn ge(state: StateContainer, a: Container, b: Container) -> Result<Container, Container> {
    let x = resolve_bind(state.clone(), a)?;
    let y = resolve_bind(state.clone(), b)?;
    let a = x.lock().unwrap().clone();
    let b = y.lock().unwrap().clone();
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => Ok(make_container(Value::Boolean(a >= b))),
        (Value::Number(a), Value::Float(b)) => Ok(make_container(Value::Boolean((a as f64) >= b))),
        (Value::Float(a), Value::Number(b)) => Ok(make_container(Value::Boolean(a >= (b as f64)))),
        (Value::Float(a), Value::Float(b)) => Ok(make_container(Value::Boolean(a >= b))),
        (Value::Object(obj), _) => call_metaprop(state.clone(), obj, vec![x, y], "ge".to_string()),
        (_, Value::Object(obj)) => call_metaprop(state.clone(), obj, vec![x, y], "ge".to_string()),
        _ => Err(make_err("cannot compare a and b")),
    }
}

pub fn lt(state: StateContainer, a: Container, b: Container) -> Result<Container, Container> {
    let x = resolve_bind(state.clone(), a)?;
    let y = resolve_bind(state.clone(), b)?;
    let a = x.lock().unwrap().clone();
    let b = y.lock().unwrap().clone();
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => Ok(make_container(Value::Boolean(a < b))),
        (Value::Number(a), Value::Float(b)) => Ok(make_container(Value::Boolean((a as f64) < b))),
        (Value::Float(a), Value::Number(b)) => Ok(make_container(Value::Boolean(a < (b as f64)))),
        (Value::Float(a), Value::Float(b)) => Ok(make_container(Value::Boolean(a < b))),
        (Value::Object(obj), _) => call_metaprop(state.clone(), obj, vec![x, y], "lt".to_string()),
        (_, Value::Object(obj)) => call_metaprop(state.clone(), obj, vec![x, y], "lt".to_string()),
        _ => Err(make_err("cannot compare a and b")),
    }
}

pub fn le(state: StateContainer, a: Container, b: Container) -> Result<Container, Container> {
    let x = resolve_bind(state.clone(), a)?;
    let y = resolve_bind(state.clone(), b)?;
    let a = x.lock().unwrap().clone();
    let b = y.lock().unwrap().clone();
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => Ok(make_container(Value::Boolean(a <= b))),
        (Value::Number(a), Value::Float(b)) => Ok(make_container(Value::Boolean((a as f64) <= b))),
        (Value::Float(a), Value::Number(b)) => Ok(make_container(Value::Boolean(a <= (b as f64)))),
        (Value::Float(a), Value::Float(b)) => Ok(make_container(Value::Boolean(a <= b))),
        (Value::Object(obj), _) => call_metaprop(state.clone(), obj, vec![x, y], "le".to_string()),
        (_, Value::Object(obj)) => call_metaprop(state.clone(), obj, vec![x, y], "le".to_string()),
        _ => Err(make_err("cannot compare a and b")),
    }
}

pub fn and(state: StateContainer, a: Container, b: Container) -> Result<Container, Container> {
    let x = resolve_bind(state.clone(), a)?;
    let y = resolve_bind(state.clone(), b)?;
    let a = x.lock().unwrap().clone();
    let b = y.lock().unwrap().clone();
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => Ok(make_container(Value::Number(a & b))),
        (Value::Number(a), Value::Float(b)) => Ok(make_container(Value::Number(a & (b as i64)))),
        (Value::Float(a), Value::Number(b)) => Ok(make_container(Value::Number((a as i64) & b))),
        (Value::Float(a), Value::Float(b)) => Ok(make_container(Value::Number((a as i64) & (b as i64)))),
        (Value::Boolean(a), Value::Boolean(b)) => Ok(make_container(Value::Boolean(a & b))),
        (Value::Object(obj), _) => call_metaprop(state.clone(), obj, vec![x, y], "and".to_string()),
        (_, Value::Object(obj)) => call_metaprop(state.clone(), obj, vec![x, y], "and".to_string()),
        _ => Err(make_err("cannot compare a and b")),
    }
}

pub fn or(state: StateContainer, a: Container, b: Container) -> Result<Container, Container> {
    let x = resolve_bind(state.clone(), a)?;
    let y = resolve_bind(state.clone(), b)?;
    let a = x.lock().unwrap().clone();
    let b = y.lock().unwrap().clone();
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => Ok(make_container(Value::Number(a | b))),
        (Value::Number(a), Value::Float(b)) => Ok(make_container(Value::Number(a | (b as i64)))),
        (Value::Float(a), Value::Number(b)) => Ok(make_container(Value::Number((a as i64) | b))),
        (Value::Float(a), Value::Float(b)) => Ok(make_container(Value::Number((a as i64) | (b as i64)))),
        (Value::Boolean(a), Value::Boolean(b)) => Ok(make_container(Value::Boolean(a | b))),
        (Value::Object(obj), _) => call_metaprop(state.clone(), obj, vec![x, y], "or".to_string()),
        (_, Value::Object(obj)) => call_metaprop(state.clone(), obj, vec![x, y], "or".to_string()),
        _ => Err(make_err("cannot compare a and b")),
    }
}

pub fn xor(state: StateContainer, a: Container, b: Container) -> Result<Container, Container> {
    let x = resolve_bind(state.clone(), a)?;
    let y = resolve_bind(state.clone(), b)?;
    let a = x.lock().unwrap().clone();
    let b = y.lock().unwrap().clone();
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => Ok(make_container(Value::Number(a ^ b))),
        (Value::Number(a), Value::Float(b)) => Ok(make_container(Value::Number(a ^ (b as i64)))),
        (Value::Float(a), Value::Number(b)) => Ok(make_container(Value::Number((a as i64) ^ b))),
        (Value::Float(a), Value::Float(b)) => Ok(make_container(Value::Number((a as i64) ^ (b as i64)))),
        (Value::Boolean(a), Value::Boolean(b)) => Ok(make_container(Value::Boolean(a ^ b))),
        (Value::Object(obj), _) => call_metaprop(state.clone(), obj, vec![x, y], "xor".to_string()),
        (_, Value::Object(obj)) => call_metaprop(state.clone(), obj, vec![x, y], "xor".to_string()),
        _ => Err(make_err("cannot compare a and b")),
    }
}

pub fn make_function(state: StateContainer, instrs: Vec<Instruction>, s: Option<StateContainer>) -> Result<Container, Container> {
    let mut obj = make_object_base();
    obj.externals.insert(0, Arc::new(Mutex::new(Box::new(instrs.clone()))));
    let mut internals = HashMap::new();
    internals.insert(0, make_container(Value::Object(obj)));
    let s = match s {
        Some(s) => s.clone(),
        None => new_state(state.clone()),
    };
    Ok(make_container(Value::Function(Function {
        internals,
        call: |state, args, gi| {
            let instrs = gi(0).unwrap();
            let instrs = match instrs.lock().unwrap().clone() {
                Value::Object(obj) => Some(obj),
                _ => None,
            };
            let instrs = instrs.unwrap().externals;
            let instrs = instrs[&0].lock().unwrap().downcast_ref::<Vec<Instruction>>().unwrap().clone();
            let mut tmps: HashMap<u64, Container> = HashMap::new();
            let r = interpret_instructions(state, &args, &mut tmps, &instrs, None)?;
            match r {
                (Some(r), _) => Ok(r),
                (None, _) => Ok(make_container(Value::Null)),
            }
        },
        state: s,
    })))
}

pub fn new_state(state: StateContainer) -> StateContainer {
    Arc::new(Mutex::new(State {
        scope: make_object(),
        parent: Some(state.clone()),
        global: {
            state.lock().unwrap().global.clone()
        },
        globaldata: {
            state.lock().unwrap().globaldata.clone()
        },
        debug: {
            state.lock().unwrap().debug
        },
    }))
}

pub fn get_from_state(ident: String, state: StateContainer) -> Result<Container, Container> {
    let obj = {
        state.lock().unwrap().scope.clone()
    };
    if has_base(state.clone(), obj.clone(), ident.clone())? {
        get_base(state.clone(), obj.clone(), ident.clone())
    } else {
        let parent = state.lock().unwrap().parent.clone();
        match parent {
            Some(s) => get_from_state(ident.clone(), s),
            None => Ok(make_container(Value::Null)),
        }
    }
}

pub fn set_to_state(ident: String, v: Container, state: StateContainer) -> Result<(), Container> {
    let obj = {
        state.lock().unwrap().scope.clone()
    };
    if has_base(state.clone(), obj.clone(), ident.clone())? {
        set_base(state.clone(), obj.clone(), ident.clone(), v.clone())
    } else {
        let parent = state.lock().unwrap().parent.clone();
        match parent {
            Some(s) => set_to_state(ident.clone(), v.clone(), s),
            None => set_base(state.clone(), state.lock().unwrap().scope.clone(), ident.clone(), v.clone())
        }
    }
}

pub fn get_var(state: StateContainer, args: &Vec<Container>, tmps: &HashMap<u64, Container>, vari: VarIndex) -> Result<Container, Container> {
    match vari {
        VarIndex::Arg(i) => Ok(match args.get(i) {
            Some(v) => v.clone(),
            None => make_container(Value::Null),
        }),
        VarIndex::Ident(ident) => get_from_state(ident, state.clone()),
        VarIndex::Temp(u) => Ok(match tmps.get(&u) {
            Some(v) => v.clone(),
            None => make_container(Value::Null),
        }),
    }
}

pub fn set_var(state: StateContainer, tmps: &mut HashMap<u64, Container>, vari: VarIndex, v: Container) -> Result<(), Container> {
    match vari {
        VarIndex::Arg(_) => Err(make_container(Value::String("can't set argument".to_string()))),
        VarIndex::Ident(ident) => set_to_state(ident.clone(), v.clone(), state.clone()),
        VarIndex::Temp(u) => {
            tmps.insert(u, v.clone());
            Ok(())
        },
    }
}

pub fn decl(state: StateContainer, vari: VarIndex) -> Result<(), Container> {
    let obj = {
        state.lock().unwrap().scope.clone()
    };
    match vari {
        VarIndex::Ident(ident) => set_base(state.clone(), obj.clone(), ident.clone(), make_container(Value::Null)),
        _ => Err(make_container(Value::String("can't declare non-identifier".to_string()))),
    }
}

pub fn make_tuple(values: Vec<Container>) -> Container {
    make_container(Value::Tuple(values.clone()))
}

pub fn detuple(state: StateContainer, v: Container) -> Result<Vec<Container>, Container> {
    let x = resolve_bind(state.clone(), v)?;
    let v = x.lock().unwrap().clone();
    match v {
        Value::Tuple(v) => Ok(v),
        _ => Ok(vec![x]),
    }
}

pub fn interpret_instructions(state: StateContainer, args: &Vec<Container>, tmps: &mut HashMap<u64, Container>, instrs: &Vec<Instruction>, pipeshort: Option<Container>) -> Result<(Option<Container>, Option<Label>), Container> {
    let mut ulabels: HashMap<u64, usize> = HashMap::new();
    let mut slabels: HashMap<String, usize> = HashMap::new();
    let mut i = 0;
    while i < instrs.len() {
        match instrs[i].clone() {
            Instruction::Label(l) => {
                match l {
                    Label::Unnamed(u) => ulabels.insert(u, i),
                    Label::Named(s) => slabels.insert(s.clone(), i),
                };
            },
            _ => {},
        }
        i += 1;
    }
    let mut defers: Vec<Instruction> = Vec::new();
    let mut pipeshort = match pipeshort {
        Some(v) => v.clone(),
        None => make_container(Value::Null),
    };
    i = 0;
    while i < instrs.len() {
        match instrs[i].clone() {
            Instruction::Return(vi) => {
                if defers.len() > 0 {
                    interpret_instructions(state.clone(), args, tmps, &defers, None)?;
                }
                return Ok((Some(get_var(state.clone(), args, tmps, vi.clone())?), None))
            },
            Instruction::Throw(vi) => {
                if defers.len() > 0 {
                    interpret_instructions(state.clone(), args, tmps, &defers, None)?;
                }
                return Err(get_var(state.clone(), args, tmps, vi.clone())?)
            },
            Instruction::Add(res, op1, op2) => {
                let op1 = get_var(state.clone(), args, tmps, op1.clone())?;
                let op2 = get_var(state.clone(), args, tmps, op2.clone())?;
                let r = add(state.clone(), op1.clone(), op2.clone())?;
                set_var(state.clone(), tmps, res.clone(), r.clone())?;
            },
            Instruction::Call(res, f, ops) => {
                let ops = ops.iter().map(|i| get_var(state.clone(), args, tmps, i.clone())).collect::<Vec<_>>();
                let mut ops2 = Vec::new();
                for i in ops {
                    ops2.push(i?);
                }
                let ops = ops2;
                let f = get_var(state.clone(), args, tmps, f.clone())?;
                let r = call(state.clone(), f, ops)?;
                set_var(state.clone(), tmps, res.clone(), r.clone())?;
            },
            Instruction::Get(res, obj, prop) => {
                let obj = get_var(state.clone(), args, tmps, obj.clone())?;
                let prop = get_var(state.clone(), args, tmps, prop.clone())?;
                let r = get(state.clone(), obj.clone(), prop.clone())?;
                set_var(state.clone(), tmps, res.clone(), r.clone())?;
            },
            Instruction::Multiply(res, op1, op2) => {
                let op1 = get_var(state.clone(), args, tmps, op1.clone())?;
                let op2 = get_var(state.clone(), args, tmps, op2.clone())?;
                let r = multiply(state.clone(), op1.clone(), op2.clone())?;
                set_var(state.clone(), tmps, res.clone(), r.clone())?;
            },
            Instruction::Negate(res, op) => {
                let op = get_var(state.clone(), args, tmps, op.clone())?;
                let r = negate(state.clone(), op.clone())?;
                set_var(state.clone(), tmps, res.clone(), r.clone())?;
            },
            Instruction::Subtract(res, op1, op2) => {
                let op1 = get_var(state.clone(), args, tmps, op1.clone())?;
                let op2 = get_var(state.clone(), args, tmps, op2.clone())?;
                let r = subtract(state.clone(), op1.clone(), op2.clone())?;
                set_var(state.clone(), tmps, res.clone(), r.clone())?;
            },
            Instruction::Has(res, obj, prop) => {
                let obj = get_var(state.clone(), args, tmps, obj.clone())?;
                let prop = get_var(state.clone(), args, tmps, prop.clone())?;
                let r = has(state.clone(), obj.clone(), prop.clone())?;
                set_var(state.clone(), tmps, res.clone(), r.clone())?;
            },
            Instruction::Set(res, obj, prop, value) => {
                let obj = get_var(state.clone(), args, tmps, obj.clone())?;
                let prop = get_var(state.clone(), args, tmps, prop.clone())?;
                let value = get_var(state.clone(), args, tmps, value.clone())?;
                let r = set(state.clone(), obj.clone(), prop.clone(), value.clone())?;
                set_var(state.clone(), tmps, res.clone(), r.clone())?;
            },
            Instruction::Decl(op) => {
                decl(state.clone(), op.clone())?;
            },
            Instruction::Label(_) => {},
            Instruction::Goto(l) => {
                let opt = match l.clone() {
                    Label::Unnamed(u) => ulabels.get(&u),
                    Label::Named(s) => slabels.get(&s),
                };
                match opt {
                    Some(u) => i = *u,
                    None => {
                        if defers.len() > 0 {
                            interpret_instructions(state.clone(), args, tmps, &defers, None)?;
                        }
                        return Ok((None, Some(l.clone())))
                    },
                };
            },
            Instruction::Eql(res, op1, op2) => {
                let op1 = get_var(state.clone(), args, tmps, op1.clone())?;
                let op2 = get_var(state.clone(), args, tmps, op2.clone())?;
                let r = eql(state.clone(), op1.clone(), op2.clone())?;
                set_var(state.clone(), tmps, res.clone(), r.clone())?;
            },
            Instruction::Neql(res, op1, op2) => {
                let op1 = get_var(state.clone(), args, tmps, op1.clone())?;
                let op2 = get_var(state.clone(), args, tmps, op2.clone())?;
                let r = neql(state.clone(), op1.clone(), op2.clone())?;
                set_var(state.clone(), tmps, res.clone(), r.clone())?;
            },
            Instruction::GotoIf(l, op) => {
                let op = get_var(state.clone(), args, tmps, op.clone())?;
                if to_boolean_base(state.clone(), op)? {
                    let opt = match l.clone() {
                        Label::Unnamed(u) => ulabels.get(&u),
                        Label::Named(s) => slabels.get(&s),
                    };
                    match opt {
                        Some(u) => i = *u,
                        None => {
                            if defers.len() > 0 {
                                interpret_instructions(state.clone(), args, tmps, &defers, None)?;
                            }
                            return Ok((None, Some(l.clone())))
                        },
                    };
                }
            },
            Instruction::Block(instvec) => {
                let s = new_state(state.clone());
                let r = interpret_instructions(s, args, tmps, &instvec, Some(pipeshort.clone()))?;
                match r {
                    (Some(v), l) => {
                        if defers.len() > 0 {
                            interpret_instructions(state.clone(), args, tmps, &defers, None)?;
                        }
                        return Ok((Some(v), l))
                    },
                    (None, Some(l)) => {
                        let opt = match l.clone() {
                            Label::Unnamed(u) => ulabels.get(&u),
                            Label::Named(s) => slabels.get(&s),
                        };
                        match opt {
                            Some(u) => i = *u,
                            None => {
                                if defers.len() > 0 {
                                    interpret_instructions(state.clone(), args, tmps, &defers, None)?;
                                }
                                return Ok((None, Some(l.clone())))
                            },
                        };
                    },
                    _ => {},
                };
            },
            Instruction::MakeTuple(res, ops) => {
                let ops = ops.iter().map(|i| get_var(state.clone(), args, tmps, i.clone())).collect::<Vec<_>>();
                let mut ops2 = Vec::new();
                for i in ops {
                    ops2.push(i?);
                }
                let ops = ops2;
                let r = make_tuple(ops);
                set_var(state.clone(), tmps, res.clone(), r.clone())?;
            },
            Instruction::DeTuple(results, op) => {
                let op = get_var(state.clone(), args, tmps, op.clone())?;
                let r = detuple(state.clone(), op)?;
                let mut i = 0;
                while i < results.len() {
                    let r = match r.get(i) {
                        Some(v) => v.clone(),
                        None => make_container(Value::Null),
                    };
                    set_var(state.clone(), tmps, results[i].clone(), r)?;
                    i += 1;
                }
            },
            Instruction::Divide(res, op1, op2) => {
                let op1 = get_var(state.clone(), args, tmps, op1.clone())?;
                let op2 = get_var(state.clone(), args, tmps, op2.clone())?;
                let r = divide(state.clone(), op1.clone(), op2.clone())?;
                set_var(state.clone(), tmps, res.clone(), r.clone())?;
            },
            Instruction::Remainder(res, op1, op2) => {
                let op1 = get_var(state.clone(), args, tmps, op1.clone())?;
                let op2 = get_var(state.clone(), args, tmps, op2.clone())?;
                let r = remainder(state.clone(), op1.clone(), op2.clone())?;
                set_var(state.clone(), tmps, res.clone(), r.clone())?;
            },
            Instruction::MakeBind(res, f) => {
                let f = get_var(state.clone(), args, tmps, f)?;
                let r = make_container(Value::Bind(f));
                set_var(state.clone(), tmps, res, r)?;
            },
            Instruction::Catch(erri, err, instvec) => {
                let s = new_state(state.clone());
                let r = interpret_instructions(s, args, tmps, &instvec, Some(pipeshort.clone()));
                match r {
                    Err(e) => {
                        set_var(state.clone(), tmps, erri, make_container(Value::Boolean(true)))?;
                        set_var(state.clone(), tmps, err, e)?;
                    },
                    Ok(r) => {
                        set_var(state.clone(), tmps, erri, make_container(Value::Boolean(false)))?;
                        match r {
                            (Some(v), l) => return Ok((Some(v), l)),
                            (None, Some(l)) => {
                                        let opt = match l.clone() {
                                    Label::Unnamed(u) => ulabels.get(&u),
                                    Label::Named(s) => slabels.get(&s),
                                };
                                match opt {
                                    Some(u) => i = *u,
                                    None => {
                                        if defers.len() > 0 {
                                            interpret_instructions(state.clone(), args, tmps, &defers, None)?;
                                        }
                                        return Ok((None, Some(l.clone())))
                                    },
                                };
                            },
                            _ => {},
                        }
                    },
                }
            },
            Instruction::Assign(r, op) => {
                let op = get_var(state.clone(), args, tmps, op.clone())?;
                set_var(state.clone(), tmps, r.clone(), op)?;
            },
            Instruction::Defer(v) => {
                defers.extend(v.iter().map(|v| v.clone()));
            },
            Instruction::Boolean(res, op) => {
                let op = make_container(Value::Boolean(op));
                set_var(state.clone(), tmps, res.clone(), op)?;
            },
            Instruction::Number(res, op) => {
                let op = make_container(Value::Number(op));
                set_var(state.clone(), tmps, res.clone(), op)?;
            },
            Instruction::Float(res, op) => {
                let op = make_container(Value::Float(op));
                set_var(state.clone(), tmps, res.clone(), op)?;
            },
            Instruction::String(res, op) => {
                let op = make_container(Value::String(op));
                set_var(state.clone(), tmps, res.clone(), op)?;
            },
            Instruction::MakeFunction(res, body) => {
                let f = make_function(state.clone(), body, None)?;
                set_var(state.clone(), tmps, res, f)?;
            },
            Instruction::Not(res, op) => {
                let op = get_var(state.clone(), args, tmps, op)?;
                let r = not(state.clone(), op)?;
                set_var(state.clone(), tmps, res, r)?;
            },
            Instruction::Gt(res, op1, op2) => {
                let op1 = get_var(state.clone(), args, tmps, op1.clone())?;
                let op2 = get_var(state.clone(), args, tmps, op2.clone())?;
                let r = gt(state.clone(), op1.clone(), op2.clone())?;
                set_var(state.clone(), tmps, res.clone(), r.clone())?;
            },
            Instruction::Ge(res, op1, op2) => {
                let op1 = get_var(state.clone(), args, tmps, op1.clone())?;
                let op2 = get_var(state.clone(), args, tmps, op2.clone())?;
                let r = ge(state.clone(), op1.clone(), op2.clone())?;
                set_var(state.clone(), tmps, res.clone(), r.clone())?;
            },
            Instruction::Lt(res, op1, op2) => {
                let op1 = get_var(state.clone(), args, tmps, op1.clone())?;
                let op2 = get_var(state.clone(), args, tmps, op2.clone())?;
                let r = lt(state.clone(), op1.clone(), op2.clone())?;
                set_var(state.clone(), tmps, res.clone(), r.clone())?;
            },
            Instruction::Le(res, op1, op2) => {
                let op1 = get_var(state.clone(), args, tmps, op1.clone())?;
                let op2 = get_var(state.clone(), args, tmps, op2.clone())?;
                let r = le(state.clone(), op1.clone(), op2.clone())?;
                set_var(state.clone(), tmps, res.clone(), r.clone())?;
            },
            Instruction::And(res, op1, op2) => {
                let op1 = get_var(state.clone(), args, tmps, op1.clone())?;
                let op2 = get_var(state.clone(), args, tmps, op2.clone())?;
                let r = and(state.clone(), op1.clone(), op2.clone())?;
                set_var(state.clone(), tmps, res.clone(), r.clone())?;
            },
            Instruction::Or(res, op1, op2) => {
                let op1 = get_var(state.clone(), args, tmps, op1.clone())?;
                let op2 = get_var(state.clone(), args, tmps, op2.clone())?;
                let r = or(state.clone(), op1.clone(), op2.clone())?;
                set_var(state.clone(), tmps, res.clone(), r.clone())?;
            },
            Instruction::Xor(res, op1, op2) => {
                let op1 = get_var(state.clone(), args, tmps, op1.clone())?;
                let op2 = get_var(state.clone(), args, tmps, op2.clone())?;
                let r = xor(state.clone(), op1.clone(), op2.clone())?;
                set_var(state.clone(), tmps, res.clone(), r.clone())?;
            },
            Instruction::GetPipeShorthand(res) => {
                set_var(state.clone(), tmps, res, pipeshort.clone())?;
            },
            Instruction::SetPipeShorthand(op) => {
                pipeshort = get_var(state.clone(), args, tmps, op)?;
            },
        }
        i += 1;
    }
    if defers.len() > 0 {
        interpret_instructions(state.clone(), args, tmps, &defers, None)?;
    }
    Ok((None, None))
}