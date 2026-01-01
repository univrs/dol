//! Gene memory layout computation for WASM compilation.
//!
//! This module computes the memory layout of DOL genes (struct-like types)
//! for WebAssembly linear memory, following C-like alignment rules.
//!
//! ## Overview
//!
//! DOL genes are the atomic units of the ontology, similar to structs in C or Rust.
//! When compiled to WASM, genes are laid out in linear memory with C-like layout
//! rules (size, alignment, padding).
//!
//! ## Type Mapping
//!
//! | DOL Type     | WASM Type | Size (bytes) | Alignment |
//! |--------------|-----------|--------------|-----------|
//! | `Int32`      | `i32`     | 4            | 4         |
//! | `Int64`      | `i64`     | 8            | 8         |
//! | `Float32`    | `f32`     | 4            | 4         |
//! | `Float64`    | `f64`     | 8            | 8         |
//! | `Bool`       | `i32`     | 4            | 4         |
//! | `Char`       | `i32`     | 4            | 4         |
//! | `Gene` (ref) | `i32`     | 4            | 4         |
//!
//! ## Layout Rules
//!
//! 1. Each field is placed at the lowest offset that satisfies its alignment
//! 2. The struct's alignment is the maximum alignment of all fields
//! 3. The struct is padded at the end to be a multiple of its alignment
//! 4. Fields are laid out in declaration order (no reordering)
//!
//! ## Example
//!
//! ```rust,ignore
//! use metadol::wasm::layout::{GeneLayoutRegistry, compute_gene_layout};
//!
//! let mut registry = GeneLayoutRegistry::new();
//! let layout = compute_gene_layout(&gene, &registry)?;
//!
//! assert_eq!(layout.total_size, 16);  // For a Point with two f64 fields
//! assert_eq!(layout.alignment, 8);
//! ```

use std::collections::HashMap;

use crate::wasm::WasmError;

/// WASM type information for field access.
///
/// Represents the primitive types available in WebAssembly,
/// plus a `Ptr` variant for pointer values (which are i32 in WASM).
///
/// # Example
///
/// ```rust,ignore
/// use metadol::wasm::layout::WasmFieldType;
///
/// let field_type = WasmFieldType::F64;
/// assert_eq!(field_type.size(), 8);
/// assert_eq!(field_type.alignment(), 8);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmFieldType {
    /// 32-bit integer
    I32,
    /// 64-bit integer
    I64,
    /// 32-bit float
    F32,
    /// 64-bit float
    F64,
    /// Pointer (i32 address in linear memory)
    Ptr,
}

impl WasmFieldType {
    /// Convert to wasm_encoder ValType.
    ///
    /// This is used when generating WASM function signatures and locals.
    #[cfg(feature = "wasm")]
    pub fn to_val_type(self) -> wasm_encoder::ValType {
        match self {
            WasmFieldType::I32 | WasmFieldType::Ptr => wasm_encoder::ValType::I32,
            WasmFieldType::I64 => wasm_encoder::ValType::I64,
            WasmFieldType::F32 => wasm_encoder::ValType::F32,
            WasmFieldType::F64 => wasm_encoder::ValType::F64,
        }
    }

    /// Get the size in bytes.
    ///
    /// Returns the number of bytes this type occupies in memory.
    pub fn size(self) -> u32 {
        match self {
            WasmFieldType::I32 | WasmFieldType::F32 | WasmFieldType::Ptr => 4,
            WasmFieldType::I64 | WasmFieldType::F64 => 8,
        }
    }

    /// Get the alignment requirement.
    ///
    /// For WASM, alignment equals size for primitive types.
    pub fn alignment(self) -> u32 {
        self.size() // For WASM, alignment == size for primitive types
    }

    /// Get the alignment as a log2 value for WASM MemArg.
    ///
    /// WASM memory instructions use log2(alignment) as their alignment parameter.
    pub fn alignment_log2(self) -> u32 {
        self.alignment().trailing_zeros()
    }
}

impl std::fmt::Display for WasmFieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WasmFieldType::I32 => write!(f, "i32"),
            WasmFieldType::I64 => write!(f, "i64"),
            WasmFieldType::F32 => write!(f, "f32"),
            WasmFieldType::F64 => write!(f, "f64"),
            WasmFieldType::Ptr => write!(f, "ptr"),
        }
    }
}

