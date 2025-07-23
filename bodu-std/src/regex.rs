use bodu_vm::{make_container, make_err, op::{make_object, set_base, to_number_base, to_string_base}, Container, Gi, StateContainer, Value};
use regex::{Captures, Match, Regex};

use crate::array::new_with_vec;

async fn lookup_regex(state: StateContainer, s: String) -> Result<Regex, Container> {
    let global = state.lock().await.globaldata.clone().unwrap();
    let regmap = &mut global.lock().await.regex;
    match regmap.get(&s) {
        Some(s) => s.clone().map_err(|_| make_err("can't parse regex")),
        None => {
            let r = Regex::new(&s).map_err(|_| ());
            regmap.insert(s.clone(), r.clone());
            r.map_err(|_| make_err("can't parse regex"))
        },
    }
}

pub async fn is_match(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("regex.is_match requires 2 arguments"));
    }
    let reg = to_string_base(state.clone(), args[0].clone()).await?;
    let reg = lookup_regex(state.clone(), reg).await?;
    let haystack = to_string_base(state.clone(), args[1].clone()).await?;
    Ok(make_container(Value::Boolean(reg.is_match(&haystack))))
}

async fn make_match(state: StateContainer, match_: Match<'_>) -> Result<Container, Container> {
    let start = match_.start();
    let end = match_.end();
    let empty = match_.is_empty();
    let len = match_.len();
    let str = match_.as_str().to_string();

    let obj = make_object();
    set_base(state.clone(), obj.clone(), "start".to_string(), make_container(Value::Number(start as i64))).await?;
    set_base(state.clone(), obj.clone(), "end".to_string(), make_container(Value::Number(end as i64))).await?;
    set_base(state.clone(), obj.clone(), "empty".to_string(), make_container(Value::Boolean(empty))).await?;
    set_base(state.clone(), obj.clone(), "len".to_string(), make_container(Value::Number(len as i64))).await?;
    set_base(state.clone(), obj.clone(), "str".to_string(), make_container(Value::String(str))).await?;
    Ok(obj)
}

pub async fn find(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("regex.find requires 2 arguments"));
    }
    let reg = to_string_base(state.clone(), args[0].clone()).await?;
    let reg = lookup_regex(state.clone(), reg).await?;
    let haystack = to_string_base(state.clone(), args[1].clone()).await?;
    let r = reg.find(&haystack);
    match r {
        None => Ok(make_container(Value::Null)),
        Some(m) => Ok(make_match(state.clone(), m).await?),
    }
}

pub async fn find_many(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("regex.find_many requires 2 arguments"));
    }
    let reg = to_string_base(state.clone(), args[0].clone()).await?;
    let reg = lookup_regex(state.clone(), reg).await?;
    let haystack = to_string_base(state.clone(), args[1].clone()).await?;
    let i = reg.find_iter(&haystack);
    let mut i2 = Vec::new();
    for j in i {
        i2.push(make_match(state.clone(), j).await?);
    }
    new_with_vec(state.clone(), i2).await
}

async fn make_capture(state: StateContainer, cap: Captures<'_>) -> Result<Container, Container> {
    let it = cap.iter();
    let mut i2 = Vec::new();
    for i in it {
        i2.push(match i {
            None => make_container(Value::Null),
            Some(m) => make_match(state.clone(), m).await?,
        });
    }
    new_with_vec(state.clone(), i2).await
}

pub async fn captures(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("regex.captures requires 2 arguments"));
    }
    let reg = to_string_base(state.clone(), args[0].clone()).await?;
    let reg = lookup_regex(state.clone(), reg).await?;
    let haystack = to_string_base(state.clone(), args[1].clone()).await?;
    match reg.captures(&haystack) {
        Some(cap) => make_capture(state.clone(), cap).await,
        None => Ok(make_container(Value::Null)),
    }
}

pub async fn captures_many(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("regex.captures_many requires 2 arguments"));
    }
    let reg = to_string_base(state.clone(), args[0].clone()).await?;
    let reg = lookup_regex(state.clone(), reg).await?;
    let haystack = to_string_base(state.clone(), args[1].clone()).await?;
    let i = reg.captures_iter(&haystack);
    let mut i2 = Vec::new();
    for j in i {
        i2.push(make_capture(state.clone(), j).await?);
    }
    new_with_vec(state.clone(), i2).await
}

pub async fn split(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("regex.split requires 2 arguments"));
    }
    let reg = to_string_base(state.clone(), args[0].clone()).await?;
    let reg = lookup_regex(state.clone(), reg).await?;
    let haystack = to_string_base(state.clone(), args[1].clone()).await?;
    let r = reg.split(&haystack);
    let r = r.map(|a| make_container(Value::String(a.to_string()))).collect::<Vec<_>>();
    new_with_vec(state.clone(), r).await
}

pub async fn splitn(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 3 {
        return Err(make_err("regex.splitn requires 3 arguments"));
    }
    let reg = to_string_base(state.clone(), args[0].clone()).await?;
    let reg = lookup_regex(state.clone(), reg).await?;
    let haystack = to_string_base(state.clone(), args[1].clone()).await?;
    let limit = to_number_base(state.clone(), args[2].clone()).await?;
    let r = reg.splitn(&haystack, limit as usize);
    let r = r.map(|a| make_container(Value::String(a.to_string()))).collect::<Vec<_>>();
    new_with_vec(state.clone(), r).await
}

pub async fn replace(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 3 {
        return Err(make_err("regex.replace requires 3 arguments"));
    }
    let reg = to_string_base(state.clone(), args[0].clone()).await?;
    let reg = lookup_regex(state.clone(), reg).await?;
    let haystack = to_string_base(state.clone(), args[1].clone()).await?;
    let rep = to_string_base(state.clone(), args[2].clone()).await?;
    let r = reg.replace(&haystack, rep).to_string();
    Ok(make_container(Value::String(r)))
}

pub async fn replace_all(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 3 {
        return Err(make_err("regex.replace_all requires 3 arguments"));
    }
    let reg = to_string_base(state.clone(), args[0].clone()).await?;
    let reg = lookup_regex(state.clone(), reg).await?;
    let haystack = to_string_base(state.clone(), args[1].clone()).await?;
    let rep = to_string_base(state.clone(), args[2].clone()).await?;
    let r = reg.replace_all(&haystack, rep).to_string();
    Ok(make_container(Value::String(r)))
}

pub async fn replacen(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 4 {
        return Err(make_err("regex.replacen requires 4 arguments"));
    }
    let reg = to_string_base(state.clone(), args[0].clone()).await?;
    let reg = lookup_regex(state.clone(), reg).await?;
    let haystack = to_string_base(state.clone(), args[1].clone()).await?;
    let limit = to_number_base(state.clone(), args[2].clone()).await?;
    let rep = to_string_base(state.clone(), args[3].clone()).await?;
    let r = reg.replacen(&haystack, limit as usize, rep).to_string();
    Ok(make_container(Value::String(r)))
}