use std::collections::HashMap;

use crate::{libstd::array::new_with_vec, vm::{make_container, make_err, op::{call, detuple, make_tuple, to_boolean_base}, Container, Function, Gi, StateContainer, Value}};

pub fn collect(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("iter.collect requires 1 argument"))
    }
    let arg = args[0].clone();
    let mut v = Vec::new();
    loop {
        let r = call(state.clone(), arg.clone(), Vec::new())?;
        let r = detuple(state.clone(), r)?;
        let b = match r.get(0) {
            None => return Err(make_err("invalid iterator passed to iter.collect")),
            Some(v) => to_boolean_base(state.clone(), v.clone())?,
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
    new_with_vec(state.clone(), v)
}

pub fn all(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("iter.all requires 2 arguments"))
    }
    let i = args[0].clone();
    let f = args[1].clone();
    loop {
        let r = call(state.clone(), i.clone(), Vec::new())?;
        let r = detuple(state.clone(), r)?;
        let b = match r.get(0) {
            None => return Err(make_err("invalid iterator passed to iter.all")),
            Some(v) => to_boolean_base(state.clone(), v.clone())?,
        };
        if !b {
            break;
        }
        let rv = match r.get(1) {
            None => return Err(make_err("invalid iterator passed to iter.all")),
            Some(v) => v.clone(),
        };
        let rv = call(state.clone(), f.clone(), vec![rv])?;
        if !to_boolean_base(state.clone(), rv)? {
            return Ok(make_container(Value::Boolean(false)))
        }
    }
    Ok(make_container(Value::Boolean(true)))
}

pub fn any(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("iter.any requires 2 arguments"))
    }
    let i = args[0].clone();
    let f = args[1].clone();
    loop {
        let r = call(state.clone(), i.clone(), Vec::new())?;
        let r = detuple(state.clone(), r)?;
        let b = match r.get(0) {
            None => return Err(make_err("invalid iterator passed to iter.any")),
            Some(v) => to_boolean_base(state.clone(), v.clone())?,
        };
        if !b {
            break;
        }
        let rv = match r.get(1) {
            None => return Err(make_err("invalid iterator passed to iter.any")),
            Some(v) => v.clone(),
        };
        let rv = call(state.clone(), f.clone(), vec![rv])?;
        if to_boolean_base(state.clone(), rv)? {
            return Ok(make_container(Value::Boolean(true)))
        }
    }
    Ok(make_container(Value::Boolean(false)))
}

pub fn chain(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
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
                let f = gi(0).unwrap();
                let r = call(state.clone(), f, Vec::new())?;
                let r = detuple(state.clone(), r)?;
                let b = match r.get(0) {
                    None => return Err(make_err("invalid iterator passed to iter.chain")),
                    Some(v) => to_boolean_base(state.clone(), v.clone())?,
                };
                if b {
                    let rv = match r.get(1) {
                        None => return Err(make_err("invalid iterator passed to iter.chain")),
                        Some(v) => v.clone(),
                    };
                    Ok(make_tuple(vec![make_container(Value::Boolean(true)), rv]))
                } else {
                    let f = gi(1).unwrap();
                    let r = call(state.clone(), f, Vec::new())?;
                    let r = detuple(state.clone(), r)?;
                    let b = match r.get(0) {
                        None => return Err(make_err("invalid iterator passed to iter.chain")),
                        Some(v) => to_boolean_base(state.clone(), v.clone())?,
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
            },
            state: state.clone(),
        }
    };
    Ok(make_container(Value::Function(h)))
}