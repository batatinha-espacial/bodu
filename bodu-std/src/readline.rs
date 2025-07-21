use std::{collections::HashMap, sync::Arc};

use bodu_vm::{make_container, make_err, op::{make_object_base, set_base, to_string_base}, Container, Function, Gi, StateContainer, Value};
use rustyline::DefaultEditor;
use tokio::sync::Mutex;

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
            caller_state: false,
        };
        fn_.internals.insert(0, $o.clone());
        set_base($state.clone(), $o.clone(), $prop.to_string(), make_container(Value::Function(fn_))).await?;
    }};
}

pub async fn new(state: StateContainer, _: Vec<Container>, _: Gi) -> Result<Container, Container> {
    let rl = DefaultEditor::new().map_err(|_| make_err("readile.new was unable to create a new readline"))?;
    let mut o = make_object_base();
    o.internals.insert(u64::MAX, make_container(Value::String("array".to_string())));
    o.externals.insert(0, Arc::new(Mutex::new(Box::new(rl))));
    let o = make_container(Value::Object(o));

    helper1!(state, readline, o, "readline");
    helper1!(state, readline_with_initial, o, "readline_with_initial");
    helper1!(state, load_history, o, "load_history");
    helper1!(state, save_history, o, "save_history");
    helper1!(state, append_history, o, "append_history");
    helper1!(state, add_history_entry, o, "add_history_entry");
    helper1!(state, clear_history, o, "clear_history");
    helper1!(state, clear_screen, o, "clear_screen");

    Ok(o)
}

async fn readline(state: StateContainer, args: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().await.clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().await;
    let o = o.downcast_mut::<DefaultEditor>().unwrap();
    if args.len() == 0 {
        return Err(make_err("readline.readline requires 1 argument"));
    }
    let s = to_string_base(state.clone(), args[0].clone()).await?;
    match o.readline(&s) {
        Err(_) => Err(make_err("readline.readline was unable to read a line")),
        Ok(s) => Ok(make_container(Value::String(s))),
    }
}

async fn readline_with_initial(state: StateContainer, args: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().await.clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().await;
    let o = o.downcast_mut::<DefaultEditor>().unwrap();
    if args.len() < 3 {
        return Err(make_err("readline.readline_with_initial requires 3 arguments"));
    }
    let s1 = to_string_base(state.clone(), args[0].clone()).await?;
    let s2 = to_string_base(state.clone(), args[1].clone()).await?;
    let s3 = to_string_base(state.clone(), args[2].clone()).await?;
    match o.readline_with_initial(&s1, (&s2, &s3)) {
        Err(_) => Err(make_err("readline.readline_with_initial was unable to read a line")),
        Ok(s) => Ok(make_container(Value::String(s))),
    }
}

async fn load_history(state: StateContainer, args: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().await.clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().await;
    let o = o.downcast_mut::<DefaultEditor>().unwrap();
    if args.len() == 0 {
        return Err(make_err("readline.load_history requires 1 argument"));
    }
    let s = to_string_base(state.clone(), args[0].clone()).await?;
    match o.load_history(&s) {
        Err(_) => Err(make_err("readline.load_history was unable to load history from file")),
        Ok(_) => Ok(make_container(Value::Null)),
    }
}

async fn save_history(state: StateContainer, args: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().await.clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().await;
    let o = o.downcast_mut::<DefaultEditor>().unwrap();
    if args.len() == 0 {
        return Err(make_err("readline.save_history requires 1 argument"));
    }
    let s = to_string_base(state.clone(), args[0].clone()).await?;
    match o.save_history(&s) {
        Err(_) => Err(make_err("readline.save_history was unable to save history to file")),
        Ok(_) => Ok(make_container(Value::Null)),
    }
}

async fn append_history(state: StateContainer, args: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().await.clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().await;
    let o = o.downcast_mut::<DefaultEditor>().unwrap();
    if args.len() == 0 {
        return Err(make_err("readline.append_history requires 1 argument"));
    }
    let s = to_string_base(state.clone(), args[0].clone()).await?;
    match o.append_history(&s) {
        Err(_) => Err(make_err("readline.append_history was unable to append history to file")),
        Ok(_) => Ok(make_container(Value::Null)),
    }
}

async fn add_history_entry(state: StateContainer, args: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().await.clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().await;
    let o = o.downcast_mut::<DefaultEditor>().unwrap();
    if args.len() == 0 {
        return Err(make_err("readline.add_history_entry requires 1 argument"));
    }
    let s = to_string_base(state.clone(), args[0].clone()).await?;
    match o.add_history_entry(&s) {
        Err(_) => Err(make_err("readline.add_history_entry was unable to add a history entry")),
        Ok(s) => Ok(make_container(Value::Boolean(s))),
    }
}

async fn clear_history(_: StateContainer, _: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().await.clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().await;
    let o = o.downcast_mut::<DefaultEditor>().unwrap();
    match o.clear_history() {
        Err(_) => Err(make_err("readline.clear_history was unable to clear history")),
        Ok(_) => Ok(make_container(Value::Null)),
    }
}

async fn clear_screen(_: StateContainer, _: Vec<Container>, gi: Gi) -> Result<Container, Container> {
    let o = gi(0).unwrap();
    let o = (match o.lock().await.clone() {
        Value::Object(o) => Some(o),
        _ => None,
    }).unwrap();
    let o = o.externals.get(&0).unwrap().clone();
    let mut o = o.lock().await;
    let o = o.downcast_mut::<DefaultEditor>().unwrap();
    match o.clear_screen() {
        Err(_) => Err(make_err("readline.clear_screen was unable to clear the screen")),
        Ok(_) => Ok(make_container(Value::Null)),
    }
}