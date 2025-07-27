use std::{collections::HashMap, sync::Arc};

use bodu_vm::op::{add, get_base, gt, lt, make_object, set_base, to_number_base, to_string_base};
use tokio::sync::Mutex;

use crate::{array::{self, new_with_vec}, vm::{make_container, make_err, op::{call, call_prop, detuple, make_object_base, make_tuple, to_boolean_base}, Container, Function, Gi, StateContainer, Value}};

pub async fn collect(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("iter.collect requires 1 argument"))
    }
    let arg = args[0].clone();
    let mut v = Vec::new();
    loop {
        let r = call(state.clone(), arg.clone(), Vec::new()).await?;
        let r = detuple(state.clone(), r).await?;
        let b = match r.get(0) {
            None => return Err(make_err("invalid iterator passed to iter.collect")),
            Some(v) => to_boolean_base(state.clone(), v.clone()).await?,
        };
        if !b {
            break;
        }
        let rv = match r.get(1) {
            None => return Err(make_err("invalid iterator passed to iter.collect")),
            Some(v) => v.clone(),
        };
        v.push(rv);
    }
    new_with_vec(state.clone(), v).await
}

pub async fn reverse(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("iter.reverse requires 1 argument"))
    }
    let arg = args[0].clone();
    let mut v = Vec::new();
    loop {
        let r = call(state.clone(), arg.clone(), Vec::new()).await?;
        let r = detuple(state.clone(), r).await?;
        let b = match r.get(0) {
            None => return Err(make_err("invalid iterator passed to iter.collect")),
            Some(v) => to_boolean_base(state.clone(), v.clone()).await?,
        };
        if !b {
            break;
        }
        let rv = match r.get(1) {
            None => return Err(make_err("invalid iterator passed to iter.collect")),
            Some(v) => v.clone(),
        };
        v.push(rv);
    }
    let v = v.into_iter().rev().collect::<Vec<_>>();
    let f = {
        let mut internals = HashMap::new();
        let mut oo = make_object_base();
        oo.externals.insert(0, Arc::new(Mutex::new(Box::new(v.clone().into_iter()))));
        let oo = make_container(Value::Object(oo));
        internals.insert(0, oo);
        Function {
            internals,
            call: reverse_next_wrapper,
            state: state.clone(),
            caller_state: false,
        }
    };
    Ok(make_container(Value::Function(f)))
}

fn reverse_next_wrapper(_: StateContainer, _: Vec<Container>, gi: Gi) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Container, Container>> + Send>> {
    Box::pin(async move {
        let i = gi(0).unwrap();
        let i = (match i.lock().await.clone() {
            Value::Object(i) => Some(i),
            _ => None,
        }).unwrap();
        let i = i.externals.get(&0).unwrap().clone();
        let mut i = i.lock().await;
        let i = i.downcast_mut::<<Vec<Container> as IntoIterator>::IntoIter>().unwrap();
        Ok(match i.next() {
            None => make_tuple(vec![make_container(Value::Boolean(false)), make_container(Value::Null)]),
            Some(i) => make_tuple(vec![make_container(Value::Boolean(true)), i.clone()]),
        })
    })
}

pub async fn all(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("iter.all requires 2 arguments"))
    }
    let i = args[0].clone();
    let f = args[1].clone();
    loop {
        let r = call(state.clone(), i.clone(), Vec::new()).await?;
        let r = detuple(state.clone(), r).await?;
        let b = match r.get(0) {
            None => return Err(make_err("invalid iterator passed to iter.all")),
            Some(v) => to_boolean_base(state.clone(), v.clone()).await?,
        };
        if !b {
            break;
        }
        let rv = match r.get(1) {
            None => return Err(make_err("invalid iterator passed to iter.all")),
            Some(v) => v.clone(),
        };
        let rv = call(state.clone(), f.clone(), vec![rv]).await?;
        if !to_boolean_base(state.clone(), rv).await? {
            return Ok(make_container(Value::Boolean(false)))
        }
    }
    Ok(make_container(Value::Boolean(true)))
}

