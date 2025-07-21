use std::{collections::HashMap, sync::Arc, pin::Pin};

use tokio::sync::Mutex;

use crate::{make_container, make_err, opfn, Container, Function, Gi, Instruction, Label, Object, ObjectProp, Operator, State, StateContainer, Value, VarIndex};

// TODO: add comments

pub async fn resolve_bind(state: StateContainer, v: Container) -> Result<Container, Container> {
    let b = v.lock().await.clone();
    match b {
        Value::Bind(b) => {
            let b = Box::pin(call(state.clone(), b, Vec::new())).await?;
            Box::pin(resolve_bind(state, b)).await
        },
        _ => Ok(v)
    }
}

pub async fn add(state: StateContainer, a: Container, b: Container) -> Result<Container, Container> {
    let x = resolve_bind(state.clone(), a).await?;
    let y = resolve_bind(state.clone(), b).await?;
    let a = x.clone().lock().await.clone();
    let b = y.clone().lock().await.clone();
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
                        Box::pin(async move {
                            let f = gi(0).unwrap();
                            let g = gi(1).unwrap();
                            let r = call(state.clone(), f, args).await?;
                            call(state.clone(), g, vec![r]).await
                        })
                    },
                    state: state.clone(),

                }
            };
            Ok(make_container(Value::Function(h)))
        },
        (Value::Boolean(a), Value::Boolean(b)) => Ok(make_container(Value::Boolean(a&b))),
        (Value::Object(obj), _) => {
            call_metaprop(state.clone(), obj, vec![x, y], "add".to_string()).await
        },
        (_, Value::Object(obj)) => {
            call_metaprop(state.clone(), obj, vec![x, y], "add".to_string()).await
        },
        _ => Err(make_err("can't add a with b")),
    }
}

pub async fn call(state: StateContainer, f: Container, args: Vec<Container>) -> Result<Container, Container> {
    let g = resolve_bind(state.clone(), f).await?;
    let f = g.clone().lock().await.clone();
    match f {
        Value::Function(f) => {
            let gi = {
                let i = f.internals.clone();
                move |v: u64| {i.get(&v).map(|s| s.clone())}
            };
            (f.call)(f.state.clone(), args, Arc::new(gi)).await
        },
        Value::Object(obj) => {
            call_metaprop(state.clone(), obj, vec![g].iter().chain(args.iter()).map(|h| h.clone()).collect::<Vec<Container>>(), "call".to_string()).await
        },
        _ => Err(make_err("can't call f")),
    }
}

pub async fn to_string_base(state: StateContainer, v: Container) -> Result<String, Container> {
    let x = resolve_bind(state.clone(), v).await?;
    let v = x.clone().lock().await.clone();
    match v {
        Value::String(v) => Ok(v),
        Value::Number(v) => Ok(v.to_string()),
        Value::Float(v) => Ok(v.to_string()),
        Value::Null => Ok("null".to_string()),
        Value::Boolean(v) => Ok(v.to_string()),
        Value::Object(obj) => {
            let tmp = call_metaprop(state.clone(), obj, vec![x], "to_string".to_string()).await?;
            Box::pin(to_string_base(state.clone(), tmp)).await
        },
        Value::Tuple(t) => {
            let mut s = Vec::new();
            for i in t.iter() {
                s.push(Box::pin(to_string_base(state.clone(), i.clone())).await?);
            }
            Ok("(".to_string()+&s.join(", ")+")")
        },
        _ => Err(make_err("can't convert v to string")),
    }
}

pub async fn to_string(state: StateContainer, v: Container) -> Result<Container, Container> {
    Ok(make_container(Value::String(to_string_base(state, v).await?)))
}

pub async fn get_base(state: StateContainer, obj: Container, prop: String) -> Result<Container, Container> {
    let o = resolve_bind(state.clone(), obj).await?;
    let obj = o.lock().await.clone();
    match obj {
        Value::Object(obj) => {
            let p = obj.props.get(&prop);
            match p {
                None => match call_metaprop(state.clone(), obj, vec![o, make_container(Value::String(prop.clone()))], "get".to_string()).await {
                    Ok(v) => Ok(v),
                    Err(_) => Ok(make_container(Value::Null)),
                },
                Some(prop) => {
                    match prop {
                        ObjectProp::Value(prop) => Ok(prop.clone()),
                        ObjectProp::GetSet(getter, _) => {
                            call(state, getter.clone(), Vec::new()).await
                        },
                    }
                },
            }
        },
        _ => Err(make_err("can't get property prop on object obj")),
    }
}

pub async fn get(state: StateContainer, obj: Container, prop: Container) -> Result<Container, Container> {
    get_base(state.clone(), obj, to_string_base(state.clone(), prop).await?).await
}

pub async fn call_metaprop(state: StateContainer, obj: Object, args: Vec<Container>, prop: String) -> Result<Container, Container> {
    let metaobj = obj.metaobj;
    let f = Box::pin(get_base(state.clone(), metaobj, prop)).await?;
    Box::pin(call(state, f, args)).await
}

