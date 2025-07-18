use crate::vm::{make_container, Container, Gi, StateContainer, Value};

fn name_base() -> String {
    if cfg!(target_os = "windows") {
        "windows".to_string()
    } else if cfg!(target_os = "macos") {
        "macos".to_string()
    } else if cfg!(target_os = "linux") {
        "linux".to_string()
    } else if cfg!(target_os = "freebsd") {
        "freebsd".to_string()
    } else {
        "unknown".to_string()
    }
}

pub async fn name(_: StateContainer, _: Vec<Container>, _: Gi) -> Result<Container, Container> {
    Ok(make_container(Value::String(name_base())))
}