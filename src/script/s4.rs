use crate::{script::s3::{ConditionType, LoopType, S3T}, vm::{Instruction, Label, Operator, VarIndex}};

pub fn s4(input: Vec<S3T>) -> Result<Vec<Instruction>, String> {
    let mut tempi: u64 = 1; // outi = 0
    let mut labeli: u64 = 1; // outli = 0
    let mut res = Vec::new();
    for i in input {
        stat(i, &mut res, &mut tempi, &mut labeli, 0, 0)?;
    }
    res.push(Instruction::Label(Label::Unnamed(0)));
    res.push(Instruction::Return(VarIndex::Temp(0)));
    Ok(res)
}

fn stat(v: S3T, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<(), String> {
    Ok(match v {
        S3T::Let(name, e) => let_(name, e, res, tempi, labeli, outi, outli)?,
        S3T::Out(v) => out(*v, res, tempi, labeli, outi, outli)?,
        S3T::Label(a) => label(a, res)?,
        S3T::Return(v) => return_(*v, res, tempi, labeli, outi, outli)?,
        S3T::Throw(v) => throw(*v, res, tempi, labeli, outi, outli)?,
        S3T::WhileUntil(a, b, c) => while_(a, b, c, res, tempi, labeli)?,
        S3T::Defer(v) => defer(v, res, tempi, labeli)?,
        S3T::Bind(name, v) => bind(name, v, res, tempi)?,
        S3T::Assign(a, b) => assign(a, b, res, tempi, labeli, outi, outli)?,
        S3T::Detuple(a, b) => detuple(a, b, res, tempi, labeli, outi, outli)?,
        S3T::LetDetuple(a, b) => let_detuple(a, b, res, tempi, labeli, outi, outli)?,
        S3T::Decorator(d, f) => decorator(d, f, res, tempi, labeli, outi, outli)?,
        S3T::MultiLet(s) => multilet(s, res)?,
        S3T::Function(a, b, c) => {
            let i = fn_(b, c, res, tempi)?;
            if let Some(name) = a {
                res.push(Instruction::Decl(VarIndex::Ident(name.clone())));
                res.push(Instruction::Assign(VarIndex::Ident(name.clone()), i));
            }
        },
        _ => {
            expr(v, res, tempi, labeli, outi, outli)?;
        },
    })
}

fn expr(v: S3T, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<VarIndex, String> {
    match v {
        S3T::Identifier(v) => identifier(v),
        S3T::If(a, b) => if_(a, b, res, tempi, labeli),
        S3T::Boolean(a) => boolean(a, res, tempi),
        S3T::Block(v) => block(v, res, tempi, labeli),
        S3T::Number(a) => number(a, res, tempi),
        S3T::Float(a) => float(a, res, tempi),
        S3T::String(a) => string(a, res, tempi),
        S3T::TryCatchFinally(a, b, c, d) => try_catch_finally(a, b, c, d, res, tempi, labeli),
        S3T::Loop(a) => loop_(a, res, tempi, labeli),
        S3T::Function(_, b, c) => fn_(b, c, res, tempi),
        S3T::Plus(a, b) => plus(a, b, res, tempi, labeli, outi, outli),
        S3T::Minus(a, b) => minus(a, b, res, tempi, labeli, outi, outli),
        S3T::Negate(a) => negate(a, res, tempi, labeli, outi, outli),
        S3T::Times(a, b) => times(a, b, res, tempi, labeli, outi, outli),
        S3T::Divide(a, b) => divide(a, b, res, tempi, labeli, outi, outli),
        S3T::Modulus(a, b) => remainder(a, b, res, tempi, labeli, outi, outli),
        S3T::Ternary(a, b, c) => ternary(a, b, c, res, tempi, labeli),
        S3T::EqualTo(a, b) => eql(a, b, res, tempi, labeli, outi, outli),
        S3T::NotEqualTo(a, b) => neql(a, b, res, tempi, labeli, outi, outli),
        S3T::Not(a) => not(a, res, tempi, labeli, outi, outli),
        S3T::Greater(a, b) => gt(a, b, res, tempi, labeli, outi, outli),
        S3T::GreaterOrEqual(a, b) => ge(a, b, res, tempi, labeli, outi, outli),
        S3T::Less(a, b) => lt(a, b, res, tempi, labeli, outi, outli),
        S3T::LessOrEqual(a, b) => le(a, b, res, tempi, labeli, outi, outli),
        S3T::And(a, b) => and(a, b, res, tempi, labeli, outi, outli),
        S3T::Or(a, b) => or(a, b, res, tempi, labeli, outi, outli),
        S3T::Xor(a, b) => xor(a, b, res, tempi, labeli, outi, outli),
        S3T::Property(a, b) => prop(a, b, res, tempi, labeli, outi, outli),
        S3T::Tuple(a) => tuple(a, res, tempi, labeli, outi, outli),
        S3T::FnCall(a, b) => fn_call(a, b, res, tempi, labeli, outi, outli),
        S3T::Pipe(a, b) => pipe(a, b, res, tempi, labeli, outi, outli),
        S3T::OrThat(a, b) => orthat(a, b, res, tempi, labeli, outi, outli),
        S3T::OperatorFn(op) => operatorfn(op, res, tempi),
        S3T::Null => null(tempi),
        S3T::PipeShorthand => pipe_shorthand(res, tempi),
        _ => Err("invalid expression".to_string()),
    }
}

fn identifier(v: String) -> Result<VarIndex, String> {
    Ok(VarIndex::Ident(v.clone()))
}

fn let_(name: String, e: Option<Box<S3T>>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<(), String> {
    if let Some(e) = e {
        let e = expr(*e, res, tempi, labeli, outi, outli)?;
        res.push(Instruction::Decl(VarIndex::Ident(name.clone())));
        res.push(Instruction::Assign(VarIndex::Ident(name.clone()), e));
        Ok(())
    } else {
        res.push(Instruction::Decl(VarIndex::Ident(name.clone())));
        Ok(())
    }
}

fn if_(ifs: Vec<(ConditionType, Box<S3T>, Vec<S3T>)>, else_: Option<Vec<S3T>>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64) -> Result<VarIndex, String> {
    let ifs = ifs.iter().map(|v| {
        match v.0 {
            ConditionType::If => (v.1.clone(), v.2.clone()),
            ConditionType::Unless => (Box::new(S3T::Not(v.1.clone())), v.2.clone()),
        }
    }).collect::<Vec<_>>();
    let after = *labeli;
    *labeli += 1;
    let iouti = *tempi;
    *tempi += 1;
    let ifs = ifs.iter().map(|v| {
        let mut vec_ = Vec::new();
        for i in v.1.clone() {
            stat(i, &mut vec_, tempi, labeli, iouti, after)?;
        }
        Ok::<_, String>((v.0.clone(), vec_))
    }).collect::<Vec<_>>();
    let mut ifs2 = Vec::new();
    for i in ifs {
        ifs2.push(i?);
    }
    let ifs = ifs2;
    let mut labels = Vec::new();
    for _ in ifs.clone() {
        labels.push(*labeli);
        *labeli += 1;
    }
    for i in labels.iter().zip(ifs.clone()) {
        let tempi1 = expr(*i.1.0, res, tempi, labeli, iouti, after)?;
        res.push(Instruction::GotoIf(Label::Unnamed(*i.0), tempi1));
    }
    let mut else_label = 0;
    match else_ {
        None => res.push(Instruction::Goto(Label::Unnamed(after))),
        Some(_) => {
            else_label = *labeli;
            *labeli += 1;
            res.push(Instruction::Goto(Label::Unnamed(else_label)));
        },
    }
    for i in labels.iter().zip(ifs.clone()) {
        res.push(Instruction::Label(Label::Unnamed(*i.0)));
        res.push(Instruction::Block(i.1.1));
        res.push(Instruction::Goto(Label::Unnamed(after)));
    }
    match else_ {
        None => {},
        Some(v) => {
            res.push(Instruction::Label(Label::Unnamed(else_label)));
            let mut vec_ = Vec::new();
            for i in v {
                stat(i, &mut vec_, tempi, labeli, iouti, after)?;
            }
            res.push(Instruction::Block(vec_));
            res.push(Instruction::Goto(Label::Unnamed(after)));
        },
    }
    res.push(Instruction::Label(Label::Unnamed(after)));
    Ok(VarIndex::Temp(iouti))
}

fn boolean(v: bool, res: &mut Vec<Instruction>, tempi: &mut u64) -> Result<VarIndex, String> {
    let i = *tempi;
    *tempi += 1;
    res.push(Instruction::Boolean(VarIndex::Temp(i), v));
    Ok(VarIndex::Temp(i))
}

fn block(v: Vec<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64) -> Result<VarIndex, String> {
    let outli = *labeli;
    *labeli += 1;
    let outi = *tempi;
    *tempi += 1;
    let mut vec_ = Vec::new();
    for i in v {
        stat(i, &mut vec_, tempi, labeli, outi, outli)?;
    }
    res.push(Instruction::Block(vec_));
    res.push(Instruction::Label(Label::Unnamed(outli)));
    Ok(VarIndex::Temp(outi))
}

fn out(v: S3T, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<(), String> {
    let v = expr(v, res, tempi, labeli, outi, outli)?;
    res.push(Instruction::Assign(VarIndex::Temp(outi), v));
    res.push(Instruction::Goto(Label::Unnamed(outli)));
    Ok(())
}

fn label(v: String, res: &mut Vec<Instruction>) -> Result<(), String> {
    res.push(Instruction::Label(Label::Named(v.clone())));
    Ok(())
}

fn number(v: i64, res: &mut Vec<Instruction>, tempi: &mut u64) -> Result<VarIndex, String> {
    let i = *tempi;
    *tempi += 1;
    res.push(Instruction::Number(VarIndex::Temp(i), v));
    Ok(VarIndex::Temp(i))
}

fn float(v: f64, res: &mut Vec<Instruction>, tempi: &mut u64) -> Result<VarIndex, String> {
    let i = *tempi;
    *tempi += 1;
    res.push(Instruction::Float(VarIndex::Temp(i), v));
    Ok(VarIndex::Temp(i))
}

fn string(v: String, res: &mut Vec<Instruction>, tempi: &mut u64) -> Result<VarIndex, String> {
    let i = *tempi;
    *tempi += 1;
    res.push(Instruction::String(VarIndex::Temp(i), v.clone()));
    Ok(VarIndex::Temp(i))
}

fn try_catch_finally(try_body: Vec<S3T>, name: String, catch_body: Vec<S3T>, finally_body: Option<Vec<S3T>>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64) -> Result<VarIndex, String> {
    let outli = *labeli;
    *labeli += 1;
    let outi = *tempi;
    *tempi += 1;
    let is_err = *tempi;
    *tempi += 1;
    let err = *tempi;
    *tempi += 1;
    let catchli = *labeli;
    *labeli += 1;
    let finallyli = match finally_body {
        Some(_) => {
            let fli = *labeli;
            *labeli += 1;
            fli
        },
        None => outli,
    };
    let mut try_vec = Vec::new();
    for i in try_body {
        stat(i, &mut try_vec, tempi, labeli, outi, finallyli)?;
    }
    res.push(Instruction::Catch(VarIndex::Temp(is_err), VarIndex::Temp(err), try_vec));
    res.push(Instruction::GotoIf(Label::Unnamed(catchli), VarIndex::Temp(is_err)));
    res.push(Instruction::Goto(Label::Unnamed(finallyli)));
    res.push(Instruction::Label(Label::Unnamed(catchli)));
    let mut catch_vec = vec![Instruction::Decl(VarIndex::Ident(name.clone())), Instruction::Assign(VarIndex::Ident(name.clone()), VarIndex::Temp(err))];
    for i in catch_body {
        stat(i, &mut catch_vec, tempi, labeli, outi, outli)?;
    }
    res.push(Instruction::Block(catch_vec));
    res.push(Instruction::Goto(Label::Unnamed(outli)));
    match finally_body {
        None => {},
        Some(finally_body) => {
            res.push(Instruction::Label(Label::Unnamed(finallyli)));
            let mut finally_vec = Vec::new();
            for i in finally_body {
                stat(i, &mut finally_vec, tempi, labeli, outi, outli)?;
            }
            res.push(Instruction::Block(finally_vec));
            res.push(Instruction::Goto(Label::Unnamed(outli)));
        },
    }
    res.push(Instruction::Label(Label::Unnamed(outli)));
    Ok(VarIndex::Temp(outi))
}

fn return_(v: S3T, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<(), String> {
    let v = expr(v, res, tempi, labeli, outi, outli)?;
    res.push(Instruction::Return(v));
    Ok(())
}

fn throw(v: S3T, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<(), String> {
    let v = expr(v, res, tempi, labeli, outi, outli)?;
    res.push(Instruction::Throw(v));
    Ok(())
}

fn loop_(v: Vec<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64) -> Result<VarIndex, String> {
    let outi = *tempi;
    *tempi += 1;
    let outli = *labeli;
    *labeli += 1;
    let loopli = *labeli;
    *labeli += 1;
    res.push(Instruction::Goto(Label::Unnamed(loopli)));
    res.push(Instruction::Label(Label::Unnamed(loopli)));
    let mut vec_ = Vec::new();
    for i in v {
        stat(i, &mut vec_, tempi, labeli, outi, outli)?;
    }
    res.push(Instruction::Block(vec_));
    res.push(Instruction::Goto(Label::Unnamed(loopli)));
    res.push(Instruction::Label(Label::Unnamed(outli)));
    Ok(VarIndex::Temp(outi))
}

fn while_(type_: LoopType, cond: Box<S3T>, body: Vec<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64) -> Result<(), String> {
    let cond = match type_ {
        LoopType::While => *cond,
        LoopType::Until => S3T::Not(cond),
    };
    let condli = *labeli;
    *labeli += 1;
    let loopli = *labeli;
    *labeli += 1;
    let outli = *labeli;
    *labeli += 1;
    let outi = *tempi;
    *tempi += 1;
    res.push(Instruction::Goto(Label::Unnamed(condli)));
    res.push(Instruction::Label(Label::Unnamed(condli)));
    let cond = expr(cond, res, tempi, labeli, outi, outli)?;
    res.push(Instruction::GotoIf(Label::Unnamed(loopli), cond));
    res.push(Instruction::Goto(Label::Unnamed(outli)));
    res.push(Instruction::Label(Label::Unnamed(loopli)));
    let mut vec_ = Vec::new();
    for i in body {
        stat(i, &mut vec_, tempi, labeli, outi, outli)?;
    }
    res.push(Instruction::Block(vec_));
    res.push(Instruction::Goto(Label::Unnamed(condli)));
    res.push(Instruction::Label(Label::Unnamed(outli)));
    Ok(())
}

fn defer(body: Vec<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64) -> Result<(), String> {
    let outli = *labeli;
    *labeli += 1;
    let outi = *tempi;
    *tempi += 1;
    let mut vec_ = Vec::new();
    for i in body {
        stat(i, &mut vec_, tempi, labeli, outi, outli)?;
    }
    res.push(Instruction::Defer(vec![Instruction::Block(vec_), Instruction::Label(Label::Unnamed(outli))]));
    Ok(())
}

fn bind(name: String, v: Box<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64) -> Result<(), String> {
    let f = fn_(vec![], vec![S3T::Return(v)], res, tempi)?;
    let vi = *tempi;
    *tempi += 1;
    res.push(Instruction::MakeBind(VarIndex::Temp(vi), f));
    res.push(Instruction::Decl(VarIndex::Ident(name.clone())));
    res.push(Instruction::Assign(VarIndex::Ident(name.clone()), VarIndex::Temp(vi)));
    Ok(())
}

fn fn_(args: Vec<String>, body: Vec<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64) -> Result<VarIndex, String> {
    let mut tempi2: u64 = 1; // outi = 0
    let mut labeli2: u64 = 1; // outli = 0
    let mut res2 = Vec::new();
    for i in args.iter().enumerate() {
        res2.push(Instruction::Decl(VarIndex::Ident(i.1.clone())));
        res2.push(Instruction::Assign(VarIndex::Ident(i.1.clone()), VarIndex::Arg(i.0)));
    }
    for i in body {
        stat(i, &mut res2, &mut tempi2, &mut labeli2, 0, 0)?;
    }
    res2.push(Instruction::Label(Label::Unnamed(0)));
    res2.push(Instruction::Return(VarIndex::Temp(0)));
    let vi = *tempi;
    *tempi += 1;
    res.push(Instruction::MakeFunction(VarIndex::Temp(vi), res2));
    Ok(VarIndex::Temp(vi))
}


fn assign(left: Box<S3T>, right: Box<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<(), String> {
    let right = expr(*right, res, tempi, labeli, outi, outli)?;
    match *left {
        S3T::Identifier(s) => res.push(Instruction::Assign(VarIndex::Ident(s), right)),
        S3T::Property(obj, prop) => {
            let res_ = *tempi;
            *tempi += 1;
            let obj = expr(*obj, res, tempi, labeli, outi, outli)?;
            let prop = expr(*prop, res, tempi, labeli, outi, outli)?;
            res.push(Instruction::Set(VarIndex::Temp(res_), obj, prop, right));
        },
        _ => return Err("invalid assignment".to_string())
    }
    Ok(())
}

fn plus(left: Box<S3T>, right: Box<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<VarIndex, String> {
    let left = expr(*left, res, tempi, labeli, outi, outli)?;
    let right = expr(*right, res, tempi, labeli, outi, outli)?;
    let vi = *tempi;
    *tempi += 1;
    res.push(Instruction::Add(VarIndex::Temp(vi), left, right));
    Ok(VarIndex::Temp(vi))
}

fn minus(left: Box<S3T>, right: Box<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<VarIndex, String> {
    let left = expr(*left, res, tempi, labeli, outi, outli)?;
    let right = expr(*right, res, tempi, labeli, outi, outli)?;
    let vi = *tempi;
    *tempi += 1;
    res.push(Instruction::Subtract(VarIndex::Temp(vi), left, right));
    Ok(VarIndex::Temp(vi))
}

fn negate(v: Box<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<VarIndex, String> {
    let v = expr(*v, res, tempi, labeli, outi, outli)?;
    let vi = *tempi;
    *tempi += 1;
    res.push(Instruction::Negate(VarIndex::Temp(vi), v));
    Ok(VarIndex::Temp(vi))
}

fn times(left: Box<S3T>, right: Box<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<VarIndex, String> {
    let left = expr(*left, res, tempi, labeli, outi, outli)?;
    let right = expr(*right, res, tempi, labeli, outi, outli)?;
    let vi = *tempi;
    *tempi += 1;
    res.push(Instruction::Multiply(VarIndex::Temp(vi), left, right));
    Ok(VarIndex::Temp(vi))
}

fn divide(left: Box<S3T>, right: Box<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<VarIndex, String> {
    let left = expr(*left, res, tempi, labeli, outi, outli)?;
    let right = expr(*right, res, tempi, labeli, outi, outli)?;
    let vi = *tempi;
    *tempi += 1;
    res.push(Instruction::Divide(VarIndex::Temp(vi), left, right));
    Ok(VarIndex::Temp(vi))
}

fn remainder(left: Box<S3T>, right: Box<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<VarIndex, String> {
    let left = expr(*left, res, tempi, labeli, outi, outli)?;
    let right = expr(*right, res, tempi, labeli, outi, outli)?;
    let vi = *tempi;
    *tempi += 1;
    res.push(Instruction::Remainder(VarIndex::Temp(vi), left, right));
    Ok(VarIndex::Temp(vi))
}

fn ternary(cond: Box<S3T>, i: Box<S3T>, e: Box<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64) -> Result<VarIndex, String> {
    let ifs = vec![(ConditionType::If, cond, vec![S3T::Out(i)])];
    let else_ = Some(vec![S3T::Out(e)]);
    if_(ifs, else_, res, tempi, labeli)
}

fn eql(left: Box<S3T>, right: Box<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<VarIndex, String> {
    let left = expr(*left, res, tempi, labeli, outi, outli)?;
    let right = expr(*right, res, tempi, labeli, outi, outli)?;
    let vi = *tempi;
    *tempi += 1;
    res.push(Instruction::Eql(VarIndex::Temp(vi), left, right));
    Ok(VarIndex::Temp(vi))
}

fn not(v: Box<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<VarIndex, String> {
    let v = expr(*v, res, tempi, labeli, outi, outli)?;
    let vi = *tempi;
    *tempi += 1;
    res.push(Instruction::Not(VarIndex::Temp(vi), v));
    Ok(VarIndex::Temp(vi))
}

fn neql(left: Box<S3T>, right: Box<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<VarIndex, String> {
    let left = expr(*left, res, tempi, labeli, outi, outli)?;
    let right = expr(*right, res, tempi, labeli, outi, outli)?;
    let vi = *tempi;
    *tempi += 1;
    res.push(Instruction::Neql(VarIndex::Temp(vi), left, right));
    Ok(VarIndex::Temp(vi))
}

fn gt(left: Box<S3T>, right: Box<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<VarIndex, String> {
    let left = expr(*left, res, tempi, labeli, outi, outli)?;
    let right = expr(*right, res, tempi, labeli, outi, outli)?;
    let vi = *tempi;
    *tempi += 1;
    res.push(Instruction::Gt(VarIndex::Temp(vi), left, right));
    Ok(VarIndex::Temp(vi))
}

fn ge(left: Box<S3T>, right: Box<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<VarIndex, String> {
    let left = expr(*left, res, tempi, labeli, outi, outli)?;
    let right = expr(*right, res, tempi, labeli, outi, outli)?;
    let vi = *tempi;
    *tempi += 1;
    res.push(Instruction::Ge(VarIndex::Temp(vi), left, right));
    Ok(VarIndex::Temp(vi))
}

fn lt(left: Box<S3T>, right: Box<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<VarIndex, String> {
    let left = expr(*left, res, tempi, labeli, outi, outli)?;
    let right = expr(*right, res, tempi, labeli, outi, outli)?;
    let vi = *tempi;
    *tempi += 1;
    res.push(Instruction::Lt(VarIndex::Temp(vi), left, right));
    Ok(VarIndex::Temp(vi))
}

fn le(left: Box<S3T>, right: Box<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<VarIndex, String> {
    let left = expr(*left, res, tempi, labeli, outi, outli)?;
    let right = expr(*right, res, tempi, labeli, outi, outli)?;
    let vi = *tempi;
    *tempi += 1;
    res.push(Instruction::Le(VarIndex::Temp(vi), left, right));
    Ok(VarIndex::Temp(vi))
}

fn and(left: Box<S3T>, right: Box<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<VarIndex, String> {
    let left = expr(*left, res, tempi, labeli, outi, outli)?;
    let right = expr(*right, res, tempi, labeli, outi, outli)?;
    let vi = *tempi;
    *tempi += 1;
    res.push(Instruction::And(VarIndex::Temp(vi), left, right));
    Ok(VarIndex::Temp(vi))
}

fn or(left: Box<S3T>, right: Box<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<VarIndex, String> {
    let left = expr(*left, res, tempi, labeli, outi, outli)?;
    let right = expr(*right, res, tempi, labeli, outi, outli)?;
    let vi = *tempi;
    *tempi += 1;
    res.push(Instruction::Or(VarIndex::Temp(vi), left, right));
    Ok(VarIndex::Temp(vi))
}

fn xor(left: Box<S3T>, right: Box<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<VarIndex, String> {
    let left = expr(*left, res, tempi, labeli, outi, outli)?;
    let right = expr(*right, res, tempi, labeli, outi, outli)?;
    let vi = *tempi;
    *tempi += 1;
    res.push(Instruction::Xor(VarIndex::Temp(vi), left, right));
    Ok(VarIndex::Temp(vi))
}

fn prop(left: Box<S3T>, right: Box<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<VarIndex, String> {
    let left = expr(*left, res, tempi, labeli, outi, outli)?;
    let right = expr(*right, res, tempi, labeli, outi, outli)?;
    let vi = *tempi;
    *tempi += 1;
    res.push(Instruction::Get(VarIndex::Temp(vi), left, right));
    Ok(VarIndex::Temp(vi))
}

fn tuple(v: Vec<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<VarIndex, String> {
    let mut v2 = Vec::new();
    for i in v {
        v2.push(expr(i, res, tempi, labeli, outi, outli)?);
    }
    let v = v2;
    let vi = *tempi;
    *tempi += 1;
    res.push(Instruction::MakeTuple(VarIndex::Temp(vi), v));
    Ok(VarIndex::Temp(vi))
}

fn detuple(r: Vec<S3T>, v: Box<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<(), String> {
    let vi = r.iter().map(|_| {
        let i = *tempi;
        *tempi += 1;
        VarIndex::Temp(i)
    }).collect::<Vec<_>>();
    let v = expr(*v, res, tempi, labeli, outi, outli)?;
    res.push(Instruction::DeTuple(vi.clone(), v));
    for i in r.into_iter().zip(vi) {
        match i.0 {
            S3T::Identifier(s) => res.push(Instruction::Assign(VarIndex::Ident(s), i.1)),
            S3T::Property(obj, prop) => {
                let res_ = *tempi;
                *tempi += 1;
                let obj = expr(*obj, res, tempi, labeli, outi, outli)?;
                let prop = expr(*prop, res, tempi, labeli, outi, outli)?;
                res.push(Instruction::Set(VarIndex::Temp(res_), obj, prop, i.1));
            },
            _ => return Err("invalid assignment".to_string())
        }
    }
    Ok(())
}

fn let_detuple(r: Vec<String>, v: Box<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<(), String> {
    let vi = r.iter().map(|_| {
        let i = *tempi;
        *tempi += 1;
        VarIndex::Temp(i)
    }).collect::<Vec<_>>();
    let v = expr(*v, res, tempi, labeli, outi, outli)?;
    res.push(Instruction::DeTuple(vi.clone(), v));
    for i in r.into_iter().zip(vi) {
        res.push(Instruction::Decl(VarIndex::Ident(i.0.clone())));
        res.push(Instruction::Assign(VarIndex::Ident(i.0.clone()), i.1));
    }
    Ok(())
}

fn fn_call(v: Box<S3T>, args: Vec<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<VarIndex, String> {
    let v = expr(*v, res, tempi, labeli, outi, outli)?;
    let mut args2 = Vec::new();
    for i in args {
        args2.push(expr(i, res, tempi, labeli, outi, outli)?);
    }
    let args = args2;
    let vi = *tempi;
    *tempi += 1;
    res.push(Instruction::Call(VarIndex::Temp(vi), v, args));
    Ok(VarIndex::Temp(vi))
}

fn decorator(d: Box<S3T>, f: Box<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<(), String> {
    let d = expr(*d, res, tempi, labeli, outi, outli)?;
    match *f {
        S3T::Function(Some(name), args, body) => {
            let i = fn_(args, body, res, tempi)?;
            let i2 = *tempi;
            *tempi += 1;
            res.push(Instruction::Call(VarIndex::Temp(i2), d, vec![i]));
            res.push(Instruction::Decl(VarIndex::Ident(name.clone())));
            res.push(Instruction::Assign(VarIndex::Ident(name.clone()), VarIndex::Temp(i2)));
            Ok(())
        },
        _ => Err("decorators can only be used with named functions".to_string()),
    }
}

fn includes_fnshorthand(v: Box<S3T>) -> bool {
    match *v {
        S3T::Let(_, v) => {
            if let Some(v) = v {
                includes_fnshorthand(v)
            } else {
                false
            }
        },
        S3T::If(vec_, v) => {
            let mut b = false;
            for i in vec_ {
                b = b || includes_fnshorthand(i.1);
                for i in i.2 {
                    b = b || includes_fnshorthand(Box::new(i));
                }
            }
            if let Some(v) = v {
                for i in v {
                    b = b || includes_fnshorthand(Box::new(i));
                }
            }
            b
        },
        S3T::Block(v) => {
            let mut b = false;
            for i in v {
                b = b || includes_fnshorthand(Box::new(i));
            }
            b
        },
        S3T::Out(v) => includes_fnshorthand(v),
        S3T::TryCatchFinally(v1, _, v2, v3) => {
            let mut b = false;
            for i in v1 {
                b = b || includes_fnshorthand(Box::new(i));
            }
            for i in v2 {
                b = b || includes_fnshorthand(Box::new(i));
            }
            if let Some(v3) = v3 {
                for i in v3 {
                    b = b || includes_fnshorthand(Box::new(i));
                }
            }
            b
        },
        S3T::Return(v) => includes_fnshorthand(v),
        S3T::Throw(v) => includes_fnshorthand(v),
        S3T::Loop(v) => {
            let mut b = false;
            for i in v {
                b = b || includes_fnshorthand(Box::new(i));
            }
            b
        },
        S3T::WhileUntil(_, v1, v2) => {
            let mut b = includes_fnshorthand(v1);
            for i in v2 {
                b = b || includes_fnshorthand(Box::new(i));
            }
            b
        },
        S3T::Defer(v) => {
            let mut b = false;
            for i in v {
                b = b || includes_fnshorthand(Box::new(i));
            }
            b
        },
        S3T::Bind(_, v) => includes_fnshorthand(v),
        S3T::Assign(v1, v2) => includes_fnshorthand(v1) || includes_fnshorthand(v2),
        S3T::Plus(v1, v2) => includes_fnshorthand(v1) || includes_fnshorthand(v2),
        S3T::Minus(v1, v2) => includes_fnshorthand(v1) || includes_fnshorthand(v2),
        S3T::Negate(v) => includes_fnshorthand(v),
        S3T::Times(v1, v2) => includes_fnshorthand(v1) || includes_fnshorthand(v2),
        S3T::Divide(v1, v2) => includes_fnshorthand(v1) || includes_fnshorthand(v2),
        S3T::Modulus(v1, v2) => includes_fnshorthand(v1) || includes_fnshorthand(v2),
        S3T::Ternary(v1, v2, v3) => includes_fnshorthand(v1) || includes_fnshorthand(v2) || includes_fnshorthand(v3),
        S3T::EqualTo(v1, v2) => includes_fnshorthand(v1) || includes_fnshorthand(v2),
        S3T::Not(v) => includes_fnshorthand(v),
        S3T::NotEqualTo(v1, v2) => includes_fnshorthand(v1) || includes_fnshorthand(v2),
        S3T::Less(v1, v2) => includes_fnshorthand(v1) || includes_fnshorthand(v2),
        S3T::LessOrEqual(v1, v2) => includes_fnshorthand(v1) || includes_fnshorthand(v2),
        S3T::Greater(v1, v2) => includes_fnshorthand(v1) || includes_fnshorthand(v2),
        S3T::GreaterOrEqual(v1, v2) => includes_fnshorthand(v1) || includes_fnshorthand(v2),
        S3T::And(v1, v2) => includes_fnshorthand(v1) || includes_fnshorthand(v2),
        S3T::Or(v1, v2) => includes_fnshorthand(v1) || includes_fnshorthand(v2),
        S3T::Xor(v1, v2) => includes_fnshorthand(v1) || includes_fnshorthand(v2),
        S3T::Property(v1, v2) => includes_fnshorthand(v1) || includes_fnshorthand(v2),
        S3T::Tuple(v) => {
            let mut b = false;
            for i in v {
                b = b || includes_fnshorthand(Box::new(i));
            }
            b
        },
        S3T::Detuple(v1, v2) => {
            let mut b = includes_fnshorthand(v2);
            for i in v1 {
                b = b || includes_fnshorthand(Box::new(i));
            }
            b
        },
        S3T::LetDetuple(_, v) => includes_fnshorthand(v),
        S3T::FnCall(v1, v2) => {
            let mut b = includes_fnshorthand(v1);
            for i in v2 {
                b = b || includes_fnshorthand(Box::new(i));
            }
            b
        },
        S3T::Decorator(v, _) => includes_fnshorthand(v),
        S3T::Pipe(v, _) => includes_fnshorthand(v),
        S3T::OrThat(v1, v2) => includes_fnshorthand(v1) || includes_fnshorthand(v2),
        S3T::PipeShorthand => true,
        _ => false,
    }
}

fn pipe(left: Box<S3T>, right: Box<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<VarIndex, String> {
    if includes_fnshorthand(right.clone()) {
        let i = VarIndex::Temp(*tempi);
        *tempi += 1;
        res.push(Instruction::GetPipeShorthand(i.clone()));
        let left = expr(*left, res, tempi, labeli, outi, outli)?;
        res.push(Instruction::SetPipeShorthand(left));
        let right = expr(*right, res, tempi, labeli, outi, outli)?;
        res.push(Instruction::SetPipeShorthand(i));
        Ok(right)
    } else {
        let left = expr(*left, res, tempi, labeli, outi, outli)?;
        let right = expr(*right.clone(), res, tempi, labeli, outi, outli)?;
        let i = *tempi;
        *tempi += 1;
        res.push(Instruction::Call(VarIndex::Temp(i), right, vec![left]));
        Ok(VarIndex::Temp(i))
    }
}

fn orthat(left: Box<S3T>, right: Box<S3T>, res: &mut Vec<Instruction>, tempi: &mut u64, labeli: &mut u64, outi: u64, outli: u64) -> Result<VarIndex, String> {
    let left = expr(*left, res, tempi, labeli, outi, outli)?;
    let right = expr(*right, res, tempi, labeli, outi, outli)?;
    let vi = *tempi;
    *tempi += 1;
    res.push(Instruction::OrThat(VarIndex::Temp(vi), left, right));
    Ok(VarIndex::Temp(vi))
}

fn operatorfn(operator: Operator, res: &mut Vec<Instruction>, tempi: &mut u64) -> Result<VarIndex, String> {
    let vi = *tempi;
    *tempi += 1;
    res.push(Instruction::OperatorFn(VarIndex::Temp(vi), operator));
    Ok(VarIndex::Temp(vi))
}

fn multilet(v: Vec<String>, res: &mut Vec<Instruction>) -> Result<(), String> {
    for i in v {
        res.push(Instruction::Decl(VarIndex::Ident(i)));
    }
    Ok(())
}

fn null(tempi: &mut u64) -> Result<VarIndex, String> {
    let r = *tempi;
    *tempi += 1;
    Ok(VarIndex::Temp(r))
}

fn pipe_shorthand(res: &mut Vec<Instruction>, tempi: &mut u64) -> Result<VarIndex, String> {
    let r = *tempi;
    *tempi += 1;
    res.push(Instruction::GetPipeShorthand(VarIndex::Temp(r)));
    Ok(VarIndex::Temp(r))
}