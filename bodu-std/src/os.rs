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

fn is_unix_base() -> bool {
    cfg!(unix)
}

pub async fn is_unix(_: StateContainer, _: Vec<Container>, _: Gi) -> Result<Container, Container> {
    Ok(make_container(Value::Boolean(is_unix_base())))
}

fn arch_base() -> &'static str {
    if cfg!(target_arch = "x86") {
        "x86"
    } else if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "arm") {
        "arm"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "unknown"
    }
}

pub async fn arch(_: StateContainer, _: Vec<Container>, _: Gi) -> Result<Container, Container> {
    Ok(make_container(Value::String(arch_base().to_string())))
}