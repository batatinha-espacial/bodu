use crate::vm::{op::make_object, Container, Gi, StateContainer};

pub async fn new(_: StateContainer, _: Vec<Container>, _: Gi) -> Result<Container, Container> {
    Ok(make_object())
}