use serde_json::{Map, Value};

/// Uni-directional matches for [`Value`]s
///
/// This will return true if all of the properties or items of `self` are in `other`, but does not
/// check if the inverse is true.
pub trait JsonEq<Rhs = Self> {
    fn json_eq(&self, other: &Rhs) -> bool;
}

impl JsonEq for Value {
    fn json_eq(&self, other: &Self) -> bool {
        match self {
            Value::Array(values) => match other {
                Value::Array(other_values) => values.json_eq(other_values),
                _ => false,
            },
            Value::Object(values) => match other {
                Value::Object(other_values) => values.json_eq(other_values),
                _ => false,
            },
            value => value == other,
        }
    }
}

impl JsonEq for Vec<Value> {
    fn json_eq(&self, other: &Self) -> bool {
        'outer: for value in self.iter() {
            for other_value in other.iter() {
                if value.json_eq(other_value) {
                    println!("found {value:?} in {other:?}");
                    continue 'outer;
                }
            }

            println!("didnt find {value:?} in {other:?}");
            return false;
        }

        true
    }
}

impl JsonEq for Map<String, Value> {
    fn json_eq(&self, other: &Self) -> bool {
        for (key, value) in self.iter() {
            if !other
                .get(key)
                .map(|other_value| value.json_eq(other_value))
                .unwrap_or(false)
            {
                println!("didn't find {key} in {other:?}");
                return false;
            }
        }

        true
    }
}

impl JsonEq for String {
    fn json_eq(&self, other: &Self) -> bool {
        self == other
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use speculoos::prelude::*;

    use serde_json::json;

    #[rstest]
    // Null
    #[case(Value::Null, Value::Null, true)]
    #[case(Value::Null, Value::Bool(false), false)]
    #[case(Value::Null, Value::Number(1.into()), false)]
    #[case(Value::Null, Value::String("text".to_string()), false)]
    #[case(Value::Null, Value::Array(Vec::new()), false)]
    #[case(Value::Null, Value::Object(Map::new()), false)]
    // Bool
    #[case(Value::Bool(true), Value::Null, false)]
    #[case(Value::Bool(true), Value::Bool(true), true)]
    #[case(Value::Bool(true), Value::Bool(false), false)]
    #[case(Value::Bool(true), Value::Number(1.into()), false)]
    #[case(Value::Bool(true), Value::String("text".to_string()), false)]
    #[case(Value::Bool(true), Value::Array(Vec::new()), false)]
    #[case(Value::Bool(true), Value::Object(Map::new()), false)]
    // Number
    #[case(Value::Number(1.into()), Value::Null, false)]
    #[case(Value::Number(1.into()), Value::Bool(true), false)]
    #[case(Value::Number(1.into()), Value::Number(1.into()), true)]
    #[case(Value::Number(1.into()), Value::Number(2.into()), false)]
    #[case(Value::Number(1.into()), Value::String("text".to_string()), false)]
    #[case(Value::Number(1.into()), Value::Array(Vec::new()), false)]
    #[case(Value::Number(1.into()), Value::Object(Map::new()), false)]
    // String
    #[case(Value::String("text".to_string()), Value::Null, false)]
    #[case(Value::String("text".to_string()), Value::Bool(false), false)]
    #[case(Value::String("text".to_string()), Value::Number(1.into()), false)]
    #[case(Value::String("text".to_string()), Value::String("text".to_string()), true)]
    #[case(Value::String("text".to_string()), Value::String("other text".to_string()), false)]
    #[case(Value::String("text".to_string()), Value::Array(Vec::new()), false)]
    #[case(Value::String("text".to_string()), Value::Object(Map::new()), false)]
    // Array
    #[case(json!([1, 2, 3, "text"]), Value::Null, false)]
    #[case(json!([1, 2, 3, "text"]), Value::Bool(false), false)]
    #[case(json!([1, 2, 3, "text"]), Value::Number(1.into()), false)]
    #[case(json!([1, 2, 3, "text"]), Value::String("text".to_string()), false)]
    #[case(json!([1, 2, 3, "text"]), json!([1, 2, 3, "text"]), true)]
    #[case(json!([1, "text"]), json!([1, 2, 3, "text"]), true)]
    #[case(json!([1, 2, 3, "text"]), json!([1, "text"]), false)]
    #[case(json!([1, 2, 3, "text"]), Value::Array(Vec::new()), false)]
    #[case(json!([1, 2, 3, "text"]), Value::Object(Map::new()), false)]
    // Object
    #[case(json!({"a": 1, "b": 2, "c": "text"}), Value::Null, false)]
    #[case(json!({"a": 1, "b": 2, "c": "text"}), Value::Bool(false), false)]
    #[case(json!({"a": 1, "b": 2, "c": "text"}), Value::Number(1.into()), false)]
    #[case(json!({"a": 1, "b": 2, "c": "text"}), Value::String("text".to_string()), false)]
    #[case(json!({"a": 1, "b": 2, "c": "text"}), Value::Array(Vec::new()), false)]
    #[case(json!({"a": 1, "b": 2, "c": "text"}), Value::Object(Map::new()), false)]
    #[case(json!({"a": 1, "b": 2, "c": "text"}), json!({"a": 1, "b": 2, "c": "text"}), true)]
    #[case(json!({"a": 1, "b": 2, "c": "text"}), json!({"a": 1, "c": "text"}), false)]
    #[case(json!({"a": 1, "c": "text"}), json!({"a": 1, "b": 2, "c": "text"}), true)]
    // Nested
    #[case(json!({"a": { "b": [1, 2], "c": "text"}, "d": [3, 4, 5]}), json!({"a": { "b": [1, 2], "c": "text"}, "d": [3, 4, 5]}), true)]
    #[case(json!({"a": { "b": [1, 2], "c": "text"}, "d": [3, 4, 5]}), json!({"a": { "b": [1], "c": "text"}, "d": [3, 4, 5]}), false)]
    #[case(json!({"a": { "b": [1, 2], "c": "text"}, "d": [3, 4, 5]}), json!({"a": { "b": [1, 2]}, "d": [3, 4, 5]}), false)]
    #[case(json!({"a": { "b": [1, 2], "c": "text"}, "d": [3, 4, 5]}), json!({"a": [1, 2], "d": [3, 4, 5]}), false)]
    #[case(json!({"a": { "b": [1, 2], "c": "text"}, "d": [3, 4, 5]}), json!({"a": { "b": [1, 2], "c": "text"}, "d": [3, 5]}), false)]
    #[case(json!({"a": { "b": [1, 2]}, "d": [3, 4, 5]}), json!({"a": { "b": [1, 2], "c": "text"}, "d": [3, 5]}), false)]
    #[case(json!({"a": { "b": [1, 2], "c": "text"}, "d": [3, 5]}), json!({"a": { "b": [1, 2], "c": "text"}, "d": [3, 4, 5]}), true)]
    #[case(json!({"a": { "b": [1, 2]}, "d": [3, 4, 5]}), json!({"a": { "b": [1, 2], "c": "text"}, "d": [3, 4, 5]}), true)]
    #[case(json!({"a": { "b": [1], "c": "text"}, "d": [3, 4, 5]}), json!({"a": { "b": [1, 2], "c": "text"}, "d": [3, 4, 5]}), true)]
    #[case(json!({"a": { "b": [1, 2], "c": "text"}}), json!({"a": { "b": [1, 2], "c": "text"}, "d": [3, 4, 5]}), true)]
    fn json_eq(#[case] a: Value, #[case] b: Value, #[case] expected: bool) {
        assert_that!(a.json_eq(&b)).is_equal_to(expected);
    }
}