pub async fn multiply(state: StateContainer, a: Container, b: Container) -> Result<Container, Container> {
    let x = resolve_bind(state.clone(), a).await?;
    let y = resolve_bind(state.clone(), b).await?;
    let a = x.clone().lock().await.clone();
    let b = y.clone().lock().await.clone();
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
                        Box::pin(async move {
                            let arg = if args.len() == 0 {
                                return Err(make_err("less than 1 argument"));
                            } else {
                                args[0].clone()
                            };
                            let f = gi(0).unwrap();
                            let n = match gi(1).unwrap().lock().await.clone() {
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
                                r = call(state.clone(), f.clone(), vec![arg.clone()]).await?;
                                i -= 1;
                            }
                            Ok(r)
                        })
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
                        Box::pin(async move {
                            let arg = if args.len() == 0 {
                                return Err(make_err("less than 1 argument"));
                            } else {
                                args[0].clone()
                            };
                            let f = gi(0).unwrap();
                            let n = match gi(1).unwrap().lock().await.clone() {
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
                                r = call(state.clone(), f.clone(), vec![arg.clone()]).await?;
                                i -= 1;
                            }
                            Ok(r)
                        })
                    },
                    state: state.clone(),
                }
            };
            Ok(make_container(Value::Function(g)))
        },
        (Value::Boolean(a), Value::Boolean(b)) => Ok(make_container(Value::Boolean(a|b))),
        (Value::Object(obj), _) => call_metaprop(state.clone(), obj, vec![x, y], "multiply".to_string()).await,
        (_, Value::Object(obj)) => call_metaprop(state.clone(), obj, vec![x, y], "multiply".to_string()).await,
        _ => Err(make_err("can't multiply a with b")),
    }
}

pub async fn negate(state: StateContainer, v: Container) -> Result<Container, Container> {
    let x = resolve_bind(state.clone(), v).await?;
    let v = x.clone().lock().await.clone();
    match v {
        Value::Number(n) => Ok(make_container(Value::Number(-n))),
        Value::Float(n) => Ok(make_container(Value::Float(-n))),
        Value::String(s) => Ok(make_container(Value::String(s.chars().rev().collect()))),
        Value::Boolean(b) => Ok(make_container(Value::Boolean(!b))),
        Value::Object(obj) => call_metaprop(state.clone(), obj, vec![x], "negate".to_string()).await,
        _ => Err(make_err("can't negate v")),
    }
}

pub async fn subtract(state: StateContainer, a: Container, b: Container) -> Result<Container, Container> {
    let x = resolve_bind(state.clone(), a).await?;
    let y = resolve_bind(state.clone(), b).await?;
    let a = x.lock().await.clone();
    let b = y.lock().await.clone();
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => Ok(make_container(Value::Number(a-b))),
        (Value::Number(a), Value::Float(b)) => Ok(make_container(Value::Float((a as f64)-b))),
        (Value::Float(a), Value::Number(b)) => Ok(make_container(Value::Float(a-(b as f64)))),
        (Value::Float(a), Value::Float(b)) => Ok(make_container(Value::Float(a-b))),
        (Value::Boolean(a), Value::Boolean(b)) => Ok(make_container(Value::Boolean(a^b))),
        (Value::Function(_), Value::Function(_)) => {
            let h = {
                let mut internals = HashMap::new();
                internals.insert(0, x);
                internals.insert(1, y);
                Function {
                    internals,
                    call: |state, args, gi| {
                        Box::pin(async move {
                            let f = gi(0).unwrap();
                            let g = gi(1).unwrap();
                            let r = call(state.clone(), f, args).await?;
                            call(state.clone(), g, vec![r.clone()]).await?;
                            Ok(r)
                        })
                    },
                    state: state.clone(),

                }
            };
            Ok(make_container(Value::Function(h)))
        },
        (Value::Object(obj), _) => call_metaprop(state.clone(), obj, vec![x, y], "subtract".to_string()).await,
        (_, Value::Object(obj)) => call_metaprop(state.clone(), obj, vec![x, y], "subtract".to_string()).await,
        _ => Err(make_err("can't subtract a and b")),
    }
}

pub async fn has_base(state: StateContainer, obj: Container, prop: String) -> Result<bool, Container> {
    let obj = resolve_bind(state.clone(), obj).await?.lock().await.clone();
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

pub async fn has(state: StateContainer, obj: Container, prop: Container) -> Result<Container, Container> {
    has_base(state.clone(), obj, to_string_base(state.clone(), prop.clone()).await?).await.map(|b| make_container(Value::Boolean(b)))
}

pub async fn set_base(state: StateContainer, obj: Container, prop: String, v: Container) -> Result<(), Container> {
    let a = if let Value::Object(o) = &mut *resolve_bind(state.clone(), obj).await?.lock().await {
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
        call(state.clone(), setter, vec![v]).await.map(|_| ())
    } else {
        Ok(())
    }
}

pub async fn set(state: StateContainer, obj: Container, prop: Container, v: Container) -> Result<Container, Container> {
    set_base(state.clone(), obj, to_string_base(state.clone(), prop.clone()).await?, v.clone()).await.map(|_| make_container(Value::Null))
}

pub async fn eql_base(state: StateContainer, a: Container, b: Container) -> Result<bool, Container> {
    let x = resolve_bind(state.clone(), a).await?;
    let y = resolve_bind(state.clone(), b).await?;
    let a = x.lock().await.clone();
    let b = y.lock().await.clone();
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
                Box::pin(eql_base(state.clone(), a.clone(), b.clone()))
            }).collect::<Vec<_>>();
            let mut d = Vec::new();
            for i in c {
                d.push(i.await?);
            }
            Ok(d.iter().all(|z| *z))
        },
        (Value::Function(_), Value::Function(_)) => Ok(Arc::ptr_eq(&x, &y)),
        _ => Ok(false),
    }
}

pub async fn eql(state: StateContainer, a: Container, b: Container) -> Result<Container, Container> {
    Ok(make_container(Value::Boolean(eql_base(state, a, b).await?)))
}

