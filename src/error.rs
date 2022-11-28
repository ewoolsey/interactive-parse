use inquire::InquireError;
use thiserror::Error;

pub type SchemaResult<T> = core::result::Result<T, SchemaError>;

#[derive(Error, Debug)]
pub enum SchemaError {
    #[error("{0}")]
    Error(String),

    #[error("{0}")]
    Generic(String),

    #[error("invalid admin address")]
    AdminAddress,

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Inquire(#[from] InquireError),

    #[error("{0}")]
    Serde(#[from] serde_json::Error),

    #[error("invalid mnemonic")]
    Mnemonic,

    #[error("invalid derivation path")]
    DerivationPath,

    #[error("Unsupported shell, must use bash or zsh")]
    UnsupportedShell,

    #[error("Unimplemented")]
    Unimplemented,

    #[error("Chain already exists")]
    ChainAlreadyExists,

    #[error("Contract already exists")]
    ContractAlreadyExists,

    #[error("Contract not found")]
    ContractNotFound,

    #[error("Env already exists")]
    EnvAlreadyExists,

    #[error("Invalid directory")]
    InvalidDir,

    #[error("Contract does not have an address")]
    NoAddr,

    #[error("Error parsing chain")]
    ChainId { chain_id: String },

    #[error("Error parsing denom")]
    Denom { name: String },

    #[error("Empty response")]
    EmptyResponse,

    #[error("Key already exists")]
    KeyAlreadyExists,

    #[error("Key not found")]
    KeyNotFound { key_name: String },

    #[error("Code id not found")]
    CodeIdNotFound,

    #[error("Env not found")]
    EnvNotFound,

    #[error("Contract address not found")]
    AddrNotFound,

    #[error(
        "{} Config file not found, perhaps you need to run \"deploy init\"?",
        "Deploy Error"
    )]
    ConfigNotFound {},

    #[error("Invalid schema")]
    InvalidSchema {},
}
