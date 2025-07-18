use std::collections::HashMap;

use crate::{libstd::array::{self, new_with_vec}, vm::{make_container, make_err, op::{call, call_prop, detuple, make_object_base, make_tuple, to_boolean_base}, Container, Function, Gi, StateContainer, Value}};

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
        }
    };
    Ok(make_container(Value::Function(g)))
}