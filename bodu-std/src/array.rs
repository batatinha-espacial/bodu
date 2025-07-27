use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

use crate::vm::{make_container, make_err, op::{make_object, make_object_base, make_tuple, set_base, to_number_base, to_string_base}, Container, Function, Gi, StateContainer, Value};

// Function pointer wrappers for array operations
fn array_get_wrapper(state: StateContainer, args: Vec<Container>, gi: Gi) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Container, Container>> + Send>> {
    Box::pin(async move {
        get(state, args, gi).await
    })
}

fn array_set_wrapper(state: StateContainer, args: Vec<Container>, gi: Gi) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Container, Container>> + Send>> {
    Box::pin(async move {
        set(state, args, gi).await
    })
}

fn array_pop_wrapper(state: StateContainer, args: Vec<Container>, gi: Gi) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Container, Container>> + Send>> {
    Box::pin(async move {
        pop(state, args, gi).await
    })
}

fn array_push_wrapper(state: StateContainer, args: Vec<Container>, gi: Gi) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Container, Container>> + Send>> {
    Box::pin(async move {
        push(state, args, gi).await
    })
}

fn array_iter_wrapper(state: StateContainer, args: Vec<Container>, gi: Gi) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Container, Container>> + Send>> {
    Box::pin(async move {
        iter(state, args, gi).await
    })
}

fn array_len_wrapper(state: StateContainer, args: Vec<Container>, gi: Gi) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Container, Container>> + Send>> {
    Box::pin(async move {
        len(state, args, gi).await
    })
}

fn array_shift_wrapper(state: StateContainer, args: Vec<Container>, gi: Gi) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Container, Container>> + Send>> {
    Box::pin(async move {
        shift(state, args, gi).await
    })
}

fn array_unshift_wrapper(state: StateContainer, args: Vec<Container>, gi: Gi) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Container, Container>> + Send>> {
    Box::pin(async move {
        unshift(state, args, gi).await
    })
}

fn meta_add_wrapper(state: StateContainer, args: Vec<Container>, gi: Gi) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Container, Container>> + Send>> {
    Box::pin(async move {
        meta_add(state, args, gi).await
    })
}

fn meta_to_string_wrapper(state: StateContainer, args: Vec<Container>, gi: Gi) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Container, Container>> + Send>> {
    Box::pin(async move {
        meta_to_string(state, args, gi).await
    })
}

fn iter_next_wrapper(_: StateContainer, _: Vec<Container>, gi: Gi) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Container, Container>> + Send>> {
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

macro_rules! helper1 {
    ($state:expr, $fcall:expr, $o:expr, $prop:expr) => {{
        let mut fn_ = Function {
            internals: HashMap::new(),
            call: $fcall,
            state: $state.clone(),
            caller_state: false,
        };
        fn_.internals.insert(0, $o.clone());
        set_base($state.clone(), $o.clone(), $prop.to_string(), make_container(Value::Function(fn_))).await?;
    }};
}

macro_rules! helper2 {
    ($state:expr, $fcall:expr, $o:expr, $prop:expr) => {{
        let fn_ = Function {
            internals: HashMap::new(),
            call: $fcall,
            state: $state.clone(),
            caller_state: false,
        };
        set_base($state.clone(), $o.clone(), $prop.to_string(), make_container(Value::Function(fn_))).await?;
    }};
}

pub async fn new(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    new_with_vec(state, args).await
}

pub async fn new_with_vec(state: StateContainer, data: Vec<Container>) -> Result<Container, Container> {
    let mut o = make_object_base();
    o.internals.insert(u64::MAX, make_container(Value::String("array".to_string())));
    o.externals.insert(0, Arc::new(Mutex::new(Box::new(data.clone()))));
    o.metaobj = make_object();
    helper2!(state, meta_add_wrapper, o.metaobj, "add");
    helper2!(state, meta_to_string_wrapper, o.metaobj, "to_string");
    let o = make_container(Value::Object(o));

    helper1!(state, array_get_wrapper, o, "get");
    helper1!(state, array_set_wrapper, o, "set");
    helper1!(state, array_pop_wrapper, o, "pop");
    helper1!(state, array_push_wrapper, o, "push");
    helper1!(state, array_iter_wrapper, o, "iter");
    helper1!(state, array_len_wrapper, o, "len");
    helper1!(state, array_shift_wrapper, o, "shift");
    helper1!(state, array_unshift_wrapper, o, "unshift");

    Ok(o)
}

pub async fn is_array(_: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("array.is_array requires 1 argument"));
    }
    let o = args[0].clone();
    let o = o.lock().await.clone();
    let o = match o {
        Value::Object(o) => o,
        _ => return Ok(make_container(Value::Boolean(false))),
    };
    let o = match o.internals.get(&u64::MAX) {
        None => return Ok(make_container(Value::Boolean(false))),
        Some(v) => v.clone(),
    };
    let o = o.lock().await.clone();
    let o = match o {
        Value::String(s) => s.clone(),
        _ => return Ok(make_container(Value::Boolean(false))),
    };
    Ok(make_container(Value::Boolean(o == "array")))
}