pub async fn any(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("iter.any requires 2 arguments"))
    }
    let i = args[0].clone();
    let f = args[1].clone();
    loop {
        let r = call(state.clone(), i.clone(), Vec::new()).await?;
        let r = detuple(state.clone(), r).await?;
        let b = match r.get(0) {
            None => return Err(make_err("invalid iterator passed to iter.any")),
            Some(v) => to_boolean_base(state.clone(), v.clone()).await?,
        };
        if !b {
            break;
        }
        let rv = match r.get(1) {
            None => return Err(make_err("invalid iterator passed to iter.any")),
            Some(v) => v.clone(),
        };
        let rv = call(state.clone(), f.clone(), vec![rv]).await?;
        if to_boolean_base(state.clone(), rv).await? {
            return Ok(make_container(Value::Boolean(true)))
        }
    }
    Ok(make_container(Value::Boolean(false)))
}

pub async fn chain(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("iter.chain requires 2 arguments"))
    }
    let f = args[0].clone();
    let g = args[1].clone();
    let h = {
        let mut internals = HashMap::new();
        internals.insert(0, f.clone());
        internals.insert(0, g.clone());
        Function {
            internals,
            call: |state, _, gi| {
                Box::pin(async move {
                    let f = gi(0).unwrap();
                    let r = call(state.clone(), f, Vec::new()).await?;
                    let r = detuple(state.clone(), r).await?;
                    let b = match r.get(0) {
                        None => return Err(make_err("invalid iterator passed to iter.chain")),
                        Some(v) => to_boolean_base(state.clone(), v.clone()).await?,
                    };
                    if b {
                        let rv = match r.get(1) {
                            None => return Err(make_err("invalid iterator passed to iter.chain")),
                            Some(v) => v.clone(),
                        };
                        Ok(make_tuple(vec![make_container(Value::Boolean(true)), rv]))
                    } else {
                        let f = gi(1).unwrap();
                        let r = call(state.clone(), f, Vec::new()).await?;
                        let r = detuple(state.clone(), r).await?;
                        let b = match r.get(0) {
                            None => return Err(make_err("invalid iterator passed to iter.chain")),
                            Some(v) => to_boolean_base(state.clone(), v.clone()).await?,
                        };
                        if b {
                            let rv = match r.get(1) {
                                None => return Err(make_err("invalid iterator passed to iter.chain")),
                                Some(v) => v.clone(),
                            };
                            Ok(make_tuple(vec![make_container(Value::Boolean(true)), rv]))
                        } else {
                            Ok(make_tuple(vec![make_container(Value::Boolean(false)), make_container(Value::Null)]))
                        }
                    }
                })
            },
            state: state.clone(),
            caller_state: true,
        }
    };
    Ok(make_container(Value::Function(h)))
}