pub async fn neql_base(state: StateContainer, a: Container, b: Container) -> Result<bool, Container> {
    Ok(!eql_base(state, a, b).await?)
}

pub async fn neql(state: StateContainer, a: Container, b: Container) -> Result<Container, Container> {
    Ok(make_container(Value::Boolean(neql_base(state, a, b).await?)))
}

pub async fn to_boolean_base(state: StateContainer, v: Container) -> Result<bool, Container> {
    let v = resolve_bind(state.clone(), v).await?.lock().await.clone();
    match v {
        Value::Number(n) => Ok(n != 0),
        Value::Float(f) => Ok(f != 0.0),
        Value::Null => Ok(false),
        Value::String(s) => Ok(s.len() != 0),
        Value::Boolean(b) => Ok(b),
        _ => Ok(true),
    }
}

pub async fn to_boolean(state: StateContainer, v: Container) -> Result<Container, Container> {
    Ok(make_container(Value::Boolean(to_boolean_base(state, v).await?)))
}

pub async fn divide(state: StateContainer, a: Container, b: Container) -> Result<Container, Container> {
    let x = resolve_bind(state.clone(), a).await?;
    let y = resolve_bind(state.clone(), b).await?;
    let a = x.lock().await.clone();
    let b = y.lock().await.clone();
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
        (Value::Object(obj), _) => call_metaprop(state.clone(), obj, vec![x, y], "divide".to_string()).await,
        (_, Value::Object(obj)) => call_metaprop(state.clone(), obj, vec![x, y], "divide".to_string()).await,
        _ => Err(make_err("cannot divide a by b")),
    }
}

pub async fn remainder(state: StateContainer, a: Container, b: Container) -> Result<Container, Container> {
    let x = resolve_bind(state.clone(), a).await?;
    let y = resolve_bind(state.clone(), b).await?;
    let a = x.lock().await.clone();
    let b = y.lock().await.clone();
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
                        Box::pin(async move {
                            let f = gi(0).unwrap();
                            let n = match gi(1).unwrap().lock().await.clone() {
                                Value::Number(n) => n,
                                _ => return Err(make_container(Value::String("data corrupted".to_string())))
                            };
                            for _ in 0..n {
                                call(state.clone(), f.clone(), args.clone()).await?;
                            }
                            Ok(make_container(Value::Null))
                        })
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
                        Box::pin(async move {
                            let f = gi(0).unwrap();
                            let n = match gi(1).unwrap().lock().await.clone() {
                                Value::Number(n) => n,
                                _ => return Err(make_container(Value::String("data corrupted".to_string())))
                            };
                            for _ in 0..n {
                                call(state.clone(), f.clone(), args.clone()).await?;
                            }
                            Ok(make_container(Value::Null))
                        })
                    },
                    state: state.clone()
                }
            };
            Ok(make_container(Value::Function(g)))
        },
        (Value::Object(obj), _) => call_metaprop(state.clone(), obj, vec![x, y], "remainder".to_string()).await,
        (_, Value::Object(obj)) => call_metaprop(state.clone(), obj, vec![x, y], "remainder".to_string()).await,
        _ => Err(make_container(Value::String("cannot take the remainder of a by b".to_string()))),
    }
}

pub async fn to_number_base(state: StateContainer, v: Container) -> Result<i64, Container> {
    let v = resolve_bind(state.clone(), v).await?.lock().await.clone();
    match v {
        Value::Number(n) => Ok(n),
        Value::Float(f) => Ok(f as i64),
        Value::Null => Ok(0),
        Value::String(s) => s.parse().map_err(|_| make_container(Value::String("cannot convert v to number".to_string()))),
        Value::Boolean(b) => Ok(if b {1} else {0}),
        _ => Err(make_container(Value::String("cannot convert v to number".to_string())))
    }
}

pub async fn to_number(state: StateContainer, v: Container) -> Result<Container, Container> {
    Ok(make_container(Value::Number(to_number_base(state, v).await?)))
}

pub async fn to_float_base(state: StateContainer, v: Container) -> Result<f64, Container> {
    let v = resolve_bind(state.clone(), v).await?.lock().await.clone();
    match v {
        Value::Number(n) => Ok(n as f64),
        Value::Float(f) => Ok(f),
        Value::Null => Ok(0.0),
        Value::String(s) => s.parse().map_err(|_| make_container(Value::String("cannot convert v to float".to_string()))),
        Value::Boolean(b) => Ok(if b {1.0} else {0.0}),
        _ => Err(make_container(Value::String("cannot convert v to float".to_string()))),
    }
}

