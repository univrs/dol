# DOL Genes to WASM Linear Memory Design

This document describes the memory layout and access patterns for compiling DOL genes (struct-like declarations) to WebAssembly linear memory.

## Table of Contents

1. [Overview](#overview)
2. [Type Mapping and Size/Alignment Rules](#type-mapping-and-sizealignment-rules)
3. [Gene Layout Computation](#gene-layout-computation)
4. [Memory Layout Diagrams](#memory-layout-diagrams)
5. [Rust Struct Definitions](#rust-struct-definitions)
6. [Memory Allocator Design](#memory-allocator-design)
7. [Field Access Patterns](#field-access-patterns)
8. [Constructor Emission](#constructor-emission)
9. [Nested Gene Handling](#nested-gene-handling)
10. [GC Considerations](#gc-considerations)
11. [Example Compilation](#example-compilation)

---

## Overview

DOL genes are the atomic units of the ontology, similar to structs in C or Rust. When compiled to WASM, genes are laid out in linear memory with C-like layout rules (size, alignment, padding).

### Design Goals

1. **Predictable Layout**: Follow C ABI conventions for struct layout
2. **Efficient Access**: Field access should compile to single load/store instructions
3. **Pointer Representation**: Gene references are i32 pointers to linear memory
4. **Simple Allocation**: Start with a bump allocator; GC can be added later
5. **Nested Support**: Genes containing other genes (by value or reference)

---

## Type Mapping and Size/Alignment Rules

### Primitive Types

| DOL Type     | WASM Type | Size (bytes) | Alignment |
|--------------|-----------|--------------|-----------|
| `Int32`      | `i32`     | 4            | 4         |
| `Int64`      | `i64`     | 8            | 8         |
| `Float32`    | `f32`     | 4            | 4         |
| `Float64`    | `f64`     | 8            | 8         |
| `Bool`       | `i32`     | 4            | 4         |
| `Char`       | `i32`     | 4            | 4         |

### Reference Types

| DOL Type       | WASM Representation | Size (bytes) | Alignment |
|----------------|---------------------|--------------|-----------|
| `Gene` (ref)   | `i32` (pointer)     | 4            | 4         |
| `String`       | `i32` (pointer)     | 4            | 4         |
| `List<T>`      | `i32` (pointer)     | 4            | 4         |
| `Option<T>`    | varies              | varies       | varies    |

### Inline vs Reference

Genes can be stored inline or by reference:

- **Inline** (`has point: Point`): The Point's fields are embedded directly
- **Reference** (`has point: &Point`): Only a 4-byte pointer is stored

```text
Inline Point in Rectangle:     Reference Point in Rectangle:
+------------------+           +------------------+
| top_left.x: f64  |           | top_left: i32 ---|--> [Point in heap]
| top_left.y: f64  |           | width: f64       |
| width: f64       |           | height: f64      |
| height: f64      |           +------------------+
+------------------+
```

---

## Gene Layout Computation

### Algorithm

```rust
fn compute_gene_layout(gene: &Gene, registry: &TypeRegistry) -> GeneLayout {
    let mut fields = Vec::new();
    let mut current_offset = 0u32;
    let mut max_alignment = 1u32;

    for stmt in &gene.statements {
        if let Statement::HasField(field) = stmt {
            let field_info = get_type_info(&field.type_, registry);
            let alignment = field_info.alignment;
            let size = field_info.size;

            // Align current offset to field's required alignment
            current_offset = align_up(current_offset, alignment);

            fields.push(FieldLayout {
                name: field.name.clone(),
                offset: current_offset,
                size,
                alignment,
                wasm_type: field_info.wasm_type,
                is_reference: field_info.is_reference,
            });

            current_offset += size;
            max_alignment = max_alignment.max(alignment);
        }
    }

    // Pad struct to alignment (for arrays of structs)
    let total_size = align_up(current_offset, max_alignment);

    GeneLayout {
        name: gene.name.clone(),
        fields,
        total_size,
        alignment: max_alignment,
    }
}

fn align_up(offset: u32, alignment: u32) -> u32 {
    (offset + alignment - 1) & !(alignment - 1)
}
```

### Layout Rules

1. Each field is placed at the lowest offset that satisfies its alignment
2. The struct's alignment is the maximum alignment of all fields
3. The struct is padded at the end to be a multiple of its alignment
4. Fields are laid out in declaration order (no reordering)

---

## Memory Layout Diagrams

### Example 1: Simple Point Gene

```dol
gene Point {
    has x: Float64
    has y: Float64
}
```

Memory Layout (16 bytes, 8-byte aligned):
```text
Offset  Size  Field
------  ----  -----
0       8     x (f64)
8       8     y (f64)
------
Total: 16 bytes
```

```text
+-------+-------+-------+-------+-------+-------+-------+-------+
|          x: Float64 (8 bytes)                                 |
+-------+-------+-------+-------+-------+-------+-------+-------+
|          y: Float64 (8 bytes)                                 |
+-------+-------+-------+-------+-------+-------+-------+-------+
  0       1       2       3       4       5       6       7
```

### Example 2: Rectangle with Inline Point

```dol
gene Rectangle {
    has top_left: Point    // Inline (embedded)
    has width: Float64
    has height: Float64
}
```

Memory Layout (32 bytes, 8-byte aligned):
```text
Offset  Size  Field
------  ----  -----
0       8     top_left.x (f64)
8       8     top_left.y (f64)
16      8     width (f64)
24      8     height (f64)
------
Total: 32 bytes
```

```text
+---------------------------------------------------------------+
|                    top_left.x (8 bytes)                       |
+---------------------------------------------------------------+
|                    top_left.y (8 bytes)                       |
+---------------------------------------------------------------+
|                    width (8 bytes)                            |
+---------------------------------------------------------------+
|                    height (8 bytes)                           |
+---------------------------------------------------------------+
  0       8      16      24      32
```

### Example 3: Mixed Types with Padding

```dol
gene Entity {
    has id: Int32
    has position: Float64    // Requires 8-byte alignment
    has active: Bool
    has score: Int32
}
```

Memory Layout (24 bytes, 8-byte aligned):
```text
Offset  Size  Field
------  ----  -----
0       4     id (i32)
4       4     [padding]
8       8     position (f64)
16      4     active (i32)
20      4     score (i32)
------
Total: 24 bytes
```

```text
+-------+-------+-------+-------+-------+-------+-------+-------+
| id (4)|[pad 4]|          position (8 bytes)                   |
+-------+-------+-------+-------+-------+-------+-------+-------+
| active|  score|
+-------+-------+
  0       4       8      12      16      20      24
```

### Example 4: Reference Types

```dol
gene Node {
    has value: Int64
    has next: &Node          // Reference (pointer)
    has prev: &Node          // Reference (pointer)
}
```

Memory Layout (16 bytes, 8-byte aligned):
```text
Offset  Size  Field
------  ----  -----
0       8     value (i64)
8       4     next (i32 pointer)
12      4     prev (i32 pointer)
------
Total: 16 bytes
```

---

## Rust Struct Definitions

```rust
use wasm_encoder::ValType;

/// Describes the memory layout of a gene (struct-like type).
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
    /// Get a field by name
    pub fn get_field(&self, name: &str) -> Option<&FieldLayout> {
        self.fields.iter().find(|f| f.name == name)
    }

    /// Get the WASM instructions to load this gene's type definition
    pub fn type_id(&self) -> u32 {
        // Hash of name for type identification
        // In practice, use a type registry with sequential IDs
        let mut hash = 0u32;
        for byte in self.name.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
        }
        hash
    }
}

/// Describes a single field within a gene layout.
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

/// WASM type information for field access.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WasmFieldType {
    /// 32-bit integer
    I32,
    /// 64-bit integer
    I64,
    /// 32-bit float
    F32,
    /// 64-bit float
    F64,
    /// Pointer (i32 address)
    Ptr,
}

impl WasmFieldType {
    /// Convert to wasm_encoder ValType
    pub fn to_val_type(self) -> ValType {
        match self {
            WasmFieldType::I32 | WasmFieldType::Ptr => ValType::I32,
            WasmFieldType::I64 => ValType::I64,
            WasmFieldType::F32 => ValType::F32,
            WasmFieldType::F64 => ValType::F64,
        }
    }

    /// Get the size in bytes
    pub fn size(self) -> u32 {
        match self {
            WasmFieldType::I32 | WasmFieldType::F32 | WasmFieldType::Ptr => 4,
            WasmFieldType::I64 | WasmFieldType::F64 => 8,
        }
    }

    /// Get the alignment requirement
    pub fn alignment(self) -> u32 {
        self.size()  // For WASM, alignment == size for primitive types
    }
}

/// Registry for looking up gene layouts by name.
#[derive(Debug, Default)]
pub struct GeneLayoutRegistry {
    layouts: std::collections::HashMap<String, GeneLayout>,
}

impl GeneLayoutRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, layout: GeneLayout) {
        self.layouts.insert(layout.name.clone(), layout);
    }

    pub fn get(&self, name: &str) -> Option<&GeneLayout> {
        self.layouts.get(name)
    }

    /// Compute and register layout for a gene
    pub fn compute_layout(&mut self, gene: &crate::ast::Gene) -> Result<&GeneLayout, String> {
        let layout = compute_gene_layout_impl(gene, self)?;
        self.layouts.insert(layout.name.clone(), layout);
        self.layouts.get(&gene.name).ok_or_else(|| "Layout not found".to_string())
    }
}

/// Compute layout implementation (forward reference for nested types)
fn compute_gene_layout_impl(
    gene: &crate::ast::Gene,
    registry: &GeneLayoutRegistry,
) -> Result<GeneLayout, String> {
    use crate::ast::{Statement, TypeExpr};

    let mut fields = Vec::new();
    let mut current_offset = 0u32;
    let mut max_alignment = 1u32;

    for stmt in &gene.statements {
        if let Statement::HasField(field_box) = stmt {
            let field = field_box.as_ref();
            let (wasm_type, size, alignment, is_ref, nested) =
                type_to_wasm_info(&field.type_, registry)?;

            // Align to field requirement
            current_offset = align_up(current_offset, alignment);

            fields.push(FieldLayout {
                name: field.name.clone(),
                offset: current_offset,
                size,
                alignment,
                wasm_type,
                is_reference: is_ref,
                nested_layout: nested,
            });

            current_offset += size;
            max_alignment = max_alignment.max(alignment);
        }
    }

    // Pad to struct alignment
    let total_size = align_up(current_offset, max_alignment);

    Ok(GeneLayout {
        name: gene.name.clone(),
        fields,
        total_size,
        alignment: max_alignment,
    })
}

fn type_to_wasm_info(
    type_expr: &crate::ast::TypeExpr,
    registry: &GeneLayoutRegistry,
) -> Result<(WasmFieldType, u32, u32, bool, Option<Box<GeneLayout>>), String> {
    use crate::ast::TypeExpr;

    match type_expr {
        TypeExpr::Named(name) => match name.as_str() {
            "Int32" | "i32" | "Bool" | "Char" =>
                Ok((WasmFieldType::I32, 4, 4, false, None)),
            "Int64" | "i64" =>
                Ok((WasmFieldType::I64, 8, 8, false, None)),
            "Float32" | "f32" =>
                Ok((WasmFieldType::F32, 4, 4, false, None)),
            "Float64" | "f64" =>
                Ok((WasmFieldType::F64, 8, 8, false, None)),
            gene_name => {
                // Look up as gene type
                if let Some(layout) = registry.get(gene_name) {
                    // Inline embedding
                    Ok((WasmFieldType::I32, layout.total_size, layout.alignment,
                        false, Some(Box::new(layout.clone()))))
                } else {
                    Err(format!("Unknown type: {}", gene_name))
                }
            }
        },
        TypeExpr::Generic { name, args: _ } => {
            // Handle reference types like &Point
            if name.starts_with('&') {
                Ok((WasmFieldType::Ptr, 4, 4, true, None))
            } else {
                Err(format!("Generic types not yet supported: {}", name))
            }
        },
        _ => Err("Complex types not yet supported".to_string()),
    }
}

fn align_up(offset: u32, alignment: u32) -> u32 {
    (offset + alignment - 1) & !(alignment - 1)
}
```

---

## Memory Allocator Design

### Bump Allocator

For initial implementation, we use a simple bump allocator. This is fast but does not support freeing memory.

```rust
/// Bump allocator state stored in WASM globals.
///
/// Memory layout:
/// - Global 0: HEAP_BASE (next free address)
/// - Global 1: HEAP_END (end of available memory)
pub struct BumpAllocator {
    heap_base_global: u32,  // Index of HEAP_BASE global
    heap_end_global: u32,   // Index of HEAP_END global
}

impl BumpAllocator {
    /// Generate WASM globals for allocator state
    pub fn emit_globals(module: &mut wasm_encoder::Module, initial_heap: u32) {
        use wasm_encoder::{GlobalSection, GlobalType, ValType, ConstExpr};

        let mut globals = GlobalSection::new();

        // HEAP_BASE: mutable i32, starts after static data
        globals.global(
            GlobalType { val_type: ValType::I32, mutable: true, shared: false },
            &ConstExpr::i32_const(initial_heap as i32),
        );

        // HEAP_END: immutable i32, end of first memory page (64KB)
        globals.global(
            GlobalType { val_type: ValType::I32, mutable: false, shared: false },
            &ConstExpr::i32_const(65536),  // 1 page = 64KB
        );

        module.section(&globals);
    }

    /// Generate the alloc function
    ///
    /// Function signature: alloc(size: i32, align: i32) -> i32
    /// Returns pointer to allocated memory, or 0 on failure
    pub fn emit_alloc_function() -> Vec<wasm_encoder::Instruction<'static>> {
        use wasm_encoder::Instruction;

        vec![
            // Load current heap pointer
            Instruction::GlobalGet(0),  // heap_base

            // Align: ptr = (ptr + align - 1) & ~(align - 1)
            Instruction::LocalGet(1),   // align
            Instruction::I32Add,
            Instruction::I32Const(1),
            Instruction::I32Sub,
            Instruction::LocalGet(1),   // align
            Instruction::I32Const(1),
            Instruction::I32Sub,
            Instruction::I32Const(-1),
            Instruction::I32Xor,
            Instruction::I32And,

            // Save aligned pointer to local
            Instruction::LocalTee(2),   // result_ptr (new local)

            // Calculate new heap pointer: aligned_ptr + size
            Instruction::LocalGet(0),   // size
            Instruction::I32Add,

            // Check if we exceeded heap end
            Instruction::LocalTee(3),   // new_heap_base
            Instruction::GlobalGet(1),  // heap_end
            Instruction::I32GtU,

            // If exceeded, return 0 (null)
            Instruction::If(wasm_encoder::BlockType::Empty),
            Instruction::I32Const(0),
            Instruction::Return,
            Instruction::End,

            // Update heap base
            Instruction::LocalGet(3),   // new_heap_base
            Instruction::GlobalSet(0),  // heap_base = new_heap_base

            // Return allocated pointer
            Instruction::LocalGet(2),   // result_ptr
            Instruction::End,
        ]
    }
}
```

### Memory Section Declaration

```rust
/// Emit WASM memory section with initial size
pub fn emit_memory_section(module: &mut wasm_encoder::Module, initial_pages: u32) {
    use wasm_encoder::{MemorySection, MemoryType};

    let mut memories = MemorySection::new();
    memories.memory(MemoryType {
        minimum: initial_pages as u64,
        maximum: Some(256),  // 16MB max
        memory64: false,
        shared: false,
        page_size_log2: None,
    });
    module.section(&memories);
}
```

---

## Field Access Patterns

### Reading a Field

```rust
/// Generate WASM instructions to read a field from a gene instance.
///
/// Stack before: [base_ptr: i32]
/// Stack after:  [field_value: field_type]
pub fn emit_field_read(
    function: &mut wasm_encoder::Function,
    field: &FieldLayout,
) {
    use wasm_encoder::Instruction;
    use wasm_encoder::MemArg;

    // Add field offset to base pointer
    if field.offset > 0 {
        function.instruction(&Instruction::I32Const(field.offset as i32));
        function.instruction(&Instruction::I32Add);
    }

    // Load based on type
    let memarg = MemArg {
        offset: 0,
        align: field.alignment.trailing_zeros(),
        memory_index: 0,
    };

    match field.wasm_type {
        WasmFieldType::I32 | WasmFieldType::Ptr => {
            function.instruction(&Instruction::I32Load(memarg));
        }
        WasmFieldType::I64 => {
            function.instruction(&Instruction::I64Load(memarg));
        }
        WasmFieldType::F32 => {
            function.instruction(&Instruction::F32Load(memarg));
        }
        WasmFieldType::F64 => {
            function.instruction(&Instruction::F64Load(memarg));
        }
    }
}
```

### Writing a Field

```rust
/// Generate WASM instructions to write a field in a gene instance.
///
/// Stack before: [base_ptr: i32, value: field_type]
/// Stack after:  []
pub fn emit_field_write(
    function: &mut wasm_encoder::Function,
    field: &FieldLayout,
) {
    use wasm_encoder::Instruction;
    use wasm_encoder::MemArg;

    // We need to compute address and keep value on stack
    // Stack: [base_ptr, value]

    // Save value to a temp local, compute address, restore value
    // For simplicity, assume we have a temp local available

    let memarg = MemArg {
        offset: field.offset as u64,
        align: field.alignment.trailing_zeros(),
        memory_index: 0,
    };

    match field.wasm_type {
        WasmFieldType::I32 | WasmFieldType::Ptr => {
            function.instruction(&Instruction::I32Store(memarg));
        }
        WasmFieldType::I64 => {
            function.instruction(&Instruction::I64Store(memarg));
        }
        WasmFieldType::F32 => {
            function.instruction(&Instruction::F32Store(memarg));
        }
        WasmFieldType::F64 => {
            function.instruction(&Instruction::F64Store(memarg));
        }
    }
}
```

### Access Pattern Example

For `rectangle.top_left.x`:

```wat
;; Assuming rectangle pointer in local 0
local.get 0           ;; base_ptr for rectangle
i32.const 0           ;; offset of top_left (inline, so 0)
i32.add               ;; ptr to top_left
i32.const 0           ;; offset of x within Point
i32.add               ;; ptr to x
f64.load align=3      ;; load x as f64 (align=3 means 8-byte aligned)
```

---

## Constructor Emission

### Gene Constructor Function

For each gene, generate a constructor function that:
1. Allocates memory for the gene
2. Initializes all fields (with defaults if provided)
3. Returns the pointer

```rust
/// Generate a constructor function for a gene.
///
/// Generated function signature: new_GeneName(field1, field2, ...) -> i32
pub fn emit_gene_constructor(
    layout: &GeneLayout,
    alloc_func_idx: u32,
) -> wasm_encoder::Function {
    use wasm_encoder::{Function, Instruction, ValType, MemArg};

    // Locals: one for the allocated pointer
    let locals = vec![(1, ValType::I32)];  // ptr local
    let mut function = Function::new(locals);

    // Call alloc(size, alignment)
    function.instruction(&Instruction::I32Const(layout.total_size as i32));
    function.instruction(&Instruction::I32Const(layout.alignment as i32));
    function.instruction(&Instruction::Call(alloc_func_idx));

    // Store result in local
    let ptr_local = layout.fields.len() as u32;  // After all params
    function.instruction(&Instruction::LocalTee(ptr_local));

    // Check for allocation failure (null pointer)
    function.instruction(&Instruction::I32Eqz);
    function.instruction(&Instruction::If(wasm_encoder::BlockType::Empty));
    function.instruction(&Instruction::Unreachable);  // Or return null
    function.instruction(&Instruction::End);

    // Initialize each field from parameters
    for (idx, field) in layout.fields.iter().enumerate() {
        // Load base pointer
        function.instruction(&Instruction::LocalGet(ptr_local));

        // Load field value from parameter
        function.instruction(&Instruction::LocalGet(idx as u32));

        // Store at offset
        let memarg = MemArg {
            offset: field.offset as u64,
            align: field.alignment.trailing_zeros(),
            memory_index: 0,
        };

        match field.wasm_type {
            WasmFieldType::I32 | WasmFieldType::Ptr => {
                function.instruction(&Instruction::I32Store(memarg));
            }
            WasmFieldType::I64 => {
                function.instruction(&Instruction::I64Store(memarg));
            }
            WasmFieldType::F32 => {
                function.instruction(&Instruction::F32Store(memarg));
            }
            WasmFieldType::F64 => {
                function.instruction(&Instruction::F64Store(memarg));
            }
        }
    }

    // Return the pointer
    function.instruction(&Instruction::LocalGet(ptr_local));
    function.instruction(&Instruction::End);

    function
}
```

### Constructor Usage Example

DOL:
```dol
gene Point {
    has x: Float64
    has y: Float64
}

fun create_origin() -> &Point {
    return Point { x: 0.0, y: 0.0 }
}
```

Generated WAT:
```wat
;; Constructor: new_Point(x: f64, y: f64) -> i32
(func $new_Point (param $x f64) (param $y f64) (result i32)
  (local $ptr i32)

  ;; Allocate 16 bytes, 8-byte aligned
  (local.set $ptr (call $alloc (i32.const 16) (i32.const 8)))

  ;; Check allocation success
  (if (i32.eqz (local.get $ptr))
    (then (unreachable)))

  ;; Store x at offset 0
  (f64.store offset=0 align=3 (local.get $ptr) (local.get $x))

  ;; Store y at offset 8
  (f64.store offset=8 align=3 (local.get $ptr) (local.get $y))

  ;; Return pointer
  (local.get $ptr)
)

;; create_origin function
(func $create_origin (result i32)
  (call $new_Point (f64.const 0.0) (f64.const 0.0))
)
```

---

## Nested Gene Handling

### Inline Embedding

When a gene contains another gene by value (inline), the inner gene's fields are embedded directly:

```dol
gene Vector2D {
    has x: Float64
    has y: Float64
}

gene Transform {
    has position: Vector2D   // Inline
    has rotation: Float64
}
```

Layout for Transform (24 bytes):
```text
Offset  Field
------  -----
0       position.x (f64)
8       position.y (f64)
16      rotation (f64)
```

### Reference Embedding

When a gene contains a reference to another gene:

```dol
gene TreeNode {
    has value: Int64
    has left: &TreeNode
    has right: &TreeNode
}
```

Layout for TreeNode (16 bytes):
```text
Offset  Field
------  -----
0       value (i64)
8       left (i32 pointer)
12      right (i32 pointer)
```

### Nested Field Access

For `transform.position.x`:

```rust
/// Emit access to nested field
pub fn emit_nested_field_read(
    function: &mut wasm_encoder::Function,
    base_layout: &GeneLayout,
    field_path: &[&str],  // e.g., ["position", "x"]
) -> Result<(), String> {
    use wasm_encoder::Instruction;

    let mut current_offset = 0u32;
    let mut current_layout = base_layout;
    let mut final_field = None;

    for (i, field_name) in field_path.iter().enumerate() {
        let field = current_layout
            .get_field(field_name)
            .ok_or_else(|| format!("Field {} not found", field_name))?;

        current_offset += field.offset;

        if i == field_path.len() - 1 {
            // Last field - this is what we're reading
            final_field = Some(field.clone());
        } else if let Some(ref nested) = field.nested_layout {
            // Navigate into nested struct
            current_layout = nested;
        } else if field.is_reference {
            // Dereference pointer, then continue
            // Emit: load pointer, add remaining offset
            if current_offset > 0 {
                function.instruction(&Instruction::I32Const(current_offset as i32));
                function.instruction(&Instruction::I32Add);
            }
            function.instruction(&Instruction::I32Load(wasm_encoder::MemArg {
                offset: 0,
                align: 2,
                memory_index: 0,
            }));
            current_offset = 0;
            // Would need to look up the pointed-to layout...
            return Err("Reference field traversal not yet implemented".to_string());
        }
    }

    // Emit final load
    if let Some(field) = final_field {
        if current_offset > 0 {
            function.instruction(&Instruction::I32Const(current_offset as i32));
            function.instruction(&Instruction::I32Add);
        }

        let memarg = wasm_encoder::MemArg {
            offset: 0,
            align: field.alignment.trailing_zeros(),
            memory_index: 0,
        };

        match field.wasm_type {
            WasmFieldType::I32 | WasmFieldType::Ptr => {
                function.instruction(&Instruction::I32Load(memarg));
            }
            WasmFieldType::I64 => {
                function.instruction(&Instruction::I64Load(memarg));
            }
            WasmFieldType::F32 => {
                function.instruction(&Instruction::F32Load(memarg));
            }
            WasmFieldType::F64 => {
                function.instruction(&Instruction::F64Load(memarg));
            }
        }
    }

    Ok(())
}
```

---

## GC Considerations

### Future GC Integration

For initial implementation, we use a bump allocator with no GC. Future work should consider:

1. **Reference Counting**: Simple but has cycle issues
2. **Tracing GC**: Mark-and-sweep or copying collector
3. **WASM GC Proposal**: Use native WASM GC types when available

### GC Root Tracking

For tracing GC, we need to identify root pointers:
- Global variables holding gene references
- Local variables on the stack
- Function parameters that are gene references

### Type Information for GC

Each gene needs a type descriptor for GC traversal:

```rust
/// Type descriptor for GC traversal
pub struct GeneTypeDescriptor {
    /// Total size of the gene in bytes
    pub size: u32,

    /// Offsets of pointer fields (for GC traversal)
    pub pointer_offsets: Vec<u32>,

    /// Whether this type contains any references
    pub has_references: bool,
}

impl GeneLayout {
    pub fn to_gc_descriptor(&self) -> GeneTypeDescriptor {
        let pointer_offsets: Vec<u32> = self
            .fields
            .iter()
            .filter(|f| f.is_reference)
            .map(|f| f.offset)
            .collect();

        GeneTypeDescriptor {
            size: self.total_size,
            has_references: !pointer_offsets.is_empty(),
            pointer_offsets,
        }
    }
}
```

### Stack Maps (Future)

For precise GC, generate stack maps at safepoints:

```text
Safepoint at offset 0x1234:
  Local 0: not a pointer
  Local 1: pointer to Gene
  Local 2: not a pointer
  Stack[0]: pointer to Gene
```

---

## Example Compilation

### Full Example

DOL Source:
```dol
gene Point {
    has x: Float64
    has y: Float64
}

gene Rectangle {
    has top_left: Point
    has width: Float64
    has height: Float64
}

fun area(rect: &Rectangle) -> Float64 {
    return rect.width * rect.height
}

fun move_rect(rect: &Rectangle, dx: Float64, dy: Float64) {
    rect.top_left.x = rect.top_left.x + dx
    rect.top_left.y = rect.top_left.y + dy
}
```

Generated WASM (WAT format):
```wat
(module
  ;; Memory: 1 page (64KB)
  (memory 1)

  ;; Globals for bump allocator
  (global $heap_base (mut i32) (i32.const 1024))  ;; Start after static data
  (global $heap_end i32 (i32.const 65536))

  ;; Allocator function
  (func $alloc (param $size i32) (param $align i32) (result i32)
    (local $ptr i32)
    (local $new_base i32)

    ;; Align heap_base
    (local.set $ptr
      (i32.and
        (i32.add (global.get $heap_base) (i32.sub (local.get $align) (i32.const 1)))
        (i32.xor (i32.sub (local.get $align) (i32.const 1)) (i32.const -1))))

    ;; Calculate new heap base
    (local.set $new_base (i32.add (local.get $ptr) (local.get $size)))

    ;; Check bounds
    (if (i32.gt_u (local.get $new_base) (global.get $heap_end))
      (then (return (i32.const 0))))

    ;; Update heap base and return pointer
    (global.set $heap_base (local.get $new_base))
    (local.get $ptr)
  )

  ;; Point constructor: new_Point(x: f64, y: f64) -> i32
  (func $new_Point (param $x f64) (param $y f64) (result i32)
    (local $ptr i32)

    (local.set $ptr (call $alloc (i32.const 16) (i32.const 8)))
    (if (i32.eqz (local.get $ptr)) (then (unreachable)))

    (f64.store offset=0 (local.get $ptr) (local.get $x))
    (f64.store offset=8 (local.get $ptr) (local.get $y))

    (local.get $ptr)
  )

  ;; Rectangle constructor: new_Rectangle(top_left_x, top_left_y, width, height) -> i32
  (func $new_Rectangle (param $tlx f64) (param $tly f64) (param $w f64) (param $h f64) (result i32)
    (local $ptr i32)

    (local.set $ptr (call $alloc (i32.const 32) (i32.const 8)))
    (if (i32.eqz (local.get $ptr)) (then (unreachable)))

    (f64.store offset=0 (local.get $ptr) (local.get $tlx))   ;; top_left.x
    (f64.store offset=8 (local.get $ptr) (local.get $tly))   ;; top_left.y
    (f64.store offset=16 (local.get $ptr) (local.get $w))    ;; width
    (f64.store offset=24 (local.get $ptr) (local.get $h))    ;; height

    (local.get $ptr)
  )

  ;; area(rect: &Rectangle) -> f64
  (func $area (export "area") (param $rect i32) (result f64)
    (f64.mul
      (f64.load offset=16 (local.get $rect))  ;; width
      (f64.load offset=24 (local.get $rect))) ;; height
  )

  ;; move_rect(rect: &Rectangle, dx: f64, dy: f64)
  (func $move_rect (export "move_rect") (param $rect i32) (param $dx f64) (param $dy f64)
    ;; rect.top_left.x += dx
    (f64.store offset=0 (local.get $rect)
      (f64.add
        (f64.load offset=0 (local.get $rect))
        (local.get $dx)))

    ;; rect.top_left.y += dy
    (f64.store offset=8 (local.get $rect)
      (f64.add
        (f64.load offset=8 (local.get $rect))
        (local.get $dy)))
  )

  ;; Export constructors
  (export "new_Point" (func $new_Point))
  (export "new_Rectangle" (func $new_Rectangle))
  (export "alloc" (func $alloc))
)
```

---

## Summary

This design provides:

1. **Predictable Memory Layout**: C-compatible struct layout with proper alignment
2. **Efficient Access**: Direct load/store with computed offsets
3. **Simple Allocation**: Bump allocator for initial implementation
4. **Nested Support**: Both inline and reference embedding of genes
5. **Extensibility**: Clear path to GC integration in the future

### Implementation Priority

1. **Phase 1**: GeneLayout and FieldLayout structs
2. **Phase 2**: Layout computation from AST
3. **Phase 3**: Bump allocator emission
4. **Phase 4**: Constructor generation
5. **Phase 5**: Field access code generation
6. **Phase 6**: Nested gene support
7. **Future**: GC integration

### Files to Create/Modify

- `src/wasm/layout.rs` - GeneLayout, FieldLayout, registry
- `src/wasm/alloc.rs` - Bump allocator code generation
- `src/wasm/codegen.rs` - Constructor and field access emission
- `src/wasm/compiler.rs` - Integrate layout computation into compilation
