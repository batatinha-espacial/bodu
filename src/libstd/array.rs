use std::{collections::HashMap, sync::{Arc, Mutex}};

use crate::vm::{make_container, op::{make_object_base, make_tuple, set_base, to_number_base}, Container, Function, Gi, StateContainer, Value};

macro_rules! helper1 {
    ($state:expr, $fcall:expr, $o:expr, $prop:expr) => {{
        let mut fn_ = Function {
            internals: HashMap::new(),
            call: $fcall,
            state: $state.clone(),
        };
        fn_.internals.insert(0, $o.clone());
        set_base($state.clone(), $o.clone(), $prop.to_string(), make_container(Value::Function(fn_)))?;
    }};
}

pub fn new(state: StateContainer, _: Vec<Container>, _: Gi) -> Result<Container, Container> {
    new_with_vec(state, Vec::new())
}

pub fn new_with_vec(state: StateContainer, data: Vec<Container>) -> Result<Container, Container> {
    let mut o = make_object_base();
    o.externals.insert(0, Arc::new(Mutex::new(Box::new(data.clone()))));
    let o = make_container(Value::Object(o));

    helper1!(state, get, o, "get");
    helper1!(state, set, o, "set");
    helper1!(state, pop, o, "pop");
    helper1!(state, push, o, "push");
    helper1!(state, iter, o, "iter");

    Ok(o)
}

fn get(state: StateContainer, args: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().unwrap().clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().unwrap();
    let o = o.downcast_mut::<Vec<Container>>().unwrap();
    if args.len() == 0 {
        return Err(make_container(Value::String("array.get requires 1 argument".to_string())))
    }
    let mut i = to_number_base(state.clone(), args[0].clone())?;
    if i < -(o.len() as i64) || i >= o.len() as i64 {
        return Ok(make_container(Value::Null))
    }
    if i < 0 {
        i = (o.len() as i64) + i;
    }
    Ok(o[i as usize].clone())
}

fn set(state: StateContainer, args: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().unwrap().clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().unwrap();
    let o = o.downcast_mut::<Vec<Container>>().unwrap();
    if args.len() < 2 {
        return Err(make_container(Value::String("array.set requires 2 argument".to_string())))
    }
    let mut i = to_number_base(state.clone(), args[0].clone())?;
    if i < -(o.len() as i64) || i >= o.len() as i64 {
        return Ok(make_container(Value::Null))
    }
    if i < 0 {
        i = (o.len() as i64) + i;
    }
    o[i as usize] = args[1].clone();
    Ok(make_container(Value::Null))
}

fn pop(_: StateContainer, _: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().unwrap().clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().unwrap();
    let o = o.downcast_mut::<Vec<Container>>().unwrap();
    let v = o.pop();
    Ok(match v {
        None => make_container(Value::Null),
        Some(v) => v.clone(),
    })
}

fn push(_: StateContainer, args: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().unwrap().clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().unwrap();
    let o = o.downcast_mut::<Vec<Container>>().unwrap();
    o.extend(args.iter().map(|v| v.clone()));
    Ok(make_container(Value::Number(o.len() as i64)))
}

fn iter(state: StateContainer, _: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().unwrap().clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().unwrap();
    let o = o.downcast_mut::<Vec<Container>>().unwrap();
    let f = {
        let mut internals = HashMap::new();
        let mut oo = make_object_base();
        oo.externals.insert(0, Arc::new(Mutex::new(Box::new(o.clone().into_iter()))));
        let oo = make_container(Value::Object(oo));
        internals.insert(0, oo);
        Function {
            internals,
            call: |_, _, gi| {
                let i = gi(0).unwrap();
                let i = (match i.lock().unwrap().clone() {
                    Value::Object(i) => Some(i),
                    _ => None,
                }).unwrap();
                let i = i.externals.get(&0).unwrap().clone();
                let mut i = i.lock().unwrap();
                let i = i.downcast_mut::<<Vec<Container> as IntoIterator>::IntoIter>().unwrap();
                Ok(match i.next() {
                    None => make_tuple(vec![make_container(Value::Boolean(false)), make_container(Value::Null)]),
                    Some(i) => make_tuple(vec![make_container(Value::Boolean(true)), i.clone()]),
                })
            },
            state: state.clone(),
        }
    };
    Ok(make_container(Value::Function(f)))
}