pub async fn to_float(state: StateContainer, v: Container) -> Result<Container, Container> {
    Ok(make_container(Value::Float(to_float_base(state, v).await?)))
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

pub async fn call_prop(state: StateContainer, obj: Container, args: Vec<Container>, prop: String) -> Result<Container, Container> {
    let f = get_base(state.clone(), obj, prop).await?;
    call(state.clone(), f, args).await
}

pub async fn not(state: StateContainer, v: Container) -> Result<Container, Container> {
    let x = resolve_bind(state.clone(), v).await?;
    let v = x.clone().lock().await.clone();
    match v {
        Value::Number(n) => Ok(make_container(Value::Number(!n))),
        Value::Float(n) => Ok(make_container(Value::Number(!(n as i64)))),
        Value::Boolean(b) => Ok(make_container(Value::Boolean(!b))),
        Value::Object(obj) => call_metaprop(state.clone(), obj, vec![x], "not".to_string()).await,
        _ => Err(make_err("can't not v")),
    }
}

pub async fn gt(state: StateContainer, a: Container, b: Container) -> Result<Container, Container> {
    let x = resolve_bind(state.clone(), a).await?;
    let y = resolve_bind(state.clone(), b).await?;
    let a = x.lock().await.clone();
    let b = y.lock().await.clone();
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => Ok(make_container(Value::Boolean(a > b))),
        (Value::Number(a), Value::Float(b)) => Ok(make_container(Value::Boolean((a as f64) > b))),
        (Value::Float(a), Value::Number(b)) => Ok(make_container(Value::Boolean(a > (b as f64)))),
        (Value::Float(a), Value::Float(b)) => Ok(make_container(Value::Boolean(a > b))),
        (Value::Object(obj), _) => call_metaprop(state.clone(), obj, vec![x, y], "gt".to_string()).await,
        (_, Value::Object(obj)) => call_metaprop(state.clone(), obj, vec![x, y], "gt".to_string()).await,
        _ => Err(make_err("cannot compare a and b")),
    }
}

pub async fn ge(state: StateContainer, a: Container, b: Container) -> Result<Container, Container> {
    let x = resolve_bind(state.clone(), a).await?;
    let y = resolve_bind(state.clone(), b).await?;
    let a = x.lock().await.clone();
    let b = y.lock().await.clone();
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => Ok(make_container(Value::Boolean(a >= b))),
        (Value::Number(a), Value::Float(b)) => Ok(make_container(Value::Boolean((a as f64) >= b))),
        (Value::Float(a), Value::Number(b)) => Ok(make_container(Value::Boolean(a >= (b as f64)))),
        (Value::Float(a), Value::Float(b)) => Ok(make_container(Value::Boolean(a >= b))),
        (Value::Object(obj), _) => call_metaprop(state.clone(), obj, vec![x, y], "ge".to_string()).await,
        (_, Value::Object(obj)) => call_metaprop(state.clone(), obj, vec![x, y], "ge".to_string()).await,
        _ => Err(make_err("cannot compare a and b")),
    }
}

pub async fn lt(state: StateContainer, a: Container, b: Container) -> Result<Container, Container> {
    let x = resolve_bind(state.clone(), a).await?;
    let y = resolve_bind(state.clone(), b).await?;
    let a = x.lock().await.clone();
    let b = y.lock().await.clone();
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => Ok(make_container(Value::Boolean(a < b))),
        (Value::Number(a), Value::Float(b)) => Ok(make_container(Value::Boolean((a as f64) < b))),
        (Value::Float(a), Value::Number(b)) => Ok(make_container(Value::Boolean(a < (b as f64)))),
        (Value::Float(a), Value::Float(b)) => Ok(make_container(Value::Boolean(a < b))),
        (Value::Object(obj), _) => call_metaprop(state.clone(), obj, vec![x, y], "lt".to_string()).await,
        (_, Value::Object(obj)) => call_metaprop(state.clone(), obj, vec![x, y], "lt".to_string()).await,
        _ => Err(make_err("cannot compare a and b")),
    }
}

pub async fn le(state: StateContainer, a: Container, b: Container) -> Result<Container, Container> {
    let x = resolve_bind(state.clone(), a).await?;
    let y = resolve_bind(state.clone(), b).await?;
    let a = x.lock().await.clone();
    let b = y.lock().await.clone();
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => Ok(make_container(Value::Boolean(a <= b))),
        (Value::Number(a), Value::Float(b)) => Ok(make_container(Value::Boolean((a as f64) <= b))),
        (Value::Float(a), Value::Number(b)) => Ok(make_container(Value::Boolean(a <= (b as f64)))),
        (Value::Float(a), Value::Float(b)) => Ok(make_container(Value::Boolean(a <= b))),
        (Value::Object(obj), _) => call_metaprop(state.clone(), obj, vec![x, y], "le".to_string()).await,
        (_, Value::Object(obj)) => call_metaprop(state.clone(), obj, vec![x, y], "le".to_string()).await,
        _ => Err(make_err("cannot compare a and b")),
    }
}

pub async fn and(state: StateContainer, a: Container, b: Container) -> Result<Container, Container> {
    let x = resolve_bind(state.clone(), a).await?;
    let y = resolve_bind(state.clone(), b).await?;
    let a = x.lock().await.clone();
    let b = y.lock().await.clone();
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => Ok(make_container(Value::Number(a & b))),
        (Value::Number(a), Value::Float(b)) => Ok(make_container(Value::Number(a & (b as i64)))),
        (Value::Float(a), Value::Number(b)) => Ok(make_container(Value::Number((a as i64) & b))),
        (Value::Float(a), Value::Float(b)) => Ok(make_container(Value::Number((a as i64) & (b as i64)))),
        (Value::Boolean(a), Value::Boolean(b)) => Ok(make_container(Value::Boolean(a & b))),
        (Value::Object(obj), _) => call_metaprop(state.clone(), obj, vec![x, y], "and".to_string()).await,
        (_, Value::Object(obj)) => call_metaprop(state.clone(), obj, vec![x, y], "and".to_string()).await,
        _ => Err(make_err("cannot compare a and b")),
    }
}