/// Describes a single field within a gene layout.
///
/// Contains all the information needed to generate WASM load/store
/// instructions for accessing this field.
///
/// # Example
///
/// ```rust,ignore
/// use metadol::wasm::layout::{FieldLayout, WasmFieldType};
///
/// let field = FieldLayout {
///     name: "x".to_string(),
///     offset: 0,
///     size: 8,
///     alignment: 8,
///     wasm_type: WasmFieldType::F64,
///     is_reference: false,
///     nested_layout: None,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct FieldLayout {
    /// Field name
    pub name: String,

    /// Byte offset from struct base address
    pub offset: u32,

    /// Size in bytes
    pub size: u32,

    /// Alignment requirement in bytes
    pub alignment: u32,

    /// WASM value type for load/store instructions
    pub wasm_type: WasmFieldType,

    /// True if this field is a pointer to another gene
    pub is_reference: bool,

    /// For inline genes, the nested layout; None for primitives
    pub nested_layout: Option<Box<GeneLayout>>,
}

impl FieldLayout {
    /// Create a new field layout for a primitive type.
    pub fn primitive(name: impl Into<String>, offset: u32, wasm_type: WasmFieldType) -> Self {
        Self {
            name: name.into(),
            offset,
            size: wasm_type.size(),
            alignment: wasm_type.alignment(),
            wasm_type,
            is_reference: false,
            nested_layout: None,
        }
    }

    /// Create a new field layout for a reference (pointer) type.
    pub fn reference(name: impl Into<String>, offset: u32) -> Self {
        Self {
            name: name.into(),
            offset,
            size: 4,
            alignment: 4,
            wasm_type: WasmFieldType::Ptr,
            is_reference: true,
            nested_layout: None,
        }
    }

    /// Create a new field layout for an inline (embedded) gene.
    pub fn inline(name: impl Into<String>, offset: u32, layout: GeneLayout) -> Self {
        Self {
            name: name.into(),
            offset,
            size: layout.total_size,
            alignment: layout.alignment,
            // For inline genes, we use I32 as a placeholder type.
            // Field access will traverse nested_layout for actual fields.
            wasm_type: WasmFieldType::I32,
            is_reference: false,
            nested_layout: Some(Box::new(layout)),
        }
    }
}

/// Describes the memory layout of a gene (struct-like type).
///
/// Contains the complete layout information needed to allocate
/// memory for a gene instance and access its fields.
///
/// # Example
///
/// ```rust,ignore
/// use metadol::wasm::layout::GeneLayout;
///
/// // Layout for a Point gene with x: f64, y: f64
/// let layout = GeneLayout {
///     name: "Point".to_string(),
///     fields: vec![/* ... */],
///     total_size: 16,
///     alignment: 8,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct GeneLayout {
    /// Fully qualified gene name (e.g., "geometry.Point")
    pub name: String,

    /// Fields in declaration order with computed offsets
    pub fields: Vec<FieldLayout>,

    /// Total size in bytes (including padding)
    pub total_size: u32,

    /// Alignment requirement in bytes
    pub alignment: u32,
}

impl GeneLayout {
    /// Create a new empty gene layout.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fields: Vec::new(),
            total_size: 0,
            alignment: 1,
        }
    }

    /// Get a field by name.
    ///
    /// Returns `None` if no field with the given name exists.
    pub fn get_field(&self, name: &str) -> Option<&FieldLayout> {
        self.fields.iter().find(|f| f.name == name)
    }

    /// Get a field's offset by name.
    ///
    /// Returns `None` if no field with the given name exists.
    pub fn get_field_offset(&self, name: &str) -> Option<u32> {
        self.get_field(name).map(|f| f.offset)
    }

    /// Get the type ID for this gene.
    ///
    /// This is a hash of the gene name, used for runtime type identification.
    /// In practice, a type registry with sequential IDs would be more efficient.
    pub fn type_id(&self) -> u32 {
        let mut hash = 0u32;
        for byte in self.name.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
        }
        hash
    }

    /// Returns true if this gene has any reference (pointer) fields.
    ///
    /// This is useful for GC root tracking.
    pub fn has_references(&self) -> bool {
        self.fields.iter().any(|f| f.is_reference)
    }

    /// Get the offsets of all pointer fields.
    ///
    /// Returns a vector of offsets for fields that contain pointers,
    /// useful for garbage collection traversal.
    pub fn pointer_offsets(&self) -> Vec<u32> {
        self.fields
            .iter()
            .filter(|f| f.is_reference)
            .map(|f| f.offset)
            .collect()
    }

    /// Get the number of fields.
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    /// Check if the gene has no fields.
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

