//! Type mapping from DOL to WIT types
//!
//! This module handles the conversion of DOL type expressions to WIT type strings,
//! with special handling for CRDT-compatible types and Component Model constraints.
//!
//! # WIT Type System
//!
//! WIT supports the following types:
//! - Primitives: bool, u8, u16, u32, u64, s8, s16, s32, s64, f32, f64, char, string
//! - Containers: list<T>, option<T>, result<T, E>, tuple<T1, T2, ...>
//! - User-defined: record, variant, enum, flags, resource
//!
//! Note: WIT does not have a native map type. Maps are represented as `list<tuple<K, V>>`.

use metadol::ast::TypeExpr;

/// Map a DOL TypeExpr to a WIT type string
pub fn map_type_expr(ty: &TypeExpr) -> String {
    match ty {
        TypeExpr::Named(name) => map_named_type(name),
        TypeExpr::Generic { name, args } => map_generic_type(name, args),
        TypeExpr::Function {
            params,
            return_type,
        } => map_function_type(params, return_type),
        TypeExpr::Tuple(types) => map_tuple_type(types),
        TypeExpr::Never => panic!("Never type cannot be mapped to WIT"),
        TypeExpr::Enum { .. } => {
            // Inline enums are not directly supported in WIT field types
            // They should be extracted to named enum declarations
            "u32".to_string()
        }
    }
}

/// Map a named DOL type to WIT
fn map_named_type(name: &str) -> String {
    match name {
        // String types
        "String" | "str" => "string".to_string(),

        // Boolean
        "Bool" | "bool" => "bool".to_string(),

        // Signed integers
        "Int" | "i32" | "Int32" => "s32".to_string(),
        "Int8" | "i8" => "s8".to_string(),
        "Int16" | "i16" => "s16".to_string(),
        "Int64" | "i64" => "s64".to_string(),

        // Unsigned integers
        "UInt" | "u32" | "UInt32" => "u32".to_string(),
        "UInt8" | "u8" => "u8".to_string(),
        "UInt16" | "u16" => "u16".to_string(),
        "UInt64" | "u64" => "u64".to_string(),

        // Floats
        "Float" | "Float32" | "f32" => "f32".to_string(),
        "Float64" | "f64" => "f64".to_string(),

        // Unit type maps to empty tuple
        "()" => "tuple<>".to_string(),

        // Char
        "char" | "Char" => "char".to_string(),

        // Custom types - convert to kebab-case for WIT convention
        name => to_wit_case(name),
    }
}

/// Map a generic DOL type to WIT
fn map_generic_type(name: &str, args: &[TypeExpr]) -> String {
    match name {
        "Option" => {
            if args.len() != 1 {
                panic!("Option requires exactly one type argument");
            }
            format!("option<{}>", map_type_expr(&args[0]))
        }
        "Result" => {
            if args.len() != 2 {
                panic!("Result requires exactly two type arguments");
            }
            format!(
                "result<{}, {}>",
                map_type_expr(&args[0]),
                map_type_expr(&args[1])
            )
        }
        "Vec" | "List" => {
            if args.len() != 1 {
                panic!("Vec/List requires exactly one type argument");
            }
            format!("list<{}>", map_type_expr(&args[0]))
        }
        "Set" => {
            // WIT doesn't have a native set type, use list
            if args.len() != 1 {
                panic!("Set requires exactly one type argument");
            }
            format!("list<{}>", map_type_expr(&args[0]))
        }
        "Map" => {
            // WIT doesn't have a native map type, use list of tuples
            if args.len() != 2 {
                panic!("Map requires exactly two type arguments");
            }
            format!(
                "list<tuple<{}, {}>>",
                map_type_expr(&args[0]),
                map_type_expr(&args[1])
            )
        }
        // Custom generic types
        name => {
            let type_name = to_wit_case(name);
            let type_args = args
                .iter()
                .map(map_type_expr)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}<{}>", type_name, type_args)
        }
    }
}

/// Map a function type to WIT
fn map_function_type(params: &[TypeExpr], return_type: &TypeExpr) -> String {
    let param_types = params
        .iter()
        .map(map_type_expr)
        .collect::<Vec<_>>()
        .join(", ");
    let ret_type = map_type_expr(return_type);

    // WIT doesn't support function types directly
    // This would need to be a callback interface or similar
    format!("func({}) -> {}", param_types, ret_type)
}