async fn get(state: StateContainer, args: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().await.clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().await;
    let o = o.downcast_mut::<Vec<Container>>().unwrap();
    if args.len() == 0 {
        return Err(make_container(Value::String("array.get requires 1 argument".to_string())))
    }
    let mut i = to_number_base(state.clone(), args[0].clone()).await?;
    if i < -(o.len() as i64) || i >= o.len() as i64 {
        return Ok(make_container(Value::Null))
    }
    if i < 0 {
        i = (o.len() as i64) + i;
    }
    Ok(o[i as usize].clone())
}

async fn set(state: StateContainer, args: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().await.clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().await;
    let o = o.downcast_mut::<Vec<Container>>().unwrap();
    if args.len() < 2 {
        return Err(make_container(Value::String("array.set requires 2 argument".to_string())))
    }
    let mut i = to_number_base(state.clone(), args[0].clone()).await?;
    if i < -(o.len() as i64) || i >= o.len() as i64 {
        return Ok(make_container(Value::Null))
    }
    if i < 0 {
        i = (o.len() as i64) + i;
    }
    o[i as usize] = args[1].clone();
    Ok(make_container(Value::Null))
}

async fn pop(_: StateContainer, _: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().await.clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().await;
    let o = o.downcast_mut::<Vec<Container>>().unwrap();
    let v = o.pop();
    Ok(match v {
        None => make_container(Value::Null),
        Some(v) => v.clone(),
    })
}

async fn push(_: StateContainer, args: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().await.clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().await;
    let o = o.downcast_mut::<Vec<Container>>().unwrap();
    o.extend(args.iter().map(|v| v.clone()));
    Ok(make_container(Value::Number(o.len() as i64)))
}

async fn iter(state: StateContainer, _: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().await.clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().await;
    let o = o.downcast_mut::<Vec<Container>>().unwrap();
    let f = {
        let mut internals = HashMap::new();
        let mut oo = make_object_base();
        oo.externals.insert(0, Arc::new(Mutex::new(Box::new(o.clone().into_iter()))));
        let oo = make_container(Value::Object(oo));
        internals.insert(0, oo);
        Function {
            internals,
            call: iter_next_wrapper,
            state: state.clone(),
            caller_state: false,
        }
    };
    Ok(make_container(Value::Function(f)))
}

async fn meta_add(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("array's metaobj add requires 2 arguments"));
    }
    let a = {
        let o = args[0].clone();
        let o = match match o.lock().await.clone() {
            Value::Object(o) => Some(o),
            _ => None,
        } {
            Some(o) => o,
            _ => return Err(make_err("array's metaobj add requires 2 arrays")),
        };
        let o = match o.externals.get(&0) {
            Some(o) => o.clone(),
            _ => return Err(make_err("array's metaobj add requires 2 arrays")),
        };
        let mut o = o.lock().await;
        let o = match o.downcast_mut::<Vec<Container>>() {
            Some(o) => o.clone(),
            _ => return Err(make_err("array's metaobj add requires 2 arrays")),
        };
        o
    };
    let b = {
        let o = args[1].clone();
        let o = match match o.lock().await.clone() {
            Value::Object(o) => Some(o),
            _ => None,
        } {
            Some(o) => o,
            _ => return Err(make_err("array's metaobj add requires 2 arrays")),
        };
        let o = match o.externals.get(&0) {
            Some(o) => o.clone(),
            _ => return Err(make_err("array's metaobj add requires 2 arrays")),
        };
        let mut o = o.lock().await;
        let o = match o.downcast_mut::<Vec<Container>>() {
            Some(o) => o.clone(),
            _ => return Err(make_err("array's metaobj add requires 2 arrays")),
        };
        o
    };
    new_with_vec(state.clone(), a.into_iter().chain(b.into_iter()).collect()).await
}

async fn meta_to_string(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("array's metaobj to_string requires 1 argument"));
    }
    let a = {
        let o = args[0].clone();
        let o = match match o.lock().await.clone() {
            Value::Object(o) => Some(o),
            _ => None,
        } {
            Some(o) => o,
            _ => return Err(make_err("array's metaobj to_string requires 1 array")),
        };
        let o = match o.externals.get(&0) {
            Some(o) => o.clone(),
            _ => return Err(make_err("array's metaobj to_string requires 1 array")),
        };
        let mut o = o.lock().await;
        let o = match o.downcast_mut::<Vec<Container>>() {
            Some(o) => o.clone(),
            _ => return Err(make_err("array's metaobj to_string requires 1 array")),
        };
        o
    };
    let mut vec_ = Vec::new();
    for i in a {
        vec_.push(to_string_base(state.clone(), i).await?);
    }
    Ok(make_container(Value::String("[".to_string()+&vec_.join(", ")+&"]")))
}

async fn len(_: StateContainer, _: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().await.clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().await;
    let o = o.downcast_mut::<Vec<Container>>().unwrap();
    Ok(make_container(Value::Number(o.len() as i64)))
}

async fn shift(_: StateContainer, _: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().await.clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().await;
    let o = o.downcast_mut::<Vec<Container>>().unwrap();
    if o.len() == 0 {
        Ok(make_container(Value::Null))
    } else {
        let i = o.remove(0);
        Ok(i)
    }
}

async fn unshift(_: StateContainer, args: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().await.clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().await;
    let o = o.downcast_mut::<Vec<Container>>().unwrap();
    *o = o.clone().into_iter().chain(args.clone().into_iter()).collect();
    Ok(make_container(Value::Number(o.len() as i64)))
}