pub async fn or(state: StateContainer, a: Container, b: Container) -> Result<Container, Container> {
    let x = resolve_bind(state.clone(), a).await?;
    let y = resolve_bind(state.clone(), b).await?;
    let a = x.lock().await.clone();
    let b = y.lock().await.clone();
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => Ok(make_container(Value::Number(a | b))),
        (Value::Number(a), Value::Float(b)) => Ok(make_container(Value::Number(a | (b as i64)))),
        (Value::Float(a), Value::Number(b)) => Ok(make_container(Value::Number((a as i64) | b))),
        (Value::Float(a), Value::Float(b)) => Ok(make_container(Value::Number((a as i64) | (b as i64)))),
        (Value::Boolean(a), Value::Boolean(b)) => Ok(make_container(Value::Boolean(a | b))),
        (Value::Object(obj), _) => call_metaprop(state.clone(), obj, vec![x, y], "or".to_string()).await,
        (_, Value::Object(obj)) => call_metaprop(state.clone(), obj, vec![x, y], "or".to_string()).await,
        _ => Err(make_err("cannot compare a and b")),
    }
}

pub async fn xor(state: StateContainer, a: Container, b: Container) -> Result<Container, Container> {
    let x = resolve_bind(state.clone(), a).await?;
    let y = resolve_bind(state.clone(), b).await?;
    let a = x.lock().await.clone();
    let b = y.lock().await.clone();
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => Ok(make_container(Value::Number(a ^ b))),
        (Value::Number(a), Value::Float(b)) => Ok(make_container(Value::Number(a ^ (b as i64)))),
        (Value::Float(a), Value::Number(b)) => Ok(make_container(Value::Number((a as i64) ^ b))),
        (Value::Float(a), Value::Float(b)) => Ok(make_container(Value::Number((a as i64) ^ (b as i64)))),
        (Value::Boolean(a), Value::Boolean(b)) => Ok(make_container(Value::Boolean(a ^ b))),
        (Value::Object(obj), _) => call_metaprop(state.clone(), obj, vec![x, y], "xor".to_string()).await,
        (_, Value::Object(obj)) => call_metaprop(state.clone(), obj, vec![x, y], "xor".to_string()).await,
        _ => Err(make_err("cannot compare a and b")),
    }
}

pub async fn orthat(state: StateContainer, a: Container, b: Container) -> Result<Container, Container> {
    let x = resolve_bind(state.clone(), a).await?;
    let b = resolve_bind(state.clone(), b).await?;
    let a = x.lock().await.clone();
    match a {
        Value::Null => Ok(b),
        _ => Ok(x),
    }
}

fn make_function_call(state: StateContainer, args: Vec<Container>, gi: Gi) -> Pin<Box<dyn std::future::Future<Output = Result<Container, Container>> + Send>> {
    Box::pin(async move {
        let state = state.clone();
        let instrs = gi(0).unwrap();
        let instrs = match instrs.lock().await.clone() {
            Value::Object(obj) => Some(obj),
            _ => None,
        };
        let instrs = instrs.unwrap().externals;
        let instrs = instrs[&0].lock().await.downcast_ref::<Vec<Instruction>>().unwrap().clone();
        let mut tmps: HashMap<u64, Container> = HashMap::new();
        let r = interpret_instructions(state.clone(), &args, &mut tmps, &instrs, None).await?;
        match r {
            (Some(r), _) => Ok(r),
            (None, _) => Ok(make_container(Value::Null)),
        }
    })
}

pub async fn make_function(state: StateContainer, instrs: Vec<Instruction>, s: Option<StateContainer>) -> Result<Arc<Mutex<Value>>, Arc<Mutex<Value>>> {
    let mut obj = make_object_base();
    obj.externals.insert(0, Arc::new(Mutex::new(Box::new(instrs.clone()))));
    let mut internals = HashMap::new();
    internals.insert(0, make_container(Value::Object(obj)));
    let s = match s {
        Some(s) => s.clone(),
        None => new_state(state.clone()).await,
    };
    Ok(make_container(Value::Function(Function {
        internals,
        call: make_function_call,
        state: s,
    })))
}

pub async fn new_state(state: StateContainer) -> StateContainer {
    Arc::new(Mutex::new(State {
        scope: make_object(),
        parent: Some(state.clone()),
        global: {
            state.lock().await.global.clone()
        },
        globaldata: {
            state.lock().await.globaldata.clone()
        },
        debug: {
            state.lock().await.debug
        },
    }))
}

pub async fn get_from_state(ident: String, state: StateContainer) -> Result<Container, Container> {
    let obj = {
        state.lock().await.scope.clone()
    };
    if has_base(state.clone(), obj.clone(), ident.clone()).await? {
        get_base(state.clone(), obj.clone(), ident.clone()).await
    } else {
        let parent = state.lock().await.parent.clone();
        match parent {
            Some(s) => Box::pin(get_from_state(ident.clone(), s)).await,
            None => Ok(make_container(Value::Null)),
        }
    }
}

pub async fn set_to_state(ident: String, v: Container, state: StateContainer) -> Result<(), Container> {
    let obj = {
        state.lock().await.scope.clone()
    };
    if has_base(state.clone(), obj.clone(), ident.clone()).await? {
        set_base(state.clone(), obj.clone(), ident.clone(), v.clone()).await
    } else {
        let parent = state.lock().await.parent.clone();
        let scope = state.lock().await.scope.clone();
        match parent {
            Some(s) => Box::pin(set_to_state(ident.clone(), v.clone(), s)).await,
            None => set_base(state.clone(), scope, ident.clone(), v.clone()).await,
        }
    }
}

