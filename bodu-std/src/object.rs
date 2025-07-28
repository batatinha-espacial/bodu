use std::{collections::HashMap, sync::Arc};

use bodu_vm::{make_container, make_err, op::{make_object_base, make_tuple}, Container, Function, Gi, StateContainer, Value};
use tokio::sync::Mutex;

pub async fn new(_: StateContainer, _: Vec<Container>, _: Gi) -> Result<Container, Container> {
    let mut obj = make_object_base();
    obj.metalocked = false;
    Ok(make_container(Value::Object(obj)))
}

pub async fn keys(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("object.keys requires 1 argument"));
    }
    let keys = {
        let o = args[0].clone();
        let o = &mut *o.lock().await;
        let o = match o {
            Value::Object(o) => o,
            _ => return Err(make_err("object.keys requires 1 object"))
        };
        o.props.keys().map(|s| s.clone()).collect::<Vec<_>>()
    };
    let f = {
        let mut internals = HashMap::new();
        let mut oo = make_object_base();
        oo.externals.insert(0, Arc::new(Mutex::new(Box::new(keys.into_iter()))));
        let oo = make_container(Value::Object(oo));
        internals.insert(0, oo);
        Function {
            internals,
            call: keys_next_wrapper,
            state: state.clone(),
            caller_state: false,
        }
    };
    Ok(make_container(Value::Function(f)))
}

fn keys_next_wrapper(_: StateContainer, _: Vec<Container>, gi: Gi) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Container, Container>> + Send>> {
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