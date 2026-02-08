//! Type mapping from DOL to Rust types
//!
//! This module handles the conversion of DOL type expressions to Rust type strings,
//! with special handling for Automerge-compatible types.

use dol::ast::TypeExpr;

/// Map a DOL TypeExpr to a Rust type string
pub fn map_type_expr(ty: &TypeExpr) -> String {
    match ty {
        TypeExpr::Named(name) => map_named_type(name),
        TypeExpr::Generic { name, args } => map_generic_type(name, args),
        TypeExpr::Function {
            params,
            return_type,
        } => map_function_type(params, return_type),
        TypeExpr::Tuple(types) => map_tuple_type(types),
        TypeExpr::Never => "!".to_string(),
        TypeExpr::Enum { .. } => {
            // Inline enums are not supported in field types yet
            "()".to_string()
        }
    }
}

/// Map a named DOL type to Rust
fn map_named_type(name: &str) -> String {
    match name {
        // Primitive types
        "String" => "String".to_string(),
        "str" => "String".to_string(), // Convert to owned String for Automerge
        "Bool" | "bool" => "bool".to_string(),

        // Integer types
        "Int" | "i32" => "i32".to_string(),
        "Int8" | "i8" => "i8".to_string(),
        "Int16" | "i16" => "i16".to_string(),
        "Int32" => "i32".to_string(),
        "Int64" | "i64" => "i64".to_string(),
        "UInt" | "u32" => "u32".to_string(),
        "UInt8" | "u8" => "u8".to_string(),
        "UInt16" | "u16" => "u16".to_string(),
        "UInt32" => "u32".to_string(),
        "UInt64" | "u64" => "u64".to_string(),

        // Float types
        "Float" | "Float32" | "f32" => "f32".to_string(),
        "Float64" | "f64" => "f64".to_string(),

        // Unit type
        "()" => "()".to_string(),

        // Custom types - convert to PascalCase
        name => dol::codegen::to_pascal_case(name),
    }
}

/// Map a generic DOL type to Rust
fn map_generic_type(name: &str, args: &[TypeExpr]) -> String {
    match name {
        "Option" => {
            if args.len() != 1 {
                panic!("Option requires exactly one type argument");
            }
            format!("Option<{}>", map_type_expr(&args[0]))
        }
        "Result" => {
            if args.len() != 2 {
                panic!("Result requires exactly two type arguments");
            }
            format!(
                "Result<{}, {}>",
                map_type_expr(&args[0]),
                map_type_expr(&args[1])
            )
        }
        "Vec" | "List" => {
            if args.len() != 1 {
                panic!("Vec/List requires exactly one type argument");
            }
            format!("Vec<{}>", map_type_expr(&args[0]))
        }
        "Set" => {
            if args.len() != 1 {
                panic!("Set requires exactly one type argument");
            }
            format!("std::collections::HashSet<{}>", map_type_expr(&args[0]))
        }
        "Map" => {
            if args.len() != 2 {
                panic!("Map requires exactly two type arguments");
            }
            format!(
                "std::collections::HashMap<{}, {}>",
                map_type_expr(&args[0]),
                map_type_expr(&args[1])
            )
        }
        // Custom generic types
        name => {
            let type_name = dol::codegen::to_pascal_case(name);
            let type_args = args
                .iter()
                .map(map_type_expr)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}<{}>", type_name, type_args)
        }
    }
}

/// Map a function type
fn map_function_type(params: &[TypeExpr], return_type: &TypeExpr) -> String {
    let param_types = params
        .iter()
        .map(map_type_expr)
        .collect::<Vec<_>>()
        .join(", ");
    let ret_type = map_type_expr(return_type);

    if ret_type == "()" {
        format!("Box<dyn Fn({})>", param_types)
    } else {
        format!("Box<dyn Fn({}) -> {}>", param_types, ret_type)
    }
}

/// Map a tuple type
fn map_tuple_type(types: &[TypeExpr]) -> String {
    if types.is_empty() {
        "()".to_string()
    } else {
        let type_strs = types
            .iter()
            .map(map_type_expr)
            .collect::<Vec<_>>()
            .join(", ");
        format!("({})", type_strs)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_primitives() {
        assert_eq!(map_named_type("String"), "String");
        assert_eq!(map_named_type("Bool"), "bool");
        assert_eq!(map_named_type("Int"), "i32");
        assert_eq!(map_named_type("Float"), "f32");
    }

    #[test]
    fn test_map_integer_types() {
        assert_eq!(map_named_type("Int8"), "i8");
        assert_eq!(map_named_type("Int16"), "i16");
        assert_eq!(map_named_type("Int32"), "i32");
        assert_eq!(map_named_type("Int64"), "i64");
        assert_eq!(map_named_type("UInt8"), "u8");
        assert_eq!(map_named_type("UInt16"), "u16");
        assert_eq!(map_named_type("UInt32"), "u32");
        assert_eq!(map_named_type("UInt64"), "u64");
    }

    #[test]
    fn test_map_collections() {
        let vec_type = TypeExpr::Generic {
            name: "Vec".to_string(),
            args: vec![TypeExpr::Named("String".to_string())],
        };
        assert_eq!(map_type_expr(&vec_type), "Vec<String>");

        let set_type = TypeExpr::Generic {
            name: "Set".to_string(),
            args: vec![TypeExpr::Named("String".to_string())],
        };
        assert_eq!(
            map_type_expr(&set_type),
            "std::collections::HashSet<String>"
        );

        let map_type = TypeExpr::Generic {
            name: "Map".to_string(),
            args: vec![
                TypeExpr::Named("String".to_string()),
                TypeExpr::Named("Int".to_string()),
            ],
        };
        assert_eq!(
            map_type_expr(&map_type),
            "std::collections::HashMap<String, i32>"
        );
    }

    #[test]
    fn test_map_option() {
        let option_type = TypeExpr::Generic {
            name: "Option".to_string(),
            args: vec![TypeExpr::Named("String".to_string())],
        };
        assert_eq!(map_type_expr(&option_type), "Option<String>");
    }

    #[test]
    fn test_map_tuple() {
        let tuple_type = vec![
            TypeExpr::Named("String".to_string()),
            TypeExpr::Named("Int".to_string()),
        ];
        assert_eq!(map_tuple_type(&tuple_type), "(String, i32)");

        let empty_tuple: Vec<TypeExpr> = vec![];
        assert_eq!(map_tuple_type(&empty_tuple), "()");
    }
}
