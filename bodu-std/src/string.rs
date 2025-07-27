use std::{collections::HashMap, sync::Arc};

use bodu_vm::{make_container, make_err, op::{make_object_base, make_tuple, to_number_base, to_string_base}, Container, Function, Gi, StateContainer, Value};
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

pub async fn reverse(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("string.trim requires 1 argument"));
    }
    let s = to_string_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::String(s.chars().rev().collect())))
}

pub async fn uppercase(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("string.uppercase requires 1 argument"));
    }
    let s = to_string_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::String(s.to_uppercase())))
}

pub async fn lowercase(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("string.lowercase requires 1 argument"));
    }
    let s = to_string_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::String(s.to_lowercase())))
}

pub async fn capitalize(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("string.capitalize requires 1 argument"));
    }
    let s = to_string_base(state.clone(), args[0].clone()).await?;
    let s = {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => {
                first.to_uppercase().collect::<String>() + chars.as_str()
            },
        }
    };
    Ok(make_container(Value::String(s)))
}

pub async fn contains(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("string.contains requires 2 arguments"));
    }
    let s = to_string_base(state.clone(), args[0].clone()).await?;
    let pat = to_string_base(state.clone(), args[1].clone()).await?;
    Ok(make_container(Value::Boolean(s.contains(&pat))))
}

pub async fn ends_with(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("string.ends_with requires 2 arguments"));
    }
    let s = to_string_base(state.clone(), args[0].clone()).await?;
    let pat = to_string_base(state.clone(), args[1].clone()).await?;
    Ok(make_container(Value::Boolean(s.ends_with(&pat))))
}

pub async fn find(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("string.find requires 2 arguments"));
    }
    let s = to_string_base(state.clone(), args[0].clone()).await?;
    let pat = to_string_base(state.clone(), args[1].clone()).await?;
    let r = s.find(&pat);
    match r {
        None => Ok(make_container(Value::Null)),
        Some(r) => Ok(make_container(Value::Number(r as i64))),
    }
}

pub async fn is_ascii(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("string.is_ascii requires 1 argument"));
    }
    let s = to_string_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Boolean(s.is_ascii())))
}

pub async fn is_char_boundary(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("string.is_char_boundary requires 2 arguments"));
    }
    let s = to_string_base(state.clone(), args[0].clone()).await?;
    let i = to_number_base(state.clone(), args[1].clone()).await?;
    Ok(make_container(Value::Boolean(s.is_char_boundary(i as usize))))
}

pub async fn is_empty(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("string.is_empty requires 1 argument"));
    }
    let s = to_string_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Boolean(s.is_empty())))
}

pub async fn lines(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("string.lines requires 1 argument"));
    }
    let s = to_string_base(state.clone(), args[0].clone()).await?;
    let s = s.lines().map(|s| s.to_string()).collect::<Vec<String>>();
    let f = {
        let mut internals = HashMap::new();
        let mut oo = make_object_base();
        oo.externals.insert(0, Arc::new(Mutex::new(Box::new(s.into_iter()))));
        let oo = make_container(Value::Object(oo));
        internals.insert(0, oo);
        Function {
            internals,
            call: lines_next_wrapper,
            state: state.clone(),
            caller_state: false,
        }
    };
    Ok(make_container(Value::Function(f)))
}

fn lines_next_wrapper(_: StateContainer, _: Vec<Container>, gi: Gi) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Container, Container>> + Send>> {
    Box::pin(async move {
        let i = gi(0).unwrap();
        let i = (match i.lock().await.clone() {
            Value::Object(i) => Some(i),
            _ => None,
        }).unwrap();
        let i = i.externals.get(&0).unwrap().clone();
        let mut i = i.lock().await;
        let i = i.downcast_mut::<<Vec<String> as IntoIterator>::IntoIter>().unwrap();
        Ok(match i.next() {
            None => make_tuple(vec![make_container(Value::Boolean(false)), make_container(Value::Null)]),
            Some(i) => make_tuple(vec![make_container(Value::Boolean(true)), make_container(Value::String(i))]),
        })
    })
}

pub async fn starts_with(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("string.starts_with requires 2 arguments"));
    }
    let s = to_string_base(state.clone(), args[0].clone()).await?;
    let pat = to_string_base(state.clone(), args[1].clone()).await?;
    Ok(make_container(Value::Boolean(s.starts_with(&pat))))
}