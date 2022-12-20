use std::collections::BTreeMap;

use error::{SchemaError, SchemaResult};
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
    title: Option<String>,
    name: String,
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
            title,
            name,
            description,
        ),
        Some(SingleOrVec::Vec(vec)) => {
            // This usually represents an optional regular type
            // Probably not a great assumption.
            let instance_type =
                Box::new(vec.into_iter().find(|x| x != &InstanceType::Null).unwrap());
            if Confirm::new("Add optional value?")
                .with_help_message(format!("{}{}", get_title_str(&title), name).as_str())
                .prompt()?
            {
                get_single_instance(
                    definitions,
                    schema.array,
                    schema.object,
                    schema.subschemas,
                    instance_type,
                    title,
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
                let schema = get_schema_object_ref(schema)?;
                parse_schema(
                    definitions,
                    Some(reference.to_string()),
                    name,
                    schema.clone(),
                )
            }
            // Or it could be a subschema
            else {
                get_subschema(definitions, title, name, schema.subschemas, description)
            }
        }
    }
}

fn update_title(mut title: Option<String>, schema: &SchemaObject) -> Option<String> {
    if let Some(metadata) = &schema.metadata {
        title = metadata.title.clone();
    }
    title
}

fn get_title_str(title: &Option<String>) -> String {
    let mut title_str = String::new();
    if let Some(title) = title {
        title_str.push_str(format!("<{}> ", title).as_str());
    }
    title_str
}

fn get_description(schema: &SchemaObject) -> String {
    match &schema.metadata {
        Some(metadata) => match &metadata.description {
            Some(description_ref) => {
                let mut description = description_ref.clone();
                if description.len() > 60 {
                    description.truncate(60);
                    description.push_str("...");
                }
                format!(": {}", description)
            }
            None => String::default(),
        },
        None => String::default(),
    }
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::boxed_local)]
fn get_single_instance(
    definitions: &BTreeMap<String, Schema>,
    array_info: Option<Box<ArrayValidation>>,
    object_info: Option<Box<ObjectValidation>>,
    subschema: Option<Box<SubschemaValidation>>,
    instance: Box<InstanceType>,
    title: Option<String>,
    name: String,
    description: String,
) -> SchemaResult<Value> {
    match *instance {
        InstanceType::String => get_string(name, description),
        InstanceType::Number => get_num(name, description),
        InstanceType::Integer => get_int(name, description),
        InstanceType::Boolean => get_bool(name, description),
        InstanceType::Array => get_array(definitions, array_info, title, name, description),
        InstanceType::Object => get_object(definitions, object_info, title, name, description),
        InstanceType::Null => {
            // This represents an optional enum
            // Likely the subschema will have info here.
            get_subschema(definitions, title, name, subschema, description)
        }
    }
}

fn get_subschema(
    definitions: &BTreeMap<String, Schema>,
    title: Option<String>,
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
            let name = match schema_object.clone().object {
                Some(mut object) => object.properties.pop_first().unwrap().0,
                None => "None".into(),
            };
            options.push(name);
        }
        let option = Select::new("Select one:", options.clone())
            .with_help_message(
                format!("{}{}{}", get_title_str(&title), name, description.as_str()).as_str(),
            )
            .prompt()?;
        let position = options.iter().position(|x| x == &option).unwrap();
        let object = get_schema_object(schema_vec[position].clone())?;
        let title = update_title(title, &object);
        Ok(parse_schema(definitions, title, name, object)?)
    }
    // Next check the all_of field.
    else if let Some(schema_vec) = subschema.all_of {
        let mut values = Vec::new();
        for schema in schema_vec {
            let object = get_schema_object(schema)?;
            let title = update_title(title.clone(), &object);
            values.push(parse_schema(
                definitions,
                title.clone(),
                name.clone(),
                object,
            )?)
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
        let Schema::Object(object) = non_null else {
                            panic!("invalid schema");
                        };
        let title = update_title(title, &object);

        if Confirm::new("Add optional value?")
            .with_help_message(format!("{}{}", get_title_str(&title), name).as_str())
            .prompt()?
        {
            parse_schema(definitions, title, name, object)
        } else {
            Ok(Value::Null)
        }
    } else {
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
    title: Option<String>,
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
        let object = get_schema_object(schema)?;
        if let Some(end) = range.end {
            if array.len() == end as usize {
                break;
            }
        }
        let start = range.start.unwrap_or_default();
        if array.len() >= start as usize
            && !Confirm::new("Add element?")
                .with_help_message(
                    format!("{}{}{}", get_title_str(&title), name, description).as_str(),
                )
                .prompt()?
        {
            break;
        }

        array.push(parse_schema(
            definitions,
            title.clone(),
            format!("{}.{}", name.clone(), i),
            object.clone(),
        )?);
    }
    Ok(Value::Array(array))
}

fn get_object(
    definitions: &BTreeMap<String, Schema>,
    object_info: Option<Box<ObjectValidation>>,
    title: Option<String>,
    _name: String,
    _description: String,
) -> SchemaResult<Value> {
    let map = object_info
        .unwrap()
        .properties
        .into_iter()
        .map(|(name, schema)| {
            let schema_object = get_schema_object(schema)?;
            let object = parse_schema(definitions, title.clone(), name.to_string(), schema_object)?;
            Ok((name, object))
        })
        .collect::<SchemaResult<Map<String, Value>>>()?;
    Ok(Value::Object(map))
}

fn get_schema_object(schema: Schema) -> SchemaResult<SchemaObject> {
    match schema {
        Schema::Bool(_) => Err(SchemaError::SchemaIsBool),
        Schema::Object(object) => Ok(object),
    }
}

fn get_schema_object_ref(schema: &Schema) -> SchemaResult<&SchemaObject> {
    match schema {
        Schema::Bool(_) => Err(SchemaError::SchemaIsBool),
        Schema::Object(object) => Ok(object),
    }
}

#[cfg(test)]
mod tests {
    use schemars::JsonSchema;
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
        /// This is a vec of tuples.
        pub vec_map: MyVecMap,
    }

    /// Doc comment on struct
    #[derive(JsonSchema, Serialize, Deserialize, Debug)]
    pub struct MyStruct2 {
        /// Doc comment on field
        pub option_int: Option<i32>,
    }

    /// Doc comment on struct
    #[derive(JsonSchema, Serialize, Deserialize, Debug)]
    pub struct MyStruct3 {
        /// Doc comment on field
        pub option_int: Option<f64>,
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

    /// Doc comment on enum
    #[derive(JsonSchema, Serialize, Deserialize, Debug)]
    pub struct MyVecMap(Vec<(String, u32)>);

    #[test]
    fn test() {
        let my_struct = MyStruct::parse_to_obj().unwrap();
        dbg!(my_struct);
    }

    #[test]
    fn test_enum() {
        let my_struct = MyEnum::parse_to_obj().unwrap();
        dbg!(my_struct);
    }
}