pub async fn get_var(state: StateContainer, args: &Vec<Container>, tmps: &HashMap<u64, Container>, vari: VarIndex) -> Result<Container, Container> {
    match vari {
        VarIndex::Arg(i) => Ok(match args.get(i) {
            Some(v) => v.clone(),
            None => make_container(Value::Null),
        }),
        VarIndex::Ident(ident) => get_from_state(ident, state.clone()).await,
        VarIndex::Temp(u) => Ok(match tmps.get(&u) {
            Some(v) => v.clone(),
            None => make_container(Value::Null),
        }),
    }
}

pub async fn set_var(state: StateContainer, tmps: &mut HashMap<u64, Container>, vari: VarIndex, v: Container) -> Result<(), Container> {
    match vari {
        VarIndex::Arg(_) => Err(make_container(Value::String("can't set argument".to_string()))),
        VarIndex::Ident(ident) => set_to_state(ident.clone(), v.clone(), state.clone()).await,
        VarIndex::Temp(u) => {
            tmps.insert(u, v.clone());
            Ok(())
        },
    }
}

pub async fn decl(state: StateContainer, vari: VarIndex) -> Result<(), Container> {
    let obj = {
        state.lock().await.scope.clone()
    };
    match vari {
        VarIndex::Ident(ident) => set_base(state.clone(), obj.clone(), ident.clone(), make_container(Value::Null)).await,
        _ => Err(make_container(Value::String("can't declare non-identifier".to_string()))),
    }
}

pub fn make_tuple(values: Vec<Container>) -> Container {
    make_container(Value::Tuple(values.clone()))
}

pub async fn detuple(state: StateContainer, v: Container) -> Result<Vec<Container>, Container> {
    let x = resolve_bind(state.clone(), v).await?;
    let v = x.lock().await.clone();
    match v {
        Value::Tuple(v) => Ok(v),
        _ => Ok(vec![x]),
    }
}

macro_rules! make_fn {
    ($state:expr, $fcall:expr) => {{
        make_container(Value::Function(Function {
            internals: HashMap::new(),
            call: |state, args, gi| {
                Box::pin(async move {
                    $fcall(state, args, gi).await
                })
            },
            state: $state.clone(),
        }))
    }};
}