pub async fn cycle(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("iter.cycle requires 1 argument"))
    }
    let f = args[0].clone();
    let g = {
        let mut internals = HashMap::new();
        let mut obj = make_object_base();
        obj.internals.insert(0, array::new_with_vec(state.clone(), vec![]).await?);
        obj.internals.insert(1, f);
        internals.insert(0, make_container(Value::Object(obj)));
        Function {
            internals,
            call: |state, _, gi| {
                Box::pin(async move {
                    let obj = gi(0).unwrap().clone();
                    let obj = &mut *obj.lock().await;
                    let obj = match obj {
                        Value::Object(obj) => obj,
                        _ => return Err(make_err("data corrupted")),
                    };
                    let arr = obj.internals[&0].clone();
                    let f = obj.internals[&1].clone();
                    let r = call(state.clone(), f, Vec::new()).await?;
                    let r = detuple(state.clone(), r).await?;
                    let b = match r.get(0) {
                        None => return Err(make_err("invalid iterator passed to iter.cycle")),
                        Some(v) => to_boolean_base(state.clone(), v.clone()).await?,
                    };
                    if b {
                        let rv = match r.get(1) {
                            None => return Err(make_err("invalid iterator passed to iter.chain")),
                            Some(v) => v.clone(),
                        };
                        call_prop(state.clone(), arr.clone(), vec![rv.clone()], "push".to_string()).await?;
                        Ok(make_tuple(vec![make_container(Value::Boolean(true)), rv]))
                    } else {
                        let f = call_prop(state.clone(), arr, Vec::new(), "iter".to_string()).await?;
                        obj.internals.insert(1, f);
                        obj.internals.insert(0, array::new_with_vec(state.clone(), vec![]).await?);
                        let arr = obj.internals[&0].clone();
                        let f = obj.internals[&1].clone();
                        let r = call(state.clone(), f, Vec::new()).await?;
                        let r = detuple(state.clone(), r).await?;
                        let b = match r.get(0) {
                            None => return Err(make_err("invalid iterator passed to iter.cycle")),
                            Some(v) => to_boolean_base(state.clone(), v.clone()).await?,
                        };
                        if b {
                            let rv = match r.get(1) {
                                None => return Err(make_err("invalid iterator passed to iter.chain")),
                                Some(v) => v.clone(),
                            };
                            call_prop(state.clone(), arr.clone(), vec![rv.clone()], "push".to_string()).await?;
                            Ok(make_tuple(vec![make_container(Value::Boolean(true)), rv]))
                        } else {
                            Ok(make_tuple(vec![make_container(Value::Boolean(false)), make_container(Value::Null)]))
                        }
                    }
                })
            },
            state: state.clone(),
            caller_state: true,
        }
    };
    Ok(make_container(Value::Function(g)))
}

pub async fn count(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("iter.count requires 1 argument"))
    }
    let i = args[0].clone();
    let mut count = 0;
    loop {
        let r = call(state.clone(), i.clone(), Vec::new()).await?;
        let r = detuple(state.clone(), r).await?;
        let b = match r.get(0) {
            None => return Err(make_err("invalid iterator passed to iter.count")),
            Some(v) => to_boolean_base(state.clone(), v.clone()).await?,
        };
        if !b {
            break;
        }
        count += 1;
    }
    Ok(make_container(Value::Number(count)))
}

pub async fn enumerate(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("iter.enumerate requires 1 argument"))
    }
    let f = args[0].clone();
    let g = {
        let mut internals = HashMap::new();
        let obj = make_object();
        set_base(state.clone(), obj.clone(), "i".to_string(), make_container(Value::Number(0))).await?;
        internals.insert(0, obj);
        internals.insert(1, f);
        Function {
            internals,
            call: |state, _, gi| {
                Box::pin(async move {
                    let obj = gi(0).unwrap().clone();
                    let i = get_base(state.clone(), obj.clone(), "i".to_string()).await?;
                    let i = to_number_base(state.clone(), i).await?;
                    let f = gi(1).unwrap().clone();
                    let r = call(state.clone(), f.clone(), vec![]).await?;
                    let r = detuple(state.clone(), r).await?;
                    let b = match r.get(0) {
                        None => return Err(make_err("invalid iterator passed to iter.enumerate")),
                        Some(v) => to_boolean_base(state.clone(), v.clone()).await?,
                    };
                    if b {
                        let rv = match r.get(1) {
                            None => return Err(make_err("invalid iterator passed to iter.enumerate")),
                            Some(v) => v.clone(),
                        };
                        set_base(state.clone(), obj.clone(), "i".to_string(), make_container(Value::Number(i+1))).await?;
                        Ok(make_tuple(vec![make_container(Value::Boolean(true)), make_tuple(vec![make_container(Value::Number(i)), rv])]))
                    } else {
                        Ok(make_tuple(vec![make_container(Value::Boolean(false)), make_container(Value::Null)]))
                    }
                })
            },
            state: state.clone(),
            caller_state: true,
        }
    };
    Ok(make_container(Value::Function(g)))
}

