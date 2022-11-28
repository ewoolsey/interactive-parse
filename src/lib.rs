use std::collections::BTreeMap;

use error::SchemaResult;
use inquire::{Confirm, CustomType, Select, Text};
use schemars::schema::{
    ArrayValidation, InstanceType, ObjectValidation, Schema, SchemaObject, SingleOrVec,
    SubschemaValidation,
};
use serde_json::{json, Map, Value};
pub mod error;
pub mod traits;

pub fn parse_schema(
    definitions: &BTreeMap<String, Schema>,
    name: String,
    val_type: Option<String>,
    schema: SchemaObject,
) -> SchemaResult<Value> {
    let description = get_description(&schema);
    match schema.instance_type.clone() {
        Some(SingleOrVec::Single(instance_type)) => get_single_instance(
            definitions,
            schema.array,
            schema.object,
            schema.subschemas,
            instance_type,
            name,
            description,
        ),
        Some(SingleOrVec::Vec(vec)) => {
            // This usually represents an optional regular type
            // Probably not a great assumption.
            let instance_type =
                Box::new(vec.into_iter().find(|x| x != &InstanceType::Null).unwrap());
            if Confirm::new("Add optional value?")
                .with_help_message(name.as_str())
                .prompt()?
            {
                get_single_instance(
                    definitions,
                    schema.array,
                    schema.object,
                    schema.subschemas,
                    instance_type,
                    name,
                    description,
                )
            } else {
                Ok(Value::Null)
            }
        }
        None => {
            // This represents a referenced type
            if let Some(reference) = schema.reference {
                let reference = reference.strip_prefix("#/definitions/").unwrap();
                let schema = definitions.get(reference).unwrap();
                let Schema::Object(schema) = schema else {
                    panic!("invalid schema");
                };
                parse_schema(
                    definitions,
                    name,
                    Some(reference.to_string()),
                    schema.clone(),
                )
            }
            // Or it could be a subschema
            else {
                get_subschema(definitions, name, schema.subschemas, description)
            }
        }
    }
}

fn get_description(schema: &SchemaObject) -> String {
    match &schema.metadata {
        Some(metadata) => match &metadata.description {
            Some(description) => format!(": {}", description),
            None => String::default(),
        },
        None => String::default(),
    }
}

fn get_single_instance(
    definitions: &BTreeMap<String, Schema>,
    array_info: Option<Box<ArrayValidation>>,
    object_info: Option<Box<ObjectValidation>>,
    subschema: Option<Box<SubschemaValidation>>,
    instance: Box<InstanceType>,
    name: String,
    description: String,
) -> SchemaResult<Value> {
    match *instance {
        InstanceType::String => get_string(name, description),
        InstanceType::Number => get_num(name, description),
        InstanceType::Integer => get_int(name, description),
        InstanceType::Boolean => get_bool(name, description),
        InstanceType::Array => get_array(definitions, array_info, name, description),
        InstanceType::Object => get_object(definitions, object_info, name, description),
        InstanceType::Null => {
            // This represents an optional enum
            // Likely the subschema will have info here.
            get_subschema(definitions, name, subschema, description)
        }
    }
}

fn get_subschema(
    definitions: &BTreeMap<String, Schema>,
    name: String,
    subschema: Option<Box<SubschemaValidation>>,
    description: String,
) -> SchemaResult<Value> {
    let subschema = subschema.unwrap();
    // First we check the one_of field.
    if let Some(schema_vec) = subschema.one_of {
        let mut options = Vec::new();
        for schema in &schema_vec {
            let Schema::Object(schema_object) = schema else {
                                panic!("invalid schema");
                            };
            let name = schema_object
                .clone()
                .object
                .unwrap()
                .properties
                .pop_first()
                .unwrap()
                .0;
            options.push(name);
        }
        let option = Select::new(name.as_str(), options.clone())
            .with_help_message(format!("{}{}", name, description.as_str()).as_str())
            .prompt()?;
        let position = options.iter().position(|x| x == &option).unwrap();
        let Schema::Object(object) = schema_vec[position].clone() else {
                            panic!("invalid schema");
                        };
        Ok(parse_schema(definitions, name, None, object)?)
    }
    // Next check the all_of field.
    else if let Some(schema_vec) = subschema.all_of {
        let mut values = Vec::new();
        for schema in schema_vec {
            let Schema::Object(object) = schema else {
                            panic!("invalid schema");
                        };
            values.push(parse_schema(definitions, name.clone(), None, object)?)
        }
        match values.len() {
            1 => Ok(values.pop().unwrap()),
            _ => Ok(Value::Array(values)),
        }
    }
    // Next check the any_of field.
    // This seems to be a weird way to get options
    else if let Some(schema_vec) = subschema.any_of {
        let non_null = schema_vec
            .into_iter()
            .find(|x| {
                let Schema::Object(object) = x else {
                            panic!("invalid schema");
                        };
                object.instance_type != Some(SingleOrVec::Single(Box::new(InstanceType::Null)))
            })
            .unwrap();

        if Confirm::new("Add optional value?")
            .with_help_message(name.as_str())
            .prompt()?
        {
            let Schema::Object(object) = non_null else {
                            panic!("invalid schema");
                        };
            parse_schema(definitions, name, None, object)
        } else {
            Ok(Value::Null)
        }
    } else {
        dbg!(subschema);
        panic!("invalid schema");
    }
}

