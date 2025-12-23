//! Built-in functions for DOL expression evaluation.
//!
//! This module provides standard library functions available
//! in the DOL runtime environment.

use crate::eval::value::{EvalError, Value};

#[cfg(test)]
use std::collections::HashMap;

/// Calls a built-in function by name with the given arguments.
pub fn call_builtin(name: &str, args: &[Value]) -> Result<Value, EvalError> {
    match name {
        "print" => builtin_print(args),
        "typeof" => builtin_typeof(args),
        "len" => builtin_len(args),
        "push" => builtin_push(args),
        "pop" => builtin_pop(args),
        "keys" => builtin_keys(args),
        "values" => builtin_values(args),
        _ => Err(EvalError::new(format!("unknown builtin: {}", name))),
    }
}

/// print(value) - Prints a value to stdout.
///
/// Returns Void.
fn builtin_print(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::arity_mismatch(1, args.len()));
    }

    println!("{}", args[0]);
    Ok(Value::Void)
}

/// typeof(value) - Returns the type name of a value as a string.
///
/// # Examples
///
/// - `typeof(42)` returns `"Int"`
/// - `typeof("hello")` returns `"String"`
fn builtin_typeof(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::arity_mismatch(1, args.len()));
    }

    Ok(Value::String(args[0].type_name().to_string()))
}

/// len(array) - Returns the length of an array.
///
/// Also works on strings to return character count.
fn builtin_len(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::arity_mismatch(1, args.len()));
    }

    match &args[0] {
        Value::Array(items) => Ok(Value::Int(items.len() as i64)),
        Value::String(s) => Ok(Value::Int(s.len() as i64)),
        Value::Record(fields) => Ok(Value::Int(fields.len() as i64)),
        _ => Err(EvalError::type_error(
            "Array, String, or Record",
            args[0].type_name(),
        )),
    }
}

/// push(array, value) - Returns a new array with value appended.
///
/// Does not mutate the original array (functional style).
fn builtin_push(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::arity_mismatch(2, args.len()));
    }

    match &args[0] {
        Value::Array(items) => {
            let mut new_items = items.clone();
            new_items.push(args[1].clone());
            Ok(Value::Array(new_items))
        }
        _ => Err(EvalError::type_error("Array", args[0].type_name())),
    }
}

/// pop(array) - Returns a tuple of (last element, rest of array).
///
/// Returns an error if the array is empty.
fn builtin_pop(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::arity_mismatch(1, args.len()));
    }

    match &args[0] {
        Value::Array(items) => {
            if items.is_empty() {
                return Err(EvalError::new("cannot pop from empty array"));
            }

            let mut rest = items.clone();
            let last = rest.pop().unwrap();

            Ok(Value::Array(vec![last, Value::Array(rest)]))
        }
        _ => Err(EvalError::type_error("Array", args[0].type_name())),
    }
}

/// keys(record) - Returns an array of record keys.
fn builtin_keys(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::arity_mismatch(1, args.len()));
    }

    match &args[0] {
        Value::Record(fields) => {
            let keys: Vec<Value> = fields.keys().map(|k| Value::String(k.clone())).collect();
            Ok(Value::Array(keys))
        }
        _ => Err(EvalError::type_error("Record", args[0].type_name())),
    }
}

/// values(record) - Returns an array of record values.
fn builtin_values(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::arity_mismatch(1, args.len()));
    }

    match &args[0] {
        Value::Record(fields) => {
            let values: Vec<Value> = fields.values().cloned().collect();
            Ok(Value::Array(values))
        }
        _ => Err(EvalError::type_error("Record", args[0].type_name())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typeof() {
        let result = builtin_typeof(&[Value::Int(42)]).unwrap();
        assert_eq!(result, Value::String("Int".to_string()));

        let result = builtin_typeof(&[Value::String("hello".to_string())]).unwrap();
        assert_eq!(result, Value::String("String".to_string()));
    }

    #[test]
    fn test_len() {
        let arr = Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let result = builtin_len(&[arr]).unwrap();
        assert_eq!(result, Value::Int(3));

        let str_val = Value::String("hello".to_string());
        let result = builtin_len(&[str_val]).unwrap();
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_push() {
        let arr = Value::Array(vec![Value::Int(1), Value::Int(2)]);
        let result = builtin_push(&[arr, Value::Int(3)]).unwrap();

        match result {
            Value::Array(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[2], Value::Int(3));
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_pop() {
        let arr = Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let result = builtin_pop(&[arr]).unwrap();

        match result {
            Value::Array(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], Value::Int(3));
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_keys_values() {
        let mut fields = HashMap::new();
        fields.insert("a".to_string(), Value::Int(1));
        fields.insert("b".to_string(), Value::Int(2));
        let record = Value::Record(fields);

        let keys = builtin_keys(std::slice::from_ref(&record)).unwrap();
        if let Value::Array(items) = keys {
            assert_eq!(items.len(), 2);
        } else {
            panic!("Expected array");
        }

        let values = builtin_values(&[record]).unwrap();
        if let Value::Array(items) = values {
            assert_eq!(items.len(), 2);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_arity_errors() {
        assert!(builtin_typeof(&[]).is_err());
        assert!(builtin_typeof(&[Value::Int(1), Value::Int(2)]).is_err());
        assert!(builtin_len(&[]).is_err());
        assert!(builtin_push(&[Value::Array(vec![])]).is_err());
    }
}
