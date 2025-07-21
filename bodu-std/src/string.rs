use std::{collections::HashMap, sync::Arc};

use bodu_vm::{make_container, make_err, op::{make_object_base, make_tuple, to_string_base}, Container, Function, Gi, StateContainer, Value};
use tokio::sync::Mutex;

pub async fn len(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("string.len requires 1 argument"));
    }
    let s = to_string_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Number(s.len() as i64)))
}

pub async fn count_chars(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("string.count_chars requires 1 argument"));
    }
    let s = to_string_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Number(s.chars().count() as i64)))
}

pub async fn chars(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("string.chars requires 1 argument"));
    }
    let s = to_string_base(state.clone(), args[0].clone()).await?;
    let f = {
        let mut internals = HashMap::new();
        let mut oo = make_object_base();
        oo.externals.insert(0, Arc::new(Mutex::new(Box::new(s.chars().collect::<Vec<char>>().into_iter()))));
        let oo = make_container(Value::Object(oo));
        internals.insert(0, oo);
        Function {
            internals,
            call: chars_next_wrapper,
            caller_state: false,
            state: state.clone(),
        }
    };
    Ok(make_container(Value::Function(f)))
}

fn chars_next_wrapper(_: StateContainer, _: Vec<Container>, gi: Gi) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Container, Container>> + Send>> {
    Box::pin(async move {
        let i = gi(0).unwrap();
        let i = (match i.lock().await.clone() {
            Value::Object(i) => Some(i),
            _ => None,
        }).unwrap();
        let i = i.externals.get(&0).unwrap().clone();
        let mut i = i.lock().await;
        let i = i.downcast_mut::<<Vec<char> as IntoIterator>::IntoIter>().unwrap();
        Ok(match i.next() {
            None => make_tuple(vec![make_container(Value::Boolean(false)), make_container(Value::Null)]),
            Some(i) => make_tuple(vec![make_container(Value::Boolean(true)), make_container(Value::String(i.to_string()))]),
        })
    })
}

pub async fn ords(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("string.ords requires 1 argument"));
    }
    let s = to_string_base(state.clone(), args[0].clone()).await?;
    let f = {
        let mut internals = HashMap::new();
        let mut oo = make_object_base();
        oo.externals.insert(0, Arc::new(Mutex::new(Box::new(s.chars().collect::<Vec<char>>().into_iter()))));
        let oo = make_container(Value::Object(oo));
        internals.insert(0, oo);
        Function {
            internals,
            call: ords_next_wrapper,
            caller_state: false,
            state: state.clone(),
        }
    };
    Ok(make_container(Value::Function(f)))
}

fn ords_next_wrapper(_: StateContainer, _: Vec<Container>, gi: Gi) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Container, Container>> + Send>> {
    Box::pin(async move {
        let i = gi(0).unwrap();
        let i = (match i.lock().await.clone() {
            Value::Object(i) => Some(i),
            _ => None,
        }).unwrap();
        let i = i.externals.get(&0).unwrap().clone();
        let mut i = i.lock().await;
        let i = i.downcast_mut::<<Vec<char> as IntoIterator>::IntoIter>().unwrap();
        Ok(match i.next() {
            None => make_tuple(vec![make_container(Value::Boolean(false)), make_container(Value::Null)]),
            Some(i) => make_tuple(vec![make_container(Value::Boolean(true)), make_container(Value::Number(i as u32 as i64))]),
        })
    })
}

pub async fn trim(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("string.trim requires 1 argument"));
    }
    let s = to_string_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::String(s.trim().to_string())))
}