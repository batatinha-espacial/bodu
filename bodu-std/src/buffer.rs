use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

use crate::vm::{make_container, make_err, op::{make_object, make_object_base, set_base, to_number_base, to_string_base}, Container, Function, Gi, StateContainer, Value};

macro_rules! helper1 {
    ($state:expr, $fcall:expr, $o:expr, $prop:expr) => {{
        let mut fn_ = Function {
            internals: HashMap::new(),
            call: |state, args, gi| {
                Box::pin(async move {
                    $fcall(state, args, gi).await
                })
            },
            state: $state.clone(),
        };
        fn_.internals.insert(0, $o.clone());
        set_base($state.clone(), $o.clone(), $prop.to_string(), make_container(Value::Function(fn_))).await?;
    }};
}

macro_rules! helper2 {
    ($state:expr, $fcall:expr, $o:expr, $prop:expr) => {{
        let fn_ = Function {
            internals: HashMap::new(),
            call: |state, args, gi| {
                Box::pin(async move {
                    $fcall(state, args, gi).await
                })
            },
            state: $state.clone(),
        };
        set_base($state.clone(), $o.clone(), $prop.to_string(), make_container(Value::Function(fn_))).await?;
    }};
}

pub async fn from_string_utf8(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("buffer.from_string_utf8 requires 1 argument"));
    }
    let s = to_string_base(state.clone(), args[0].clone()).await?;
    let s = s.as_bytes().to_vec();
    new_from_vec(state.clone(), s).await
}

pub async fn from_string_utf16be(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("buffer.from_string_utf16be requires 1 argument"));
    }
    let s = to_string_base(state.clone(), args[0].clone()).await?;
    let s = s.encode_utf16().flat_map(|v| v.to_be_bytes()).collect();
    new_from_vec(state.clone(), s).await
}

pub async  fn from_string_utf16le(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("buffer.from_string_utf16le requires 1 argument"));
    }
    let s = to_string_base(state.clone(), args[0].clone()).await?;
    let s = s.encode_utf16().flat_map(|v| v.to_le_bytes()).collect();
    new_from_vec(state.clone(), s).await
}

pub async fn new_from_vec(state: StateContainer, data: Vec<u8>) -> Result<Container, Container> {
    let mut o = make_object_base();
    o.internals.insert(u64::MAX, make_container(Value::String("buffer".to_string())));
    o.externals.insert(0, Arc::new(Mutex::new(Box::new(data.clone()))));
    o.metaobj = make_object();
    helper2!(state, meta_to_string, o.metaobj, "to_string");

    let o = make_container(Value::Object(o));
    helper1!(state, get, o, "get");
    helper1!(state, len, o, "len");
    helper1!(state, to_string_utf8, o, "to_string_utf8");
    
    Ok(o)
}

async fn get(state: StateContainer, args: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().await.clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().await;
    let o = o.downcast_mut::<Vec<u8>>().unwrap();
    if args.len() == 0 {
        return Err(make_err("buffer.get requires 1 argument"));
    }
    let mut i = to_number_base(state.clone(), args[0].clone()).await?;
    if i < -(o.len() as i64) || i >= o.len() as i64 {
        return Ok(make_container(Value::Null))
    }
    if i < 0 {
        i = (o.len() as i64) + i;
    }
    Ok(make_container(Value::Number(o[i as usize] as i64)))
}

async fn len(_: StateContainer, _: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().await.clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().await;
    let o = o.downcast_mut::<Vec<u8>>().unwrap();
    Ok(make_container(Value::Number(o.len() as i64)))
}

async fn meta_to_string(_: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("buffer's metaobj to_string requires 1 argument"));
    }
    let a = {
        let o = args[0].clone();
        let o = match match o.lock().await.clone() {
            Value::Object(o) => Some(o),
            _ => None,
        } {
            Some(o) => o,
            _ => return Err(make_err("buffer's metaobj to_string requires 1 buffer")),
        };
        let o = match o.externals.get(&0) {
            Some(o) => o.clone(),
            _ => return Err(make_err("buffer's metaobj to_string requires 1 buffer")),
        };
        let mut o = o.lock().await;
        let o = match o.downcast_mut::<Vec<u8>>() {
            Some(o) => o.clone(),
            _ => return Err(make_err("buffer's metaobj to_string requires 1 buffer")),
        };
        o
    };
    let a = a.iter().map(|v| v.to_string()).collect::<Vec<_>>();
    Ok(make_container(Value::String("<buffer ".to_string() + &a.join(" ") + ">")))
}

async fn to_string_utf8(_: StateContainer, _: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().await.clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().await;
    let o = o.downcast_mut::<Vec<u8>>().unwrap();
    let s = String::from_utf8(o.clone());
    match s {
        Ok(s) => Ok(make_container(Value::String(s))),
        Err(_) => Err(make_err("buffer.to_string_utf8 can't decode invalid utf8")),
    }
}