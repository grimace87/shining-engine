
#[derive(Debug)]
pub enum EngineError {
    OpFailed(String),
    MissingResource(String),
    Compatibility(String),
    EngineError(String),
    UserError(String)
}
