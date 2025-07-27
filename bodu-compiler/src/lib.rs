use bodu_vm::{Instruction, Label, Operator, VarIndex};

pub fn compile_instrs(instrs: Vec<Instruction>) -> Vec<u8> {
    let mut v = Vec::new();
    write_vec_instr(instrs, &mut v);
    v
}

fn write_vec_instr(instrs: Vec<Instruction>, vec_: &mut Vec<u8>) {
    for i in instrs {
        write_instr(i, vec_);
    }
    vec_.push(0x0);
}

fn write_instr(instr: Instruction, vec_: &mut Vec<u8>) {
    match instr {
        Instruction::Add(result, op1, op2) => {
            vec_.push(0x1);
            write_varindex(result, vec_);
            write_varindex(op1, vec_);
            write_varindex(op2, vec_);
        },
        Instruction::Return(op) => {
            vec_.push(0x2);
            write_varindex(op, vec_);
        },
        Instruction::Throw(op) => {
            vec_.push(0x3);
            write_varindex(op, vec_);
        },
        Instruction::Call(result, f, args) => {
            vec_.push(0x4);
            write_varindex(result, vec_);
            write_varindex(f, vec_);
            write_vec_varindex(args, vec_);
        },
        Instruction::Get(result, obj, prop) => {
            vec_.push(0x5);
            write_varindex(result, vec_);
            write_varindex(obj, vec_);
            write_varindex(prop, vec_);
        },
        Instruction::Multiply(result, op1, op2) => {
            vec_.push(0x6);
            write_varindex(result, vec_);
            write_varindex(op1, vec_);
            write_varindex(op2, vec_);
        },
        Instruction::Negate(result, op) => {
            vec_.push(0x7);
            write_varindex(result, vec_);
            write_varindex(op, vec_);
        },
        Instruction::Subtract(result, op1, op2) => {
            vec_.push(0x8);
            write_varindex(result, vec_);
            write_varindex(op1, vec_);
            write_varindex(op2, vec_);
        },
        Instruction::Has(result, obj, prop) => {
            vec_.push(0x9);
            write_varindex(result, vec_);
            write_varindex(obj, vec_);
            write_varindex(prop, vec_);
        },
        Instruction::Set(result, obj, prop, value) => {
            vec_.push(0xA);
            write_varindex(result, vec_);
            write_varindex(obj, vec_);
            write_varindex(prop, vec_);
            write_varindex(value, vec_);
        },
        Instruction::Decl(op) => {
            vec_.push(0xB);
            write_varindex(op, vec_);
        },
        Instruction::Label(l) => {
            vec_.push(0xC);
            write_label(l, vec_);
        },
        Instruction::Goto(l) => {
            vec_.push(0xD);
            write_label(l, vec_);
        },
        Instruction::Eql(result, op1, op2) => {
            vec_.push(0xE);
            write_varindex(result, vec_);
            write_varindex(op1, vec_);
            write_varindex(op2, vec_);
        },
        Instruction::Neql(result, op1, op2) => {
            vec_.push(0xF);
            write_varindex(result, vec_);
            write_varindex(op1, vec_);
            write_varindex(op2, vec_);
        },
        Instruction::GotoIf(l, op) => {
            vec_.push(0x10);
            write_label(l, vec_);
            write_varindex(op, vec_);
        },
        Instruction::Block(instrs) => {
            vec_.push(0x11);
            write_vec_instr(instrs, vec_);
        },
        Instruction::MakeTuple(result, ops) => {
            vec_.push(0x12);
            write_varindex(result, vec_);
            write_vec_varindex(ops, vec_);
        },
        Instruction::DeTuple(results, op) => {
            vec_.push(0x13);
            write_vec_varindex(results, vec_);
            write_varindex(op, vec_);
        },
        Instruction::Divide(result, op1, op2) => {
            vec_.push(0x14);
            write_varindex(result, vec_);
            write_varindex(op1, vec_);
            write_varindex(op2, vec_);
        },
        Instruction::Remainder(result, op1, op2) => {
            vec_.push(0x15);
            write_varindex(result, vec_);
            write_varindex(op1, vec_);
            write_varindex(op2, vec_);
        },
        Instruction::MakeBind(result, f) => {
            vec_.push(0x16);
            write_varindex(result, vec_);
            write_varindex(f, vec_);
        },
        Instruction::Catch(iserr, err, block) => {
            vec_.push(0x17);
            write_varindex(iserr, vec_);
            write_varindex(err, vec_);
            write_vec_instr(block, vec_);
        },
        Instruction::Assign(result, op) => {
            vec_.push(0x18);
            write_varindex(result, vec_);
            write_varindex(op, vec_);
        },
        Instruction::Defer(block) => {
            vec_.push(0x19);
            write_vec_instr(block, vec_);
        },
        Instruction::Boolean(result, op) => {
            vec_.push(0x1A);
            write_varindex(result, vec_);
            vec_.push(if op {
                1
            } else {
                0
            });
        },
        Instruction::Number(result, op) => {
            vec_.push(0x1B);
            write_varindex(result, vec_);
            let op = op.to_le_bytes();
            vec_.extend_from_slice(&op);
        },
        Instruction::Float(result, op) => {
            vec_.push(0x1C);
            write_varindex(result, vec_);
            let op = op.to_le_bytes();
            vec_.extend_from_slice(&op);
        },
        Instruction::String(result, op) => {
            vec_.push(0x1D);
            write_varindex(result, vec_);
            write_string(op, vec_);
        },
        Instruction::MakeFunction(result, body) => {
            vec_.push(0x1E);
            write_varindex(result, vec_);
            write_vec_instr(body, vec_);
        },
        Instruction::Not(result, op) => {
            vec_.push(0x1F);
            write_varindex(result, vec_);
            write_varindex(op, vec_);
        },
        Instruction::Gt(result, op1, op2) => {
            vec_.push(0x20);
            write_varindex(result, vec_);
            write_varindex(op1, vec_);
            write_varindex(op2, vec_);
        },
        Instruction::Ge(result, op1, op2) => {
            vec_.push(0x21);
            write_varindex(result, vec_);
            write_varindex(op1, vec_);
            write_varindex(op2, vec_);
        },
        Instruction::Lt(result, op1, op2) => {
            vec_.push(0x22);
            write_varindex(result, vec_);
            write_varindex(op1, vec_);
            write_varindex(op2, vec_);
        },
        Instruction::Le(result, op1, op2) => {
            vec_.push(0x23);
            write_varindex(result, vec_);
            write_varindex(op1, vec_);
            write_varindex(op2, vec_);
        },
        Instruction::And(result, op1, op2) => {
            vec_.push(0x24);
            write_varindex(result, vec_);
            write_varindex(op1, vec_);
            write_varindex(op2, vec_);
        },
        Instruction::Or(result, op1, op2) => {
            vec_.push(0x25);
            write_varindex(result, vec_);
            write_varindex(op1, vec_);
            write_varindex(op2, vec_);
        },
        Instruction::Xor(result, op1, op2) => {
            vec_.push(0x26);
            write_varindex(result, vec_);
            write_varindex(op1, vec_);
            write_varindex(op2, vec_);
        },
        Instruction::GetPipeShorthand(result) => {
            vec_.push(0x27);
            write_varindex(result, vec_);
        },
        Instruction::SetPipeShorthand(op) => {
            vec_.push(0x28);
            write_varindex(op, vec_);
        },
        Instruction::OrThat(result, op1, op2) => {
            vec_.push(0x29);
            write_varindex(result, vec_);
            write_varindex(op1, vec_);
            write_varindex(op2, vec_);
        },
        Instruction::OperatorFn(result, opfn) => {
            vec_.push(0x2A);
            write_varindex(result, vec_);
            write_operator(opfn, vec_);
        },
        Instruction::Debug(result) => {
            vec_.push(0x2B);
            write_varindex(result, vec_);
        },
        Instruction::Release(result) => {
            vec_.push(0x2C);
            write_varindex(result, vec_);
        },
        Instruction::Maybe(result) => {
            vec_.push(0x2D);
            write_varindex(result, vec_);
        },
        Instruction::ToNumber(result, op) => {
            vec_.push(0x2E);
            write_varindex(result, vec_);
            write_varindex(op, vec_);
        },
        Instruction::Iterate(r1, r2, it) => {
            vec_.push(0x2F);
            write_varindex(r1, vec_);
            write_varindex(r2, vec_);
            write_varindex(it, vec_);
        },
        Instruction::Probably(result) => {
            vec_.push(0x30);
            write_varindex(result, vec_);
        },
        Instruction::Possibly(result) => {
            vec_.push(0x31);
            write_varindex(result, vec_);
        },
        Instruction::IsntNull(result, op) => {
            vec_.push(0x32);
            write_varindex(result, vec_);
            write_varindex(op, vec_);
        },
    }
}

