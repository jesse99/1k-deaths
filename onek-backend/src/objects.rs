use super::{Color, Id, Object, Value};
use std::collections::HashMap;

pub fn load_objects() -> HashMap<Id, Object> {
    let mut result = HashMap::new();
    add_objects(include_str!("../data/terrain.ron"), &mut result);
    result
}

fn add_objects(text: &str, objects: &mut HashMap<Id, Object>) {
    let value: ron::Value = ron::from_str(text).unwrap();
    let seq: Vec<ron::Value> = value.into_rust().unwrap();

    for object in seq {
        let map: ron::Map = object.into_rust().unwrap();
        let object = onek_object(map);
        let id = object.get("id").unwrap().to_id().clone();
        let old = objects.insert(id.to_owned(), object);
        assert!(old.is_none(), "id '{id:?}' already exists'");
    }
}

fn onek_object(map: ron::Map) -> Object {
    let mut object = Object::default();

    for (key, value) in map {
        let key = into_str(key);
        let value = into_obj(value);
        if key == "id" {
            object.insert(key, Value::Id(Id(value.to_str().to_owned())));
        } else if key == "color" || key == "back_color" {
            object.insert(key, Value::Color(Color::new(value.to_str())));
        } else {
            object.insert(key, value);
        }
    }

    object
}

fn into_obj(value: ron::Value) -> Value {
    match value {
        ron::Value::Bool(v) => Value::Bool(v),
        ron::Value::Char(v) => Value::Char(v),
        ron::Value::Map(_) => panic!("Map is not supported"),
        ron::Value::Number(v) => match v {
            ron::Number::Integer(w) => Value::Int(i32::try_from(w).unwrap()),
            ron::Number::Float(_) => panic!("Float is not supported"),
        },
        ron::Value::Option(_) => panic!("Option is not supported"),
        ron::Value::String(v) => Value::String(v),
        ron::Value::Seq(v) => Value::Seq(v.into_iter().map(|w| into_obj(w)).collect()),
        ron::Value::Unit => panic!("unit is not supported"),
    }
}

fn into_str(object: ron::Value) -> String {
    match object {
        ron::Value::String(s) => s,
        _ => panic!("object {object:?} is not a String"),
    }
}
