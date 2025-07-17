use std::sync::Arc;

use crate::{libstd::array, vm::{make_container, make_err, op::{call, make_object_base}, Container, Gi, ObjectProp, StateContainer, Value}};

pub fn encode(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("json.encode requires 1 argument"));
    }
    let mut visited = Vec::new();
    let v = encode_base(state.clone(), args[0].clone(), &mut visited)?;
    Ok(make_container(Value::String(v.to_string())))
}

fn encode_base(state: StateContainer, v: Container, visited: &mut Vec<Container>) -> Result<serde_json::Value, Container> {
    if visited.iter().any(|a| Arc::ptr_eq(a, &v)) {
        return Err(make_err("json.encode was called with a cyclic object"))
    }
    visited.push(v.clone());
    let isarr = {
        let o = v.clone();
        let o = o.lock().unwrap().clone();
        match o {
            Value::Object(o) => {
                match o.internals.get(&u64::MAX) {
                    None => false,
                    Some(v) => {
                        let o = v.clone();
                        let o = o.lock().unwrap().clone();
                        match o {
                            Value::String(s) => s.clone() == "array",
                            _ => false,
                        }
                    },
                }
            },
            _ => false,
        }
    };
    let v = &*v.lock().unwrap();
    match v {
        Value::Null => Ok(serde_json::Value::Null),
        Value::Number(a) => match serde_json::Number::from_f64(*a as f64) {
            Some(a) => Ok(serde_json::Value::Number(a)),
            None => Err(make_err("json.encode was unable to convert number or float")),
        },
        Value::Float(a) => match serde_json::Number::from_f64(*a) {
            Some(a) => Ok(serde_json::Value::Number(a)),
            None => Err(make_err("json.encode was unable to convert number or float")),
        },
        Value::String(a) => Ok(serde_json::Value::String(a.clone())),
        Value::Boolean(a) => Ok(serde_json::Value::Bool(*a)),
        Value::Object(o) => {
            if isarr {
                let mut vec_ = Vec::new();
                let o = {
                    let o = match o.externals.get(&0) {
                        Some(o) => o.clone(),
                        _ => return Err(make_err("json.encode can't encode array")),
                    };
                    let mut o = o.lock().unwrap();
                    let o = match o.downcast_mut::<Vec<Container>>() {
                        Some(o) => o.clone(),
                        _ => return Err(make_err("json.encode can't encode array")),
                    };
                    o
                };
                for i in o {
                    vec_.push(encode_base(state.clone(), i, visited)?);
                }
                Ok(serde_json::Value::Array(vec_))
            } else {
                let mut map = serde_json::Map::new();
                for i in o.props.iter() {
                    match i.1 {
                        ObjectProp::Value(v) => map.insert(i.0.clone(), encode_base(state.clone(), v.clone(), visited)?),
                        ObjectProp::GetSet(v, _) => map.insert(i.0.clone(), encode_base(state.clone(), call(state.clone(), v.clone(), Vec::new())?, visited)?),
                    };
                }
                Ok(serde_json::Value::Object(map))
            }
        },
        _ => Err(make_err("json.encode cannot encode tuples or functions")),
    }
}

pub fn decode(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("json.decode requires 1 argument"));
    }
    let a = args[0].clone();
    let a = match a.lock().unwrap().clone() {
        Value::String(s) => s,
        _ => return Err(make_err("json.decode requires 1 string")),
    };
    let v: serde_json::Value = serde_json::from_str(&a).map_err(|_| make_err("json.decode couldn't decode the JSON string"))?;
    decode_base(state.clone(), v)
}

fn decode_base(state: StateContainer, v: serde_json::Value) -> Result<Container, Container> {
    match v {
        serde_json::Value::Null => Ok(make_container(Value::Null)),
        serde_json::Value::Number(a) => {
            let a = a.as_f64();
            match a {
                Some(a) => Ok(make_container(Value::Float(a))),
                None => Err(make_err("json.decode couldn't decode a float")),
            }
        },
        serde_json::Value::Bool(a) => Ok(make_container(Value::Boolean(a))),
        serde_json::Value::String(a) => Ok(make_container(Value::String(a.clone()))),
        serde_json::Value::Array(a) => {
            let mut vec_ = Vec::new();
            for i in a {
                vec_.push(decode_base(state.clone(), i)?);
            }
            array::new_with_vec(state.clone(), vec_)
        },
        serde_json::Value::Object(a) => {
            let mut obj = make_object_base();
            for i in a {
                obj.props.insert(i.0.clone(), ObjectProp::Value(decode_base(state.clone(), i.1)?));
            }
            Ok(make_container(Value::Object(obj)))
        },
    }
}