impl Default for GeneLayout {
    fn default() -> Self {
        Self::new("")
    }
}

/// Registry for looking up gene layouts by name.
///
/// Maintains a collection of computed gene layouts that can be
/// referenced when computing layouts for genes with nested types.
///
/// # Example
///
/// ```rust,ignore
/// use metadol::wasm::layout::GeneLayoutRegistry;
///
/// let mut registry = GeneLayoutRegistry::new();
///
/// // Register a Point layout
/// registry.register(point_layout);
///
/// // Later, when computing Rectangle layout, we can look up Point
/// let point = registry.get("Point");
/// ```
#[derive(Debug, Default, Clone)]
pub struct GeneLayoutRegistry {
    layouts: HashMap<String, GeneLayout>,
}

impl GeneLayoutRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a layout in the registry.
    ///
    /// If a layout with the same name already exists, it will be replaced.
    pub fn register(&mut self, layout: GeneLayout) {
        self.layouts.insert(layout.name.clone(), layout);
    }

    /// Get a layout by name.
    ///
    /// Returns `None` if no layout with the given name has been registered.
    pub fn get(&self, name: &str) -> Option<&GeneLayout> {
        self.layouts.get(name)
    }

    /// Check if a layout with the given name exists.
    pub fn contains(&self, name: &str) -> bool {
        self.layouts.contains_key(name)
    }

    /// Get the number of registered layouts.
    pub fn len(&self) -> usize {
        self.layouts.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.layouts.is_empty()
    }

    /// Iterate over all registered layouts.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &GeneLayout)> {
        self.layouts.iter()
    }

    /// Remove a layout from the registry.
    ///
    /// Returns the removed layout if it existed.
    pub fn remove(&mut self, name: &str) -> Option<GeneLayout> {
        self.layouts.remove(name)
    }

    /// Clear all registered layouts.
    pub fn clear(&mut self) {
        self.layouts.clear();
    }
}

/// Align an offset up to the specified alignment.
///
/// Returns the smallest value >= offset that is a multiple of alignment.
///
/// # Arguments
///
/// * `offset` - The current byte offset
/// * `alignment` - The required alignment (must be a power of 2)
///
/// # Example
///
/// ```rust,ignore
/// use metadol::wasm::layout::align_up;
///
/// assert_eq!(align_up(5, 4), 8);  // 5 -> 8 (next multiple of 4)
/// assert_eq!(align_up(8, 4), 8);  // 8 -> 8 (already aligned)
/// assert_eq!(align_up(0, 8), 0);  // 0 -> 0 (already aligned)
/// ```
#[inline]
pub fn align_up(offset: u32, alignment: u32) -> u32 {
    debug_assert!(
        alignment.is_power_of_two(),
        "alignment must be a power of 2"
    );
    (offset + alignment - 1) & !(alignment - 1)
}

/// Information about a DOL type for layout computation.
#[derive(Debug, Clone)]
struct TypeInfo {
    wasm_type: WasmFieldType,
    size: u32,
    alignment: u32,
    is_reference: bool,
    nested_layout: Option<Box<GeneLayout>>,
}