/// Map a tuple type to WIT
fn map_tuple_type(types: &[TypeExpr]) -> String {
    if types.is_empty() {
        "tuple<>".to_string()
    } else {
        let type_strs = types
            .iter()
            .map(map_type_expr)
            .collect::<Vec<_>>()
            .join(", ");
        format!("tuple<{}>", type_strs)
    }
}

/// Convert a DOL identifier to WIT kebab-case convention
///
/// WIT uses kebab-case for identifiers:
/// - ChatMessage → chat-message
/// - userId → user-id
/// - HTTPServer → http-server
pub fn to_wit_case(name: &str) -> String {
    heck::AsKebabCase(name).to_string()
}

/// Convert a DOL field name to WIT kebab-case
pub fn to_wit_field_name(name: &str) -> String {
    heck::AsKebabCase(name).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_primitives() {
        assert_eq!(map_named_type("String"), "string");
        assert_eq!(map_named_type("Bool"), "bool");
        assert_eq!(map_named_type("Int"), "s32");
        assert_eq!(map_named_type("Float"), "f32");
    }

    #[test]
    fn test_map_integer_types() {
        assert_eq!(map_named_type("Int8"), "s8");
        assert_eq!(map_named_type("Int16"), "s16");
        assert_eq!(map_named_type("Int32"), "s32");
        assert_eq!(map_named_type("Int64"), "s64");
        assert_eq!(map_named_type("UInt8"), "u8");
        assert_eq!(map_named_type("UInt16"), "u16");
        assert_eq!(map_named_type("UInt32"), "u32");
        assert_eq!(map_named_type("UInt64"), "u64");
    }

    #[test]
    fn test_map_option() {
        let option_type = TypeExpr::Generic {
            name: "Option".to_string(),
            args: vec![TypeExpr::Named("String".to_string())],
        };
        assert_eq!(map_type_expr(&option_type), "option<string>");
    }

    #[test]
    fn test_map_result() {
        let result_type = TypeExpr::Generic {
            name: "Result".to_string(),
            args: vec![
                TypeExpr::Named("String".to_string()),
                TypeExpr::Named("String".to_string()),
            ],
        };
        assert_eq!(map_type_expr(&result_type), "result<string, string>");
    }

    #[test]
    fn test_map_list() {
        let vec_type = TypeExpr::Generic {
            name: "Vec".to_string(),
            args: vec![TypeExpr::Named("String".to_string())],
        };
        assert_eq!(map_type_expr(&vec_type), "list<string>");

        let list_type = TypeExpr::Generic {
            name: "List".to_string(),
            args: vec![TypeExpr::Named("Int".to_string())],
        };
        assert_eq!(map_type_expr(&list_type), "list<s32>");
    }

    #[test]
    fn test_map_set() {
        let set_type = TypeExpr::Generic {
            name: "Set".to_string(),
            args: vec![TypeExpr::Named("String".to_string())],
        };
        // Set maps to list in WIT (no native set type)
        assert_eq!(map_type_expr(&set_type), "list<string>");
    }

    #[test]
    fn test_map_map() {
        let map_type = TypeExpr::Generic {
            name: "Map".to_string(),
            args: vec![
                TypeExpr::Named("String".to_string()),
                TypeExpr::Named("Int".to_string()),
            ],
        };
        // Map maps to list<tuple<K, V>> in WIT
        assert_eq!(map_type_expr(&map_type), "list<tuple<string, s32>>");
    }

    #[test]
    fn test_map_tuple() {
        let tuple_type = vec![
            TypeExpr::Named("String".to_string()),
            TypeExpr::Named("Int".to_string()),
            TypeExpr::Named("Bool".to_string()),
        ];
        assert_eq!(map_tuple_type(&tuple_type), "tuple<string, s32, bool>");

        let empty_tuple: Vec<TypeExpr> = vec![];
        assert_eq!(map_tuple_type(&empty_tuple), "tuple<>");
    }

    #[test]
    fn test_to_wit_case() {
        assert_eq!(to_wit_case("ChatMessage"), "chat-message");
        assert_eq!(to_wit_case("userId"), "user-id");
        assert_eq!(to_wit_case("HTTPServer"), "http-server");
        assert_eq!(to_wit_case("simple"), "simple");
    }

    #[test]
    fn test_nested_generics() {
        let nested = TypeExpr::Generic {
            name: "Option".to_string(),
            args: vec![TypeExpr::Generic {
                name: "List".to_string(),
                args: vec![TypeExpr::Named("String".to_string())],
            }],
        };
        assert_eq!(map_type_expr(&nested), "option<list<string>>");
    }
}