fn get_int(name: String, description: String) -> SchemaResult<Value> {
    Ok(json!(CustomType::<i64>::new(name.as_str())
        .with_help_message(format!("int{}", description).as_str())
        .prompt()?))
}

fn get_string(name: String, description: String) -> SchemaResult<Value> {
    Ok(Value::String(
        Text::new(name.as_str())
            .with_help_message(format!("string{}", description).as_str())
            .prompt()?,
    ))
}

fn get_num(name: String, description: String) -> SchemaResult<Value> {
    Ok(json!(CustomType::<f64>::new(name.as_str())
        .with_help_message(format!("num{}", description).as_str())
        .prompt()?))
}

fn get_bool(name: String, description: String) -> SchemaResult<Value> {
    Ok(json!(CustomType::<bool>::new(name.as_str())
        .with_help_message(format!("bool{}", description).as_str())
        .prompt()?))
}

fn get_array(
    definitions: &BTreeMap<String, Schema>,
    array_info: Option<Box<ArrayValidation>>,
    name: String,
    description: String,
) -> SchemaResult<Value> {
    let array_info = array_info.unwrap();
    let mut array = Vec::new();
    let range = array_info.min_items..array_info.max_items;
    let schemas = match array_info.items.unwrap() {
        SingleOrVec::Single(single) => {
            vec![*single]
        }
        SingleOrVec::Vec(vec) => vec,
    };
    for (i, schema) in schemas.into_iter().enumerate() {
        let Schema::Object(object) = schema else {
                                panic!("invalid schema");
        };
        if let Some(end) = range.end {
            if array.len() == end as usize {
                break;
            }
        }
        let start = range.start.unwrap_or_default();
        if array.len() >= start as usize
            && !Confirm::new("Add item?")
                .with_help_message(format!("{}{}", name, description).as_str())
                .prompt()?
        {
            break;
        }

        array.push(parse_schema(
            definitions,
            format!("{}.{}", name.clone(), i),
            None,
            object.clone(),
        )?);
    }
    Ok(Value::Array(array))
}

fn get_object(
    definitions: &BTreeMap<String, Schema>,
    object_info: Option<Box<ObjectValidation>>,
    _name: String,
    _description: String,
) -> SchemaResult<Value> {
    let map = object_info
        .unwrap()
        .properties
        .into_iter()
        .map(|(name, schema)| {
            let Schema::Object(schema_object) = schema else {
                            panic!("invalid schema");
                        };

            let object = parse_schema(definitions, name.to_string(), None, schema_object)?;
            Ok((name, object))
        })
        .collect::<SchemaResult<Map<String, Value>>>()?;
    Ok(Value::Object(map))
}

#[cfg(test)]
mod tests {
    use schemars::{schema_for, JsonSchema};
    use serde::{Deserialize, Serialize};

    use crate::traits::InteractiveParseObj;

    /// This is the struct used for testing.
    #[derive(JsonSchema, Serialize, Deserialize, Debug)]
    pub struct MyStruct {
        /// This is an integer.
        pub my_int: Option<i32>,
        /// This is a boolean.
        pub my_bool: bool,
        /// This is an optional tuple of ints.
        pub my_tuple: Option<(i32, Option<i32>)>,
        /// This is a vec of ints.
        pub my_vec: Vec<i32>,
        /// This is an enumerated type.
        pub my_enum: Option<MyEnum>,
        /// This is an object.
        pub str_2: Option<MyStruct2>,
    }

    /// Doc comment on struct
    #[derive(JsonSchema, Serialize, Deserialize, Debug)]
    pub struct MyStruct2 {
        /// Doc comment on field
        pub option_int: Option<i32>,
    }

    /// Doc comment on enum
    #[derive(JsonSchema, Serialize, Deserialize, Debug)]
    pub enum MyEnum {
        /// This is a tuple variant.
        StringNewType(Option<String>),
        /// This is a struct variant.
        StructVariant {
            /// This is a vec of floats.
            floats: Vec<f32>,
        },
    }

    #[test]
    fn test() {
        let root_schema = schema_for!(MyStruct);
        println!("{:#?}", &root_schema);
        //println!("{:#?}", &root_schema.definitions);
        let my_struct = MyStruct::interactive_parse().unwrap();
        dbg!(my_struct);

        //println!("{:?}", schema.definitions)
        //println!("{:?}", schema.meta_schema)
    }
}