/// Get type information from a DOL type expression.
///
/// Maps DOL type names to their WASM representation, size, and alignment.
fn type_to_wasm_info(
    type_expr: &crate::ast::TypeExpr,
    registry: &GeneLayoutRegistry,
) -> Result<TypeInfo, WasmError> {
    use crate::ast::TypeExpr;

    match type_expr {
        TypeExpr::Named(name) => match name.as_str() {
            // Primitive integer types
            "Int32" | "i32" => Ok(TypeInfo {
                wasm_type: WasmFieldType::I32,
                size: 4,
                alignment: 4,
                is_reference: false,
                nested_layout: None,
            }),
            "Int64" | "i64" => Ok(TypeInfo {
                wasm_type: WasmFieldType::I64,
                size: 8,
                alignment: 8,
                is_reference: false,
                nested_layout: None,
            }),
            // Floating point types
            "Float32" | "f32" => Ok(TypeInfo {
                wasm_type: WasmFieldType::F32,
                size: 4,
                alignment: 4,
                is_reference: false,
                nested_layout: None,
            }),
            "Float64" | "f64" => Ok(TypeInfo {
                wasm_type: WasmFieldType::F64,
                size: 8,
                alignment: 8,
                is_reference: false,
                nested_layout: None,
            }),
            // Boolean and character (represented as i32)
            "Bool" | "bool" => Ok(TypeInfo {
                wasm_type: WasmFieldType::I32,
                size: 4,
                alignment: 4,
                is_reference: false,
                nested_layout: None,
            }),
            "Char" | "char" => Ok(TypeInfo {
                wasm_type: WasmFieldType::I32,
                size: 4,
                alignment: 4,
                is_reference: false,
                nested_layout: None,
            }),
            // String is a pointer
            "String" => Ok(TypeInfo {
                wasm_type: WasmFieldType::Ptr,
                size: 4,
                alignment: 4,
                is_reference: true,
                nested_layout: None,
            }),
            // Check if it's a reference type (starts with &)
            _ if name.starts_with('&') => {
                // Reference to another gene - always a pointer
                Ok(TypeInfo {
                    wasm_type: WasmFieldType::Ptr,
                    size: 4,
                    alignment: 4,
                    is_reference: true,
                    nested_layout: None,
                })
            }
            // Look up as gene type (inline embedding)
            gene_name => {
                if let Some(layout) = registry.get(gene_name) {
                    // Inline embedding - the gene's fields are embedded directly
                    Ok(TypeInfo {
                        wasm_type: WasmFieldType::I32, // Placeholder for nested
                        size: layout.total_size,
                        alignment: layout.alignment,
                        is_reference: false,
                        nested_layout: Some(Box::new(layout.clone())),
                    })
                } else {
                    Err(WasmError::new(format!("Unknown type: {}", gene_name)))
                }
            }
        },
        TypeExpr::Generic { name, args: _ } => {
            // Handle reference types like &Point or List<T>
            if name.starts_with('&') {
                Ok(TypeInfo {
                    wasm_type: WasmFieldType::Ptr,
                    size: 4,
                    alignment: 4,
                    is_reference: true,
                    nested_layout: None,
                })
            } else if name == "List" || name == "Vec" || name == "Option" {
                // Collection types are pointers
                Ok(TypeInfo {
                    wasm_type: WasmFieldType::Ptr,
                    size: 4,
                    alignment: 4,
                    is_reference: true,
                    nested_layout: None,
                })
            } else {
                Err(WasmError::new(format!(
                    "Generic types not yet fully supported: {}",
                    name
                )))
            }
        }
        TypeExpr::Function { .. } => {
            // Function types are represented as function table indices (i32)
            Ok(TypeInfo {
                wasm_type: WasmFieldType::I32,
                size: 4,
                alignment: 4,
                is_reference: false,
                nested_layout: None,
            })
        }
        TypeExpr::Tuple(types) => {
            // Tuples need special handling - for now, treat as inline
            // TODO: Implement proper tuple layout
            if types.is_empty() {
                Ok(TypeInfo {
                    wasm_type: WasmFieldType::I32,
                    size: 0,
                    alignment: 1,
                    is_reference: false,
                    nested_layout: None,
                })
            } else {
                Err(WasmError::new("Tuple types not yet supported".to_string()))
            }
        }
        TypeExpr::Never => {
            // Never type has no runtime representation
            Err(WasmError::new(
                "Never type cannot have a memory layout".to_string(),
            ))
        }
        TypeExpr::Enum { .. } => {
            // Enums are represented as i32 discriminants (for now)
            // TODO: Implement proper enum layout with variant data
            Ok(TypeInfo {
                wasm_type: WasmFieldType::I32,
                size: 4,
                alignment: 4,
                is_reference: false,
                nested_layout: None,
            })
        }
    }
}

