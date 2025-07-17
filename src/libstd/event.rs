use std::{collections::HashMap, sync::{Arc, Mutex}};

use crate::vm::{make_container, make_err, op::{call, eql_base, make_object_base, set_base, to_string_base}, Container, Function, Gi, StateContainer, Value};

#[derive(Clone)]
struct EventsData {
    pub data: HashMap<String, EventData>,
}

#[derive(Clone)]
struct EventData {
    pub active: bool,
    pub listeners: Vec<Listener>,
}

#[derive(Clone)]
enum Listener {
    Once(Container),
    On(Container),
}

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
    new_base(state.clone())
}

pub fn new_base(state: StateContainer) -> Result<Container, Container> {
    let events_data = EventsData {
        data: HashMap::new(),
    };
    let mut o = make_object_base();
    o.externals.insert(0, Arc::new(Mutex::new(Box::new(events_data))));
    let o = make_container(Value::Object(o));

    helper1!(state, on, o, "on");
    helper1!(state, once, o, "once");
    helper1!(state, off, o, "off");
    helper1!(state, isactive, o, "isactive");
    helper1!(state, activate, o, "activate");
    helper1!(state, deactivate, o, "deactivate");
    helper1!(state, clear, o, "clear");
    helper1!(state, emit, o, "emit");

    Ok(o)
}

fn init_event(events_data: &mut EventsData, name: String) {
    if let None = events_data.data.get(&name) {
        events_data.data.insert(name, EventData {
            active: true,
            listeners: Vec::new(),
        });
    }
}

fn on(state: StateContainer, args: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().unwrap().clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().unwrap();
    let o = o.downcast_mut::<EventsData>().unwrap();
    if args.len() < 2 {
        return Err(make_err("event.on requires 2 arguments"));
    }
    let name = to_string_base(state.clone(), args[0].clone())?;
    let f = args[1].clone();
    init_event(o, name.clone());
    let event = o.data.get_mut(&name).unwrap();
    event.listeners.push(Listener::On(f.clone()));
    Ok(f.clone())
}

fn once(state: StateContainer, args: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().unwrap().clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().unwrap();
    let o = o.downcast_mut::<EventsData>().unwrap();
    if args.len() < 2 {
        return Err(make_err("event.once requires 2 arguments"));
    }
    let name = to_string_base(state.clone(), args[0].clone())?;
    let f = args[1].clone();
    init_event(o, name.clone());
    let event = o.data.get_mut(&name).unwrap();
    event.listeners.push(Listener::Once(f.clone()));
    Ok(f.clone())
}

fn off(state: StateContainer, args: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().unwrap().clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().unwrap();
    let o = o.downcast_mut::<EventsData>().unwrap();
    if args.len() < 2 {
        return Err(make_err("event.off requires 2 arguments"));
    }
    let name = to_string_base(state.clone(), args[0].clone())?;
    let f = args[1].clone();
    init_event(o, name.clone());
    let event = o.data.get_mut(&name).unwrap();
    let mut r = false;
    let mut listeners = Vec::new();
    for i in event.listeners.iter() {
        match i {
            Listener::On(a) => {
                if eql_base(state.clone(), f.clone(), a.clone())? {
                    r = true;
                } else {
                    listeners.push(i.clone());
                }
            },
            Listener::Once(a) => {
                if eql_base(state.clone(), f.clone(), a.clone())? {
                    r = true;
                } else {
                    listeners.push(i.clone());
                }
            },
        }
    }
    event.listeners = listeners;
    Ok(make_container(Value::Boolean(r)))
}

fn isactive(state: StateContainer, args: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().unwrap().clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().unwrap();
    let o = o.downcast_mut::<EventsData>().unwrap();
    if args.len() == 0 {
        return Err(make_err("event.isactive requires 1 argument"));
    }
    let name = to_string_base(state.clone(), args[0].clone())?;
    init_event(o, name.clone());
    Ok(make_container(Value::Boolean(o.data.get_mut(&name).unwrap().active)))
}

fn activate(state: StateContainer, args: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().unwrap().clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().unwrap();
    let o = o.downcast_mut::<EventsData>().unwrap();
    if args.len() == 0 {
        return Err(make_err("event.activate requires 1 argument"));
    }
    let name = to_string_base(state.clone(), args[0].clone())?;
    init_event(o, name.clone());
    o.data.get_mut(&name).unwrap().active = true;
    Ok(make_container(Value::Null))
}

fn deactivate(state: StateContainer, args: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().unwrap().clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().unwrap();
    let o = o.downcast_mut::<EventsData>().unwrap();
    if args.len() == 0 {
        return Err(make_err("event.deactivate requires 1 argument"));
    }
    let name = to_string_base(state.clone(), args[0].clone())?;
    init_event(o, name.clone());
    o.data.get_mut(&name).unwrap().active = false;
    Ok(make_container(Value::Null))
}

fn clear(state: StateContainer, args: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().unwrap().clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().unwrap();
    let o = o.downcast_mut::<EventsData>().unwrap();
    if args.len() == 0 {
        return Err(make_err("event.clear requires 1 argument"));
    }
    let name = to_string_base(state.clone(), args[0].clone())?;
    o.data.remove(&name);
    Ok(make_container(Value::Null))
}

fn emit(state: StateContainer, args: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().unwrap().clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().unwrap();
    let o = o.downcast_mut::<EventsData>().unwrap();
    if args.len() == 0 {
        return Err(make_err("event.emit requires at least 1 argument"));
    }
    let mut args = args.clone();
    let name = to_string_base(state.clone(), args.remove(0))?;
    init_event(o, name.clone());
    let event = o.data.get_mut(&name).unwrap();
    if !event.active {
        return Ok(make_container(Value::Null));
    }
    for i in event.listeners.iter() {
        match i {
            Listener::Once(f) => {
                let _ = call(state.clone(), f.clone(), args.clone());
            },
            Listener::On(f) => {
                let _ = call(state.clone(), f.clone(), args.clone());
            },
        }
    }
    event.listeners = event.listeners.iter().map(|i| i.clone()).filter(|i| {
        match i {
            Listener::Once(_) => false,
            Listener::On(_) => true,
        }
    }).collect();
    Ok(make_container(Value::Null))
}