fn write_vec_varindex(v: Vec<VarIndex>, vec_: &mut Vec<u8>) {
    write_u64(v.len() as u64, vec_);
    for i in v {
        write_varindex(i, vec_);
    }
}

fn write_u64(i: u64, vec_: &mut Vec<u8>) {
    let i = i.to_le_bytes();
    vec_.extend_from_slice(&i);
}

fn write_string(s: String, vec_: &mut Vec<u8>) {
    let s = s.as_bytes();
    write_u64(s.len() as u64, vec_);
    vec_.extend_from_slice(s);
}

fn write_varindex(v: VarIndex, vec_: &mut Vec<u8>) {
    match v {
        VarIndex::Arg(v) => {
            vec_.push(0x0);
            write_u64(v as u64, vec_);
        },
        VarIndex::Ident(s) => {
            vec_.push(0x1);
            write_string(s, vec_);
        },
        VarIndex::Temp(v) => {
            vec_.push(0x2);
            write_u64(v, vec_);
        },
    }
}

fn write_label(l: Label, vec_: &mut Vec<u8>) {
    match l {
        Label::Named(s) => {
            vec_.push(0x0);
            write_string(s, vec_);
        },
        Label::Unnamed(i) => {
            vec_.push(0x1);
            write_u64(i, vec_);
        },
    }
}

fn write_operator(op: Operator, vec_: &mut Vec<u8>) {
    vec_.push(match op {
        Operator::Plus => 0x0,
        Operator::Minus => 0x1,
        Operator::Times => 0x2,
        Operator::Divide => 0x3,
        Operator::Modulus => 0x4,
        Operator::OrThat => 0x5,
        Operator::Ternary => 0x6,
        Operator::EqualTo => 0x7,
        Operator::Not => 0x8,
        Operator::NotEqualTo => 0x9,
        Operator::Less => 0xA,
        Operator::LessOrEqual => 0xB,
        Operator::Greater => 0xC,
        Operator::GreaterOrEqual => 0xD,
        Operator::And => 0xE,
        Operator::Or => 0xF,
        Operator::Xor => 0x10,
        Operator::Property => 0x11,
        Operator::Tuple => 0x12,
        Operator::Pipe => 0x13,
        Operator::IsntNull => 0x14,
    });
}