/// Compute the memory layout for a gene.
///
/// Processes the gene's field declarations and computes offsets,
/// following C-like struct layout rules with proper alignment and padding.
///
/// # Arguments
///
/// * `gene` - The gene AST node to compute layout for
/// * `registry` - Registry of previously computed gene layouts (for nested types)
///
/// # Returns
///
/// Returns a `GeneLayout` containing the computed field offsets and total size.
///
/// # Errors
///
/// Returns an error if:
/// - A field references an unknown type
/// - A type cannot be represented in WASM memory
///
/// # Example
///
/// ```rust,ignore
/// use metadol::wasm::layout::{compute_gene_layout, GeneLayoutRegistry};
///
/// let registry = GeneLayoutRegistry::new();
/// let layout = compute_gene_layout(&point_gene, &registry)?;
///
/// // Point { x: Float64, y: Float64 }
/// assert_eq!(layout.total_size, 16);
/// assert_eq!(layout.alignment, 8);
/// assert_eq!(layout.get_field("x").unwrap().offset, 0);
/// assert_eq!(layout.get_field("y").unwrap().offset, 8);
/// ```
pub fn compute_gene_layout(
    gene: &crate::ast::Gene,
    registry: &GeneLayoutRegistry,
) -> Result<GeneLayout, WasmError> {
    use crate::ast::Statement;

    let mut fields = Vec::new();
    let mut current_offset = 0u32;
    let mut max_alignment = 1u32;

    // Process each statement in the gene
    for stmt in &gene.statements {
        // Only process HasField statements (typed fields)
        if let Statement::HasField(field_box) = stmt {
            let field = field_box.as_ref();

            // Get type information for this field
            let type_info = type_to_wasm_info(&field.type_, registry)?;

            // Align current offset to field's required alignment
            current_offset = align_up(current_offset, type_info.alignment);

            // Create the field layout
            let field_layout = FieldLayout {
                name: field.name.clone(),
                offset: current_offset,
                size: type_info.size,
                alignment: type_info.alignment,
                wasm_type: type_info.wasm_type,
                is_reference: type_info.is_reference,
                nested_layout: type_info.nested_layout,
            };

            fields.push(field_layout);

            // Advance offset and track max alignment
            current_offset += type_info.size;
            max_alignment = max_alignment.max(type_info.alignment);
        }
    }

    // Pad struct to alignment (for arrays of structs)
    let total_size = if max_alignment > 0 {
        align_up(current_offset, max_alignment)
    } else {
        current_offset
    };

    Ok(GeneLayout {
        name: gene.name.clone(),
        fields,
        total_size,
        alignment: max_alignment,
    })
}

/// Type descriptor for GC traversal.
///
/// Provides information needed by a garbage collector to traverse
/// and manage gene instances in memory.
#[derive(Debug, Clone)]
pub struct GeneTypeDescriptor {
    /// Total size of the gene in bytes
    pub size: u32,

    /// Offsets of pointer fields (for GC traversal)
    pub pointer_offsets: Vec<u32>,

    /// Whether this type contains any references
    pub has_references: bool,
}

