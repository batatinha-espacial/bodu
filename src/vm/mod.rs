use std::{any::Any, collections::HashMap, sync::{Arc, Mutex, OnceLock, Weak}};

pub mod op;

#[derive(Clone, Debug)]
pub enum Value {
    Number(i64),
    Float(f64),
    Null,
    String(String),
    Boolean(bool),
    Object(Object),
    Tuple(Vec<Container>),
    Bind(Container),
    Function(Function),
}

#[derive(Clone, Debug)]
pub struct Object {
    pub props: HashMap<String, ObjectProp>,
    pub internals: HashMap<u64, Container>,
    pub metaobj: Container,
    pub externals: HashMap<u64, SharedAny>,
}

#[derive(Clone, Debug)]
pub enum ObjectProp {
    Value(Container),
    GetSet(Container, Container),
}

pub type Gi = Arc<dyn Fn(u64) -> Option<Container>>;

#[derive(Clone, Debug)]
pub struct Function {
    pub internals: HashMap<u64, Container>,
    pub call: fn(StateContainer, Vec<Container>, Gi) -> Result<Container, Container>,
    pub state: StateContainer,
}

pub type Container = Arc<Mutex<Value>>;

pub fn make_container(v: Value) -> Container {
    let v = Container::new(Mutex::new(v));
    let mut cs = CONTAINERS.get().unwrap().lock().unwrap();
    cs.push(Arc::downgrade(&v));
    v
}

#[derive(Debug)]
pub struct State {
    pub scope: Container,
    pub parent: Option<StateContainer>,
    pub global: Option<StateContainer>,
}

pub type StateContainer = Arc<Mutex<State>>;

#[derive(Clone)]
pub enum Instruction {
    Add(VarIndex, VarIndex, VarIndex), // result, op1, op2
    Return(VarIndex), // op
    Throw(VarIndex), // op
    Call(VarIndex, VarIndex, Vec<VarIndex>), // result, f, ...args
    Get(VarIndex, VarIndex, VarIndex), // result, obj, prop
    Multiply(VarIndex, VarIndex, VarIndex), // result, op1, op2
    Negate(VarIndex, VarIndex), // result, op
    Subtract(VarIndex, VarIndex, VarIndex), // result, op1, op2
    Has(VarIndex, VarIndex, VarIndex), // result, obj, prop
    Set(VarIndex, VarIndex, VarIndex, VarIndex), // result, obj, prop, value
    Decl(VarIndex), // op
    Label(Label),
    Goto(Label),
    Eql(VarIndex, VarIndex, VarIndex), // result, op1, op2
    Neql(VarIndex, VarIndex, VarIndex), // result, op1, op2
    GotoIf(Label, VarIndex),
    Block(Vec<Instruction>),
    MakeTuple(VarIndex, Vec<VarIndex>), // result, ...ops
    DeTuple(Vec<VarIndex>, VarIndex), // ...results, op
    Divide(VarIndex, VarIndex, VarIndex), // result, op1, op2
    Remainder(VarIndex, VarIndex, VarIndex), // result, op1, op2
    MakeBind(VarIndex, VarIndex), // result, f
    Catch(VarIndex, VarIndex, Vec<Instruction>), // err?, err, block
}

#[derive(Clone)]
pub enum VarIndex {
    Arg(usize),
    Ident(String),
    Temp(u64),
}

#[derive(Clone)]
pub enum Label {
    Named(String),
    Unnamed(u64),
}

pub type SharedAny = Arc<Mutex<Box<dyn Any + Send>>>;

pub fn make_err(v: &str) -> Container {
    make_container(Value::String(v.to_string()))
}

static CONTAINERS: OnceLock<Mutex<Vec<Weak<Mutex<Value>>>>> = OnceLock::new();
static DEFERS: OnceLock<Mutex<Vec<Arc<dyn Fn() -> () + Sync + Send>>>> = OnceLock::new();

pub fn init() {
    let _ = CONTAINERS.set(Mutex::new(Vec::new()));
    let _ = DEFERS.set(Mutex::new(Vec::new()));
}

pub fn shutdown() {
    let mut dfs = DEFERS.get().unwrap().lock().unwrap();
    let dfs = dfs.as_mut_slice();
    for i in dfs {
        i();
    }
    let mut cs = CONTAINERS.get().unwrap().lock().unwrap();
    let cs = cs.as_mut_slice();
    for i in cs {
        let mut i = match i.upgrade() {
            Some(i) => i,
            None => continue,
        };
        match Arc::get_mut(&mut i).unwrap().get_mut().unwrap() {
            Value::Object(obj) => {
                obj.props.clear();
                obj.internals.clear();
                obj.metaobj = Arc::new(Mutex::new(Value::Null));
                obj.externals.clear();
            },
            Value::Tuple(t) => {
                t.clear();
            },
            Value::Bind(b) => {
                *b = Arc::new(Mutex::new(Value::Null));
            },
            Value::Function(f) => {
                f.internals.clear();
                f.state = Arc::new(Mutex::new(State { scope: Arc::new(Mutex::new(Value::Null)), parent: None, global: None }));
            },
            _ => {}
        }
    }
}

pub fn push_defer(df: Arc<dyn Fn() -> () + Sync + Send>) {
    let mut dfs = DEFERS.get().unwrap().lock().unwrap();
    dfs.push(df);
}