pub async fn filter(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("iter.filter requires 2 arguments"))
    }
    let it = args[0].clone();
    let f = args[1].clone();
    let g = {
        let mut internals = HashMap::new();
        internals.insert(0, it);
        internals.insert(1, f);
        Function {
            internals,
            call: |state, _, gi| {
                Box::pin(async move {
                    let it = gi(0).unwrap().clone();
                    let f = gi(1).unwrap().clone();
                    let r = loop {
                        let r = call(state.clone(), it.clone(), vec![]).await?;
                        let r = detuple(state.clone(), r).await?;
                        let b = match r.get(0) {
                            None => return Err(make_err("invalid iterator passed to iter.filter")),
                            Some(v) => to_boolean_base(state.clone(), v.clone()).await?,
                        };
                        if b {
                            let rv = match r.get(1) {
                                None => return Err(make_err("invalid iterator passed to iter.filter")),
                                Some(v) => v.clone(),
                            };
                            let b = call(state.clone(), f.clone(), vec![rv.clone()]).await?;
                            let b = to_boolean_base(state.clone(), b).await?;
                            if b {
                                break make_tuple(vec![make_container(Value::Boolean(true)), rv.clone()]);
                            }
                        } else {
                            break make_tuple(vec![make_container(Value::Boolean(false)), make_container(Value::Null)]);
                        }
                    };
                    Ok(r)
                })
            },
            state: state.clone(),
            caller_state: true,
        }
    };
    Ok(make_container(Value::Function(g)))
}

pub async fn map(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("iter.map requires 2 arguments"))
    }
    let it = args[0].clone();
    let f = args[1].clone();
    let g = {
        let mut internals = HashMap::new();
        internals.insert(0, it);
        internals.insert(1, f);
        Function {
            internals,
            call: |state, _, gi| {
                Box::pin(async move {
                    let it = gi(0).unwrap().clone();
                    let f = gi(1).unwrap().clone();
                    let r = call(state.clone(), it.clone(), vec![]).await?;
                    let r = detuple(state.clone(), r).await?;
                    let b = match r.get(0) {
                        None => return Err(make_err("invalid iterator passed to iter.map")),
                        Some(v) => to_boolean_base(state.clone(), v.clone()).await?,
                    };
                    if b {
                        let rv = match r.get(1) {
                            None => return Err(make_err("invalid iterator passed to iter.map")),
                            Some(v) => v.clone(),
                        };
                        let rv = call(state.clone(), f.clone(), vec![rv.clone()]).await?;
                        Ok(make_tuple(vec![make_container(Value::Boolean(true)), rv.clone()]))
                    } else {
                        Ok(make_tuple(vec![make_container(Value::Boolean(false)), make_container(Value::Null)]))
                    }
                })
            },
            state: state.clone(),
            caller_state: true,
        }
    };
    Ok(make_container(Value::Function(g)))
}

pub async fn sum(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("iter.sum requires 1 argument"))
    }
    let i = args[0].clone();
    let mut sum = make_container(Value::Number(0));
    loop {
        let r = call(state.clone(), i.clone(), Vec::new()).await?;
        let r = detuple(state.clone(), r).await?;
        let b = match r.get(0) {
            None => return Err(make_err("invalid iterator passed to iter.sum")),
            Some(v) => to_boolean_base(state.clone(), v.clone()).await?,
        };
        if !b {
            break;
        }
        let rv = match r.get(1) {
            None => return Err(make_err("invalid iterator passed to iter.sum")),
            Some(v) => v.clone(),
        };
        sum = add(state.clone(), sum, rv).await?;
    }
    Ok(sum)
}

