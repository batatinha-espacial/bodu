use std::{any::Any, collections::HashMap, sync::{Arc, Mutex, OnceLock, Weak}};

// standard operations that bodu code can do
pub mod op;

// bodu values. those are often wrapped in Arc<Mutex<T>>s.
#[derive(Clone, Debug)]
pub enum Value {
    Number(i64), // integers
    Float(f64), // floats
    Null, // null
    String(String), // strings
    Boolean(bool), // booleans
    Object(Object), // objects
    Tuple(Vec<Container>), // tuples
    Bind(Container), // binds are like getter/setter properties, but they are variables and only have a getter.
    Function(Function), // functions
}

// objects
#[derive(Clone, Debug)]
pub struct Object {
    pub props: HashMap<String, ObjectProp>, // properties
    pub internals: HashMap<u64, Container>, // internal values used by the object
    pub metaobj: Container, // meta objects are objects that define how the object behaves (like for operator overloading)
    pub externals: HashMap<u64, SharedAny>, // internal values that aren't bodu language values
}

// object properties
#[derive(Clone, Debug)]
pub enum ObjectProp {
    Value(Container), // value properties
    GetSet(Container, Container), // getter/setter properties
}

// gets the function's internal values
pub type Gi = Arc<dyn Fn(u64) -> Option<Container>>;

// functions
#[derive(Clone, Debug)]
pub struct Function {
    pub internals: HashMap<u64, Container>, // internal values used by the function
    pub call: fn(StateContainer, Vec<Container>, Gi) -> Result<Container, Container>, // the actual function
    pub state: StateContainer, // the state it runs on
}

// Arc<Mutex<Value>>, used almost everywhere in the codebase
pub type Container = Arc<Mutex<Value>>;

// creates a new container
pub fn make_container(v: Value) -> Container {
    let v = Container::new(Mutex::new(v));
    // START don't touch this
    let mut cs = CONTAINERS.get().unwrap().lock().unwrap();
    cs.push(Arc::downgrade(&v));
    // END don't touch this
    v
}

// the state: it contains the scope of a function and it is an execution context (kinda)
#[derive(Debug)]
pub struct State {
    pub scope: Container, // the scope
    pub parent: Option<StateContainer>, // parent state
    pub global: Option<StateContainer>, // global state
}

// Container but for States
pub type StateContainer = Arc<Mutex<State>>;

// instructions for the VM
// TODO: explain what these are
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

// type for object externals
pub type SharedAny = Arc<Mutex<Box<dyn Any + Send>>>;

// makes an error, could be a macro
pub fn make_err(v: &str) -> Container {
    make_container(Value::String(v.to_string()))
}

// DON'T TOUCH ANYTHING BELOW THIS LINE

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