pub async fn interpret_instructions(state: StateContainer, args: &Vec<Container>, tmps: &mut HashMap<u64, Container>, instrs: &Vec<Instruction>, pipeshort: Option<Container>) -> Result<(Option<Container>, Option<Label>), Container> {
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
                    Box::pin(interpret_instructions(state.clone(), args, tmps, &defers, None)).await?;
                }
                return Ok((Some(get_var(state.clone(), args, tmps, vi.clone()).await?), None))
            },
            Instruction::Throw(vi) => {
                if defers.len() > 0 {
                    Box::pin(interpret_instructions(state.clone(), args, tmps, &defers, None)).await?;
                }
                return Err(get_var(state.clone(), args, tmps, vi.clone()).await?)
            },
            Instruction::Add(res, op1, op2) => {
                let op1 = get_var(state.clone(), args, tmps, op1.clone()).await?;
                let op2 = get_var(state.clone(), args, tmps, op2.clone()).await?;
                let r = add(state.clone(), op1.clone(), op2.clone()).await?;
                set_var(state.clone(), tmps, res.clone(), r.clone()).await?;
            },
            Instruction::Call(res, f, ops) => {
                let ops = ops.iter().map(|i| get_var(state.clone(), args, tmps, i.clone())).collect::<Vec<_>>();
                let mut ops2 = Vec::new();
                for i in ops {
                    ops2.push(i.await?);
                }
                let ops = ops2;
                let f = get_var(state.clone(), args, tmps, f.clone()).await?;
                let r = call(state.clone(), f, ops).await?;
                set_var(state.clone(), tmps, res.clone(), r.clone()).await?;
            },
            Instruction::Get(res, obj, prop) => {
                let obj = get_var(state.clone(), args, tmps, obj.clone()).await?;
                let prop = get_var(state.clone(), args, tmps, prop.clone()).await?;
                let r = get(state.clone(), obj.clone(), prop.clone()).await?;
                set_var(state.clone(), tmps, res.clone(), r.clone()).await?;
            },
            Instruction::Multiply(res, op1, op2) => {
                let op1 = get_var(state.clone(), args, tmps, op1.clone()).await?;
                let op2 = get_var(state.clone(), args, tmps, op2.clone()).await?;
                let r = multiply(state.clone(), op1.clone(), op2.clone()).await?;
                set_var(state.clone(), tmps, res.clone(), r.clone()).await?;
            },
            Instruction::Negate(res, op) => {
                let op = get_var(state.clone(), args, tmps, op.clone()).await?;
                let r = negate(state.clone(), op.clone()).await?;
                set_var(state.clone(), tmps, res.clone(), r.clone()).await?;
            },
            Instruction::Subtract(res, op1, op2) => {
                let op1 = get_var(state.clone(), args, tmps, op1.clone()).await?;
                let op2 = get_var(state.clone(), args, tmps, op2.clone()).await?;
                let r = subtract(state.clone(), op1.clone(), op2.clone()).await?;
                set_var(state.clone(), tmps, res.clone(), r.clone()).await?;
            },
            Instruction::Has(res, obj, prop) => {
                let obj = get_var(state.clone(), args, tmps, obj.clone()).await?;
                let prop = get_var(state.clone(), args, tmps, prop.clone()).await?;
                let r = has(state.clone(), obj.clone(), prop.clone()).await?;
                set_var(state.clone(), tmps, res.clone(), r.clone()).await?;
            },
            Instruction::Set(res, obj, prop, value) => {
                let obj = get_var(state.clone(), args, tmps, obj.clone()).await?;
                let prop = get_var(state.clone(), args, tmps, prop.clone()).await?;
                let value = get_var(state.clone(), args, tmps, value.clone()).await?;
                let r = set(state.clone(), obj.clone(), prop.clone(), value.clone()).await?;
                set_var(state.clone(), tmps, res.clone(), r.clone()).await?;
            },
            Instruction::Decl(op) => {
                decl(state.clone(), op.clone()).await?;
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
                            Box::pin(interpret_instructions(state.clone(), args, tmps, &defers, None)).await?;
                        }
                        return Ok((None, Some(l.clone())))
                    },
                };
            },
            Instruction::Eql(res, op1, op2) => {
                let op1 = get_var(state.clone(), args, tmps, op1.clone()).await?;
                let op2 = get_var(state.clone(), args, tmps, op2.clone()).await?;
                let r = eql(state.clone(), op1.clone(), op2.clone()).await?;
                set_var(state.clone(), tmps, res.clone(), r.clone()).await?;
            },
            Instruction::Neql(res, op1, op2) => {
                let op1 = get_var(state.clone(), args, tmps, op1.clone()).await?;
                let op2 = get_var(state.clone(), args, tmps, op2.clone()).await?;
                let r = neql(state.clone(), op1.clone(), op2.clone()).await?;
                set_var(state.clone(), tmps, res.clone(), r.clone()).await?;
            },
            Instruction::GotoIf(l, op) => {
                let op = get_var(state.clone(), args, tmps, op.clone()).await?;
                if to_boolean_base(state.clone(), op).await? {
                    let opt = match l.clone() {
                        Label::Unnamed(u) => ulabels.get(&u),
                        Label::Named(s) => slabels.get(&s),
                    };
                    match opt {
                        Some(u) => i = *u,
                        None => {
                            if defers.len() > 0 {
                                Box::pin(interpret_instructions(state.clone(), args, tmps, &defers, None)).await?;
                            }
                            return Ok((None, Some(l.clone())))
                        },
                    };
                }
            },
            Instruction::Block(instvec) => {
                let s = new_state(state.clone()).await;
                let r = Box::pin(interpret_instructions(s, args, tmps, &instvec, Some(pipeshort.clone()))).await?;
                match r {
                    (Some(v), l) => {
                        if defers.len() > 0 {
                            Box::pin(interpret_instructions(state.clone(), args, tmps, &defers, None)).await?;
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
                                    Box::pin(interpret_instructions(state.clone(), args, tmps, &defers, None)).await?;
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
                    ops2.push(i.await?);
                }
                let ops = ops2;
                let r = make_tuple(ops);
                set_var(state.clone(), tmps, res.clone(), r.clone()).await?;
            },
            Instruction::DeTuple(results, op) => {
                let op = get_var(state.clone(), args, tmps, op.clone()).await?;
                let r = detuple(state.clone(), op).await?;
                let mut i = 0;
                while i < results.len() {
                    let r = match r.get(i) {
                        Some(v) => v.clone(),
                        None => make_container(Value::Null),
                    };
                    set_var(state.clone(), tmps, results[i].clone(), r).await?;
                    i += 1;
                }
            },
            Instruction::Divide(res, op1, op2) => {
                let op1 = get_var(state.clone(), args, tmps, op1.clone()).await?;
                let op2 = get_var(state.clone(), args, tmps, op2.clone()).await?;
                let r = divide(state.clone(), op1.clone(), op2.clone()).await?;
                set_var(state.clone(), tmps, res.clone(), r.clone()).await?;
            },
            Instruction::Remainder(res, op1, op2) => {
                let op1 = get_var(state.clone(), args, tmps, op1.clone()).await?;
                let op2 = get_var(state.clone(), args, tmps, op2.clone()).await?;
                let r = remainder(state.clone(), op1.clone(), op2.clone()).await?;
                set_var(state.clone(), tmps, res.clone(), r.clone()).await?;
            },
            Instruction::MakeBind(res, f) => {
                let f = get_var(state.clone(), args, tmps, f).await?;
                let r = make_container(Value::Bind(f));
                set_var(state.clone(), tmps, res, r).await?;
            },
            Instruction::Catch(erri, err, instvec) => {
                let s = new_state(state.clone()).await;
                let r = Box::pin(interpret_instructions(s, args, tmps, &instvec, Some(pipeshort.clone()))).await;
                match r {
                    Err(e) => {
                        set_var(state.clone(), tmps, erri, make_container(Value::Boolean(true))).await?;
                        set_var(state.clone(), tmps, err, e).await?;
                    },
                    Ok(r) => {
                        set_var(state.clone(), tmps, erri, make_container(Value::Boolean(false))).await?;
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
                                            Box::pin(interpret_instructions(state.clone(), args, tmps, &defers, None)).await?;
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
                let op = get_var(state.clone(), args, tmps, op.clone()).await?;
                set_var(state.clone(), tmps, r.clone(), op).await?;
            },
            Instruction::Defer(v) => {
                defers.extend(v.iter().map(|v| v.clone()));
            },
            Instruction::Boolean(res, op) => {
                let op = make_container(Value::Boolean(op));
                set_var(state.clone(), tmps, res.clone(), op).await?;
            },
            Instruction::Number(res, op) => {
                let op = make_container(Value::Number(op));
                set_var(state.clone(), tmps, res.clone(), op).await?;
            },
            Instruction::Float(res, op) => {
                let op = make_container(Value::Float(op));
                set_var(state.clone(), tmps, res.clone(), op).await?;
            },
            Instruction::String(res, op) => {
                let op = make_container(Value::String(op));
                set_var(state.clone(), tmps, res.clone(), op).await?;
            },
            Instruction::MakeFunction(res, body) => {
                let f = make_function(state.clone(), body, None).await?;
                set_var(state.clone(), tmps, res, f).await?;
            },
            Instruction::Not(res, op) => {
                let op = get_var(state.clone(), args, tmps, op).await?;
                let r = not(state.clone(), op).await?;
                set_var(state.clone(), tmps, res, r).await?;
            },
            Instruction::Gt(res, op1, op2) => {
                let op1 = get_var(state.clone(), args, tmps, op1.clone()).await?;
                let op2 = get_var(state.clone(), args, tmps, op2.clone()).await?;
                let r = gt(state.clone(), op1.clone(), op2.clone()).await?;
                set_var(state.clone(), tmps, res.clone(), r.clone()).await?;
            },
            Instruction::Ge(res, op1, op2) => {
                let op1 = get_var(state.clone(), args, tmps, op1.clone()).await?;
                let op2 = get_var(state.clone(), args, tmps, op2.clone()).await?;
                let r = ge(state.clone(), op1.clone(), op2.clone()).await?;
                set_var(state.clone(), tmps, res.clone(), r.clone()).await?;
            },
            Instruction::Lt(res, op1, op2) => {
                let op1 = get_var(state.clone(), args, tmps, op1.clone()).await?;
                let op2 = get_var(state.clone(), args, tmps, op2.clone()).await?;
                let r = lt(state.clone(), op1.clone(), op2.clone()).await?;
                set_var(state.clone(), tmps, res.clone(), r.clone()).await?;
            },
            Instruction::Le(res, op1, op2) => {
                let op1 = get_var(state.clone(), args, tmps, op1.clone()).await?;
                let op2 = get_var(state.clone(), args, tmps, op2.clone()).await?;
                let r = le(state.clone(), op1.clone(), op2.clone()).await?;
                set_var(state.clone(), tmps, res.clone(), r.clone()).await?;
            },
            Instruction::And(res, op1, op2) => {
                let op1 = get_var(state.clone(), args, tmps, op1.clone()).await?;
                let op2 = get_var(state.clone(), args, tmps, op2.clone()).await?;
                let r = and(state.clone(), op1.clone(), op2.clone()).await?;
                set_var(state.clone(), tmps, res.clone(), r.clone()).await?;
            },
            Instruction::Or(res, op1, op2) => {
                let op1 = get_var(state.clone(), args, tmps, op1.clone()).await?;
                let op2 = get_var(state.clone(), args, tmps, op2.clone()).await?;
                let r = or(state.clone(), op1.clone(), op2.clone()).await?;
                set_var(state.clone(), tmps, res.clone(), r.clone()).await?;
            },
            Instruction::Xor(res, op1, op2) => {
                let op1 = get_var(state.clone(), args, tmps, op1.clone()).await?;
                let op2 = get_var(state.clone(), args, tmps, op2.clone()).await?;
                let r = xor(state.clone(), op1.clone(), op2.clone()).await?;
                set_var(state.clone(), tmps, res.clone(), r.clone()).await?;
            },
            Instruction::GetPipeShorthand(res) => {
                set_var(state.clone(), tmps, res, pipeshort.clone()).await?;
            },
            Instruction::SetPipeShorthand(op) => {
                pipeshort = get_var(state.clone(), args, tmps, op).await?;
            },
            Instruction::OrThat(res, op1, op2) => {
                let op1 = get_var(state.clone(), args, tmps, op1.clone()).await?;
                let op2 = get_var(state.clone(), args, tmps, op2.clone()).await?;
                let r = orthat(state.clone(), op1.clone(), op2.clone()).await?;
                set_var(state.clone(), tmps, res.clone(), r.clone()).await?;
            },
            Instruction::OperatorFn(res, op) => {
                let f = match op {
                    Operator::Plus => make_fn!(state, opfn::plus),
                    Operator::Minus => make_fn!(state, opfn::minus),
                    Operator::Times => make_fn!(state, opfn::times),
                    Operator::Divide => make_fn!(state, opfn::divide_),
                    Operator::Modulus => make_fn!(state, opfn::modulus),
                    Operator::OrThat => make_fn!(state, opfn::orthat_),
                    Operator::Ternary => make_fn!(state, opfn::ternary),
                    Operator::EqualTo => make_fn!(state, opfn::equalto),
                    Operator::Not => make_fn!(state, opfn::not_),
                    Operator::NotEqualTo => make_fn!(state, opfn::notequalto),
                    Operator::Less => make_fn!(state, opfn::less),
                    Operator::LessOrEqual => make_fn!(state, opfn::lessorequal),
                    Operator::Greater => make_fn!(state, opfn::greater),
                    Operator::GreaterOrEqual => make_fn!(state, opfn::greaterorequal),
                    Operator::And => make_fn!(state, opfn::and_),
                    Operator::Or => make_fn!(state, opfn::or_),
                    Operator::Xor => make_fn!(state, opfn::xor_),
                    Operator::Property => make_fn!(state, opfn::property),
                    Operator::Tuple => make_fn!(state, opfn::tuple),
                    Operator::Pipe => make_fn!(state, opfn::pipe),
                };
                set_var(state.clone(), tmps, res, f).await?;
            },
        }
        i += 1;
    }
    if defers.len() > 0 {
        Box::pin(interpret_instructions(state.clone(), args, tmps, &defers, None)).await?;
    }
    Ok((None, None))
}