pub async fn min(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("iter.min requires 1 argument"))
    }
    let i = args[0].clone();
    let mut min = make_container(Value::Number(0));
    {
        let r = call(state.clone(), i.clone(), Vec::new()).await?;
        let r = detuple(state.clone(), r).await?;
        let b = match r.get(0) {
            None => return Err(make_err("invalid iterator passed to iter.min")),
            Some(v) => to_boolean_base(state.clone(), v.clone()).await?,
        };
        if !b {
            return Ok(min);
        }
        min = match r.get(1) {
            None => return Err(make_err("invalid iterator passed to iter.min")),
            Some(v) => v.clone(),
        };
    }
    loop {
        let r = call(state.clone(), i.clone(), Vec::new()).await?;
        let r = detuple(state.clone(), r).await?;
        let b = match r.get(0) {
            None => return Err(make_err("invalid iterator passed to iter.min")),
            Some(v) => to_boolean_base(state.clone(), v.clone()).await?,
        };
        if !b {
            break;
        }
        let rv = match r.get(1) {
            None => return Err(make_err("invalid iterator passed to iter.min")),
            Some(v) => v.clone(),
        };
        let lower = lt(state.clone(), rv.clone(), min.clone()).await?;
        if to_boolean_base(state.clone(), lower.clone()).await? {
            min = rv;
        }
    }
    Ok(min)
}

pub async fn max(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("iter.max requires 1 argument"))
    }
    let i = args[0].clone();
    let mut max = make_container(Value::Number(0));
    {
        let r = call(state.clone(), i.clone(), Vec::new()).await?;
        let r = detuple(state.clone(), r).await?;
        let b = match r.get(0) {
            None => return Err(make_err("invalid iterator passed to iter.max")),
            Some(v) => to_boolean_base(state.clone(), v.clone()).await?,
        };
        if !b {
            return Ok(max);
        }
        max = match r.get(1) {
            None => return Err(make_err("invalid iterator passed to iter.max")),
            Some(v) => v.clone(),
        };
    }
    loop {
        let r = call(state.clone(), i.clone(), Vec::new()).await?;
        let r = detuple(state.clone(), r).await?;
        let b = match r.get(0) {
            None => return Err(make_err("invalid iterator passed to iter.max")),
            Some(v) => to_boolean_base(state.clone(), v.clone()).await?,
        };
        if !b {
            break;
        }
        let rv = match r.get(1) {
            None => return Err(make_err("invalid iterator passed to iter.max")),
            Some(v) => v.clone(),
        };
        let greater = gt(state.clone(), rv.clone(), max.clone()).await?;
        if to_boolean_base(state.clone(), greater.clone()).await? {
            max = rv;
        }
    }
    Ok(max)
}

pub async fn join(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("iter.join requires 2 arguments"))
    }
    let i = args[0].clone();
    let sep = to_string_base(state.clone(), args[1].clone()).await?;
    let mut res = String::new();
    {
        let r = call(state.clone(), i.clone(), Vec::new()).await?;
        let r = detuple(state.clone(), r).await?;
        let b = match r.get(0) {
            None => return Err(make_err("invalid iterator passed to iter.join")),
            Some(v) => to_boolean_base(state.clone(), v.clone()).await?,
        };
        if !b {
            return Ok(make_container(Value::String(res)));
        }
        let r =  match r.get(1) {
            None => return Err(make_err("invalid iterator passed to iter.join")),
            Some(v) => to_string_base(state.clone(), v.clone()).await?,
        };
        res += &r;
    }
    loop {
        let r = call(state.clone(), i.clone(), Vec::new()).await?;
        let r = detuple(state.clone(), r).await?;
        let b = match r.get(0) {
            None => return Err(make_err("invalid iterator passed to iter.join")),
            Some(v) => to_boolean_base(state.clone(), v.clone()).await?,
        };
        if !b {
            break;
        }
        let rv = match r.get(1) {
            None => return Err(make_err("invalid iterator passed to iter.join")),
            Some(v) => v.clone(),
        };
        res += &sep;
        res += &to_string_base(state.clone(), rv).await?;
    }
    Ok(make_container(Value::String(res)))
}