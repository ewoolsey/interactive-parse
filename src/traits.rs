use schemars::{schema_for, JsonSchema};
use serde::de::DeserializeOwned;

use crate::{error::SchemaResult, parse_schema};

pub trait InteractiveParseObj
where
    Self: Sized,
{
    fn interactive_parse() -> SchemaResult<Self>;
}

impl<T> InteractiveParseObj for T
where
    T: JsonSchema + DeserializeOwned,
{
    fn interactive_parse() -> SchemaResult<Self> {
        let root_schema = schema_for!(T);
        let mut name = String::default();
        if let Some(metadata) = &root_schema.schema.metadata {
            if let Some(title) = &metadata.title {
                name = title.clone();
            }
        }
        let value = parse_schema(&root_schema.definitions, name, None, root_schema.schema)?;
        let my_struct = serde_json::from_value::<T>(value)?;
        Ok(my_struct)
    }
}
