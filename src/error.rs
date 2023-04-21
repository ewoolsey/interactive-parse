use inquire::InquireError;
use serde_json::Value;
use thiserror::Error;

pub type SchemaResult<T> = core::result::Result<T, SchemaError>;

#[derive(Error, Debug)]
pub enum SchemaError {
    #[error(transparent)]
    Inquire(#[from] InquireError),

    #[error("{0}")]
    Generic(String),

    #[error("Undo depth: {depth}")]
    Undo { depth: u16 },

    #[error(
        "interactive-parse generated this json object: {}\n{}",
        serde_json::to_string_pretty(&value).unwrap(),
        serde_error
    )]
    Serde {
        value: Value,
        serde_error: serde_json::Error,
    },

    #[error(
        "Schemas of this type cannot yet be parsed interactively
        Please open an issue at \"https://github.com/ewoolsey/interactive-parse\""
    )]
    SchemaIsBool,

    #[error(
        "Parsing schemas of this type are not yet supported.
        Please open an issue at \"https://github.com/ewoolsey/interactive-parse\""
    )]
    Unimplemented,
}
