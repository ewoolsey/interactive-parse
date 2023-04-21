use std::cell::Cell;

use schemars::{schema_for, JsonSchema};
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::{error::SchemaResult, parse_schema};

pub trait InteractiveParseVal
where
    Self: Sized,
{
    fn parse_to_val() -> SchemaResult<Value>;
}

impl<T> InteractiveParseVal for T
where
    T: JsonSchema,
{
    fn parse_to_val() -> SchemaResult<Value> {
        let root_schema = schema_for!(T);
        let name = String::default();
        let mut title = None;
        if let Some(metadata) = &root_schema.schema.metadata {
            if let Some(title_ref) = &metadata.title {
                title = Some(title_ref.clone());
            }
        }

        let value = parse_schema(
            &root_schema.definitions,
            title,
            name,
            root_schema.schema,
            &Cell::new(0),
        )?;

        Ok(value)
    }
}

pub trait InteractiveParseObj
where
    Self: Sized,
{
    fn parse_to_obj() -> SchemaResult<Self>;
}

impl<T> InteractiveParseObj for T
where
    T: JsonSchema + DeserializeOwned,
{
    fn parse_to_obj() -> SchemaResult<Self> {
        let value = Self::parse_to_val()?;
        let my_struct = serde_json::from_value::<T>(value.clone()).map_err(|e| {
            crate::error::SchemaError::Serde {
                value,
                serde_error: e,
            }
        })?;
        Ok(my_struct)
    }
}