impl GeneLayout {
    /// Convert to a GC type descriptor.
    ///
    /// Creates a descriptor that can be used by a garbage collector
    /// to properly traverse and manage gene instances.
    pub fn to_gc_descriptor(&self) -> GeneTypeDescriptor {
        let pointer_offsets = self.pointer_offsets();
        GeneTypeDescriptor {
            size: self.total_size,
            has_references: !pointer_offsets.is_empty(),
            pointer_offsets,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Gene, HasField, Span, Statement, TypeExpr};

    fn make_field(name: &str, type_name: &str) -> Statement {
        Statement::HasField(Box::new(HasField {
            name: name.to_string(),
            type_: TypeExpr::Named(type_name.to_string()),
            default: None,
            constraint: None,
            span: Span::default(),
        }))
    }

    fn make_gene(name: &str, statements: Vec<Statement>) -> Gene {
        Gene {
            name: name.to_string(),
            extends: None,
            statements,
            exegesis: "Test gene".to_string(),
            span: Span::default(),
        }
    }

    #[test]
    fn test_wasm_field_type_sizes() {
        assert_eq!(WasmFieldType::I32.size(), 4);
        assert_eq!(WasmFieldType::I64.size(), 8);
        assert_eq!(WasmFieldType::F32.size(), 4);
        assert_eq!(WasmFieldType::F64.size(), 8);
        assert_eq!(WasmFieldType::Ptr.size(), 4);
    }

    #[test]
    fn test_wasm_field_type_alignments() {
        assert_eq!(WasmFieldType::I32.alignment(), 4);
        assert_eq!(WasmFieldType::I64.alignment(), 8);
        assert_eq!(WasmFieldType::F32.alignment(), 4);
        assert_eq!(WasmFieldType::F64.alignment(), 8);
        assert_eq!(WasmFieldType::Ptr.alignment(), 4);
    }

    #[test]
    fn test_align_up() {
        assert_eq!(align_up(0, 4), 0);
        assert_eq!(align_up(1, 4), 4);
        assert_eq!(align_up(4, 4), 4);
        assert_eq!(align_up(5, 4), 8);
        assert_eq!(align_up(0, 8), 0);
        assert_eq!(align_up(4, 8), 8);
        assert_eq!(align_up(8, 8), 8);
        assert_eq!(align_up(9, 8), 16);
    }

    #[test]
    fn test_simple_point_layout() {
        // gene Point { has x: Float64, has y: Float64 }
        let gene = make_gene(
            "Point",
            vec![make_field("x", "Float64"), make_field("y", "Float64")],
        );

        let registry = GeneLayoutRegistry::new();
        let layout = compute_gene_layout(&gene, &registry).unwrap();

        assert_eq!(layout.name, "Point");
        assert_eq!(layout.total_size, 16);
        assert_eq!(layout.alignment, 8);
        assert_eq!(layout.fields.len(), 2);

        assert_eq!(layout.fields[0].name, "x");
        assert_eq!(layout.fields[0].offset, 0);
        assert_eq!(layout.fields[0].wasm_type, WasmFieldType::F64);

        assert_eq!(layout.fields[1].name, "y");
        assert_eq!(layout.fields[1].offset, 8);
        assert_eq!(layout.fields[1].wasm_type, WasmFieldType::F64);
    }

    #[test]
    fn test_mixed_types_with_padding() {
        // gene Entity {
        //   has id: Int32
        //   has position: Float64  // Requires 8-byte alignment
        //   has active: Bool
        //   has score: Int32
        // }
        let gene = make_gene(
            "Entity",
            vec![
                make_field("id", "Int32"),
                make_field("position", "Float64"),
                make_field("active", "Bool"),
                make_field("score", "Int32"),
            ],
        );

        let registry = GeneLayoutRegistry::new();
        let layout = compute_gene_layout(&gene, &registry).unwrap();

        assert_eq!(layout.name, "Entity");
        assert_eq!(layout.alignment, 8);

        // id at offset 0 (4 bytes)
        assert_eq!(layout.fields[0].offset, 0);
        // position at offset 8 (aligned to 8, not 4)
        assert_eq!(layout.fields[1].offset, 8);
        // active at offset 16 (4 bytes)
        assert_eq!(layout.fields[2].offset, 16);
        // score at offset 20 (4 bytes)
        assert_eq!(layout.fields[3].offset, 20);

        // Total: 24 bytes (20 + 4, padded to 8-byte alignment)
        assert_eq!(layout.total_size, 24);
    }

    #[test]
    fn test_gene_layout_registry() {
        let mut registry = GeneLayoutRegistry::new();

        let point_layout = GeneLayout {
            name: "Point".to_string(),
            fields: vec![],
            total_size: 16,
            alignment: 8,
        };

        registry.register(point_layout);

        assert!(registry.contains("Point"));
        assert!(!registry.contains("Rectangle"));
        assert_eq!(registry.len(), 1);

        let retrieved = registry.get("Point").unwrap();
        assert_eq!(retrieved.total_size, 16);
    }

    #[test]
    fn test_nested_gene_layout() {
        // First, create and register Point
        let point_gene = make_gene(
            "Point",
            vec![make_field("x", "Float64"), make_field("y", "Float64")],
        );

        let mut registry = GeneLayoutRegistry::new();
        let point_layout = compute_gene_layout(&point_gene, &registry).unwrap();
        registry.register(point_layout);

        // Now create Rectangle with inline Point
        let rect_gene = make_gene(
            "Rectangle",
            vec![
                make_field("top_left", "Point"),
                make_field("width", "Float64"),
                make_field("height", "Float64"),
            ],
        );

        let rect_layout = compute_gene_layout(&rect_gene, &registry).unwrap();

        assert_eq!(rect_layout.name, "Rectangle");
        assert_eq!(rect_layout.alignment, 8);
        // Point (16) + width (8) + height (8) = 32
        assert_eq!(rect_layout.total_size, 32);

        // top_left at offset 0
        assert_eq!(rect_layout.fields[0].offset, 0);
        assert_eq!(rect_layout.fields[0].size, 16);
        assert!(rect_layout.fields[0].nested_layout.is_some());

        // width at offset 16
        assert_eq!(rect_layout.fields[1].offset, 16);

        // height at offset 24
        assert_eq!(rect_layout.fields[2].offset, 24);
    }

    #[test]
    fn test_reference_type_layout() {
        // gene Node {
        //   has value: Int64
        //   has next: &Node  (reference)
        // }
        let gene = Gene {
            name: "Node".to_string(),
            extends: None,
            statements: vec![
                make_field("value", "Int64"),
                Statement::HasField(Box::new(HasField {
                    name: "next".to_string(),
                    type_: TypeExpr::Named("&Node".to_string()),
                    default: None,
                    constraint: None,
                    span: Span::default(),
                })),
            ],
            exegesis: "Test".to_string(),
            span: Span::default(),
        };

        let registry = GeneLayoutRegistry::new();
        let layout = compute_gene_layout(&gene, &registry).unwrap();

        assert_eq!(layout.fields[0].name, "value");
        assert_eq!(layout.fields[0].offset, 0);
        assert_eq!(layout.fields[0].wasm_type, WasmFieldType::I64);
        assert!(!layout.fields[0].is_reference);

        assert_eq!(layout.fields[1].name, "next");
        assert_eq!(layout.fields[1].offset, 8);
        assert_eq!(layout.fields[1].wasm_type, WasmFieldType::Ptr);
        assert!(layout.fields[1].is_reference);

        // Total: 12 bytes, but padded to 16 for 8-byte alignment
        assert_eq!(layout.total_size, 16);
    }

    #[test]
    fn test_gc_descriptor() {
        let layout = GeneLayout {
            name: "Node".to_string(),
            fields: vec![
                FieldLayout {
                    name: "value".to_string(),
                    offset: 0,
                    size: 8,
                    alignment: 8,
                    wasm_type: WasmFieldType::I64,
                    is_reference: false,
                    nested_layout: None,
                },
                FieldLayout {
                    name: "next".to_string(),
                    offset: 8,
                    size: 4,
                    alignment: 4,
                    wasm_type: WasmFieldType::Ptr,
                    is_reference: true,
                    nested_layout: None,
                },
                FieldLayout {
                    name: "prev".to_string(),
                    offset: 12,
                    size: 4,
                    alignment: 4,
                    wasm_type: WasmFieldType::Ptr,
                    is_reference: true,
                    nested_layout: None,
                },
            ],
            total_size: 16,
            alignment: 8,
        };

        let descriptor = layout.to_gc_descriptor();
        assert_eq!(descriptor.size, 16);
        assert!(descriptor.has_references);
        assert_eq!(descriptor.pointer_offsets, vec![8, 12]);
    }

    #[test]
    fn test_empty_gene() {
        let gene = make_gene("Empty", vec![]);
        let registry = GeneLayoutRegistry::new();
        let layout = compute_gene_layout(&gene, &registry).unwrap();

        assert_eq!(layout.total_size, 0);
        assert_eq!(layout.alignment, 1);
        assert!(layout.is_empty());
    }

    #[test]
    fn test_field_layout_constructors() {
        let primitive = FieldLayout::primitive("x", 0, WasmFieldType::F64);
        assert_eq!(primitive.size, 8);
        assert_eq!(primitive.alignment, 8);
        assert!(!primitive.is_reference);

        let reference = FieldLayout::reference("next", 8);
        assert_eq!(reference.size, 4);
        assert_eq!(reference.wasm_type, WasmFieldType::Ptr);
        assert!(reference.is_reference);
    }
}
