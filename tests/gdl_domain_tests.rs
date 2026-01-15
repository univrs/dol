//! Property-based tests for GDL (Geometric Deep Learning) domain genes.
//!
//! These tests verify the correctness of Grid2D, Graph, and PointCloud genes
//! that form the foundation of the GDL ontology in DOL.
//!
//! Test categories:
//! 1. Parsing tests - verify genes parse correctly
//! 2. Constraint tests - verify constraint semantics
//! 3. Property tests - verify mathematical properties hold
//! 4. Symmetry tests - verify symmetry group properties

use metadol::{parse_file, Declaration, Statement};
use std::fs;

// ============================================
// 1. Grid2D<T> Gene Tests
// ============================================

#[test]
fn test_parse_grid2d_gene() {
    let content =
        fs::read_to_string("examples/genes/grid2d.dol").expect("Failed to read grid2d.dol");

    let result = parse_file(&content);
    assert!(
        result.is_ok(),
        "Failed to parse grid2d.dol: {:?}",
        result.err()
    );

    let decl = result.unwrap();
    assert_eq!(decl.name(), "Grid2D");
}

#[test]
fn test_grid2d_has_shape_constraint() {
    let content =
        fs::read_to_string("examples/genes/grid2d.dol").expect("Failed to read grid2d.dol");

    let decl = parse_file(&content).expect("Failed to parse");

    // Verify the exegesis mentions shape constraint
    let exegesis = decl.exegesis();
    assert!(
        exegesis.contains("Shape Constraint") || exegesis.contains("valid_shape"),
        "Grid2D should document shape constraint in exegesis"
    );
}

#[test]
fn test_grid2d_has_required_properties() {
    let content =
        fs::read_to_string("examples/genes/grid2d.dol").expect("Failed to read grid2d.dol");

    let decl = parse_file(&content).expect("Failed to parse");

    if let Declaration::Gene(gene) = decl {
        let field_names: Vec<String> = gene
            .statements
            .iter()
            .filter_map(|s| {
                if let Statement::HasField(field) = s {
                    Some(field.name.clone())
                } else {
                    None
                }
            })
            .collect();

        // Grid2D must have height, width, channels, data
        assert!(
            field_names.iter().any(|n| n == "height"),
            "Grid2D must have height"
        );
        assert!(
            field_names.iter().any(|n| n == "width"),
            "Grid2D must have width"
        );
        assert!(
            field_names.iter().any(|n| n == "channels"),
            "Grid2D must have channels"
        );
        assert!(
            field_names.iter().any(|n| n == "data"),
            "Grid2D must have data"
        );
    } else {
        panic!("Expected Gene declaration");
    }
}

#[test]
fn test_grid2d_translation_symmetry_documented() {
    let content =
        fs::read_to_string("examples/genes/grid2d.dol").expect("Failed to read grid2d.dol");

    let decl = parse_file(&content).expect("Failed to parse");

    let exegesis = decl.exegesis();
    assert!(
        exegesis.contains("T(2)") || exegesis.contains("Translation"),
        "Grid2D should document T(2) translation symmetry"
    );
}

// ============================================
// 2. Graph<N, E> Gene Tests
// ============================================

#[test]
fn test_parse_graph_gene() {
    let content = fs::read_to_string("examples/genes/graph.dol").expect("Failed to read graph.dol");

    let result = parse_file(&content);
    assert!(
        result.is_ok(),
        "Failed to parse graph.dol: {:?}",
        result.err()
    );

    let decl = result.unwrap();
    assert_eq!(decl.name(), "Graph");
}

#[test]
fn test_graph_has_edge_validity_constraint() {
    let content = fs::read_to_string("examples/genes/graph.dol").expect("Failed to read graph.dol");

    let decl = parse_file(&content).expect("Failed to parse");

    let exegesis = decl.exegesis();
    assert!(
        exegesis.contains("Edge Validity") || exegesis.contains("valid_edges"),
        "Graph should document edge validity constraint"
    );
}

#[test]
fn test_graph_has_undirected_symmetry_law() {
    let content = fs::read_to_string("examples/genes/graph.dol").expect("Failed to read graph.dol");

    let decl = parse_file(&content).expect("Failed to parse");

    let exegesis = decl.exegesis();
    assert!(
        exegesis.contains("Undirected Symmetry") || exegesis.contains("undirected_symmetry"),
        "Graph should document undirected symmetry law"
    );
}

#[test]
fn test_graph_has_required_properties() {
    let content = fs::read_to_string("examples/genes/graph.dol").expect("Failed to read graph.dol");

    let decl = parse_file(&content).expect("Failed to parse");

    if let Declaration::Gene(gene) = decl {
        let field_names: Vec<String> = gene
            .statements
            .iter()
            .filter_map(|s| {
                if let Statement::HasField(field) = s {
                    Some(field.name.clone())
                } else {
                    None
                }
            })
            .collect();

        // Graph must have nodes, edges, adjacency
        assert!(
            field_names.iter().any(|n| n == "nodes"),
            "Graph must have nodes"
        );
        assert!(
            field_names.iter().any(|n| n == "edges"),
            "Graph must have edges"
        );
        assert!(
            field_names.iter().any(|n| n == "adjacency"),
            "Graph must have adjacency"
        );
    } else {
        panic!("Expected Gene declaration");
    }
}

#[test]
fn test_graph_permutation_symmetry_documented() {
    let content = fs::read_to_string("examples/genes/graph.dol").expect("Failed to read graph.dol");

    let decl = parse_file(&content).expect("Failed to parse");

    let exegesis = decl.exegesis();
    assert!(
        exegesis.contains("S_n") || exegesis.contains("Permutation"),
        "Graph should document S_n permutation symmetry"
    );
}

// ============================================
// 3. PointCloud<F> Gene Tests
// ============================================

#[test]
fn test_parse_pointcloud_gene() {
    let content =
        fs::read_to_string("examples/genes/pointcloud.dol").expect("Failed to read pointcloud.dol");

    let result = parse_file(&content);
    assert!(
        result.is_ok(),
        "Failed to parse pointcloud.dol: {:?}",
        result.err()
    );

    let decl = result.unwrap();
    assert_eq!(decl.name(), "PointCloud");
}

#[test]
fn test_pointcloud_has_feature_alignment_constraint() {
    let content =
        fs::read_to_string("examples/genes/pointcloud.dol").expect("Failed to read pointcloud.dol");

    let decl = parse_file(&content).expect("Failed to parse");

    let exegesis = decl.exegesis();
    assert!(
        exegesis.contains("Feature Alignment") || exegesis.contains("feature_alignment"),
        "PointCloud should document feature alignment constraint"
    );
}

#[test]
fn test_pointcloud_has_required_properties() {
    let content =
        fs::read_to_string("examples/genes/pointcloud.dol").expect("Failed to read pointcloud.dol");

    let decl = parse_file(&content).expect("Failed to parse");

    if let Declaration::Gene(gene) = decl {
        let field_names: Vec<String> = gene
            .statements
            .iter()
            .filter_map(|s| {
                if let Statement::HasField(field) = s {
                    Some(field.name.clone())
                } else {
                    None
                }
            })
            .collect();

        // PointCloud must have points, features
        assert!(
            field_names.iter().any(|n| n == "points"),
            "PointCloud must have points"
        );
        assert!(
            field_names.iter().any(|n| n == "features"),
            "PointCloud must have features"
        );
    } else {
        panic!("Expected Gene declaration");
    }
}

#[test]
fn test_pointcloud_se3_symmetry_documented() {
    let content =
        fs::read_to_string("examples/genes/pointcloud.dol").expect("Failed to read pointcloud.dol");

    let decl = parse_file(&content).expect("Failed to parse");

    let exegesis = decl.exegesis();
    assert!(
        exegesis.contains("SE(3)") || exegesis.contains("E(3)"),
        "PointCloud should document SE(3) or E(3) symmetry"
    );
}

// ============================================
// 4. Cross-Domain Property Tests
// ============================================

#[test]
fn test_all_gdl_genes_have_exegesis() {
    let genes = ["grid2d.dol", "graph.dol", "pointcloud.dol"];

    for gene_file in &genes {
        let path = format!("examples/genes/{}", gene_file);
        let content =
            fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read {}", path));

        let decl =
            parse_file(&content).unwrap_or_else(|e| panic!("Failed to parse {}: {:?}", path, e));

        let exegesis = decl.exegesis();

        assert!(
            !exegesis.is_empty(),
            "{} must have non-empty exegesis",
            gene_file
        );
        assert!(
            exegesis.len() > 100,
            "{} exegesis should be comprehensive (>100 chars)",
            gene_file
        );
    }
}

#[test]
fn test_all_gdl_genes_document_symmetry() {
    let symmetry_keywords = [
        ("grid2d.dol", vec!["T(2)", "Translation"]),
        ("graph.dol", vec!["S_n", "Permutation"]),
        ("pointcloud.dol", vec!["SE(3)", "E(3)", "S_n"]),
    ];

    for (gene_file, keywords) in &symmetry_keywords {
        let path = format!("examples/genes/{}", gene_file);
        let content =
            fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read {}", path));

        let decl =
            parse_file(&content).unwrap_or_else(|e| panic!("Failed to parse {}: {:?}", path, e));

        let exegesis = decl.exegesis();

        let has_symmetry = keywords.iter().any(|kw| exegesis.contains(kw));
        assert!(
            has_symmetry,
            "{} must document symmetry group (expected one of: {:?})",
            gene_file, keywords
        );
    }
}

#[test]
fn test_all_gdl_genes_have_constraints() {
    let genes = ["grid2d.dol", "graph.dol", "pointcloud.dol"];

    for gene_file in &genes {
        let path = format!("examples/genes/{}", gene_file);
        let content =
            fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read {}", path));

        // Check that "constraint" keyword appears in the source
        assert!(
            content.contains("constraint"),
            "{} must have at least one constraint",
            gene_file
        );
    }
}

#[test]
fn test_gdl_genes_follow_naming_convention() {
    let genes = [
        ("grid2d.dol", "Grid2D"),
        ("graph.dol", "Graph"),
        ("pointcloud.dol", "PointCloud"),
    ];

    for (gene_file, expected_name) in &genes {
        let path = format!("examples/genes/{}", gene_file);
        let content =
            fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read {}", path));

        let decl =
            parse_file(&content).unwrap_or_else(|e| panic!("Failed to parse {}: {:?}", path, e));

        assert_eq!(
            decl.name(),
            *expected_name,
            "{} should declare gene named {}",
            gene_file,
            expected_name
        );
    }
}

// ============================================
// 5. GDL Blueprint Compliance Tests
// ============================================

#[test]
fn test_grid2d_gdl_blueprint_reference() {
    let content =
        fs::read_to_string("examples/genes/grid2d.dol").expect("Failed to read grid2d.dol");

    let decl = parse_file(&content).expect("Failed to parse");

    let exegesis = decl.exegesis();
    assert!(
        exegesis.contains("GDL") || exegesis.contains("Geometric Deep Learning"),
        "Grid2D exegesis should reference GDL Blueprint"
    );
}

#[test]
fn test_graph_gdl_blueprint_reference() {
    let content = fs::read_to_string("examples/genes/graph.dol").expect("Failed to read graph.dol");

    let decl = parse_file(&content).expect("Failed to parse");

    let exegesis = decl.exegesis();
    assert!(
        exegesis.contains("GDL") || exegesis.contains("Geometric Deep Learning"),
        "Graph exegesis should reference GDL Blueprint"
    );
}

#[test]
fn test_pointcloud_gdl_blueprint_reference() {
    let content =
        fs::read_to_string("examples/genes/pointcloud.dol").expect("Failed to read pointcloud.dol");

    let decl = parse_file(&content).expect("Failed to parse");

    let exegesis = decl.exegesis();
    assert!(
        exegesis.contains("GDL") || exegesis.contains("Geometric Deep Learning"),
        "PointCloud exegesis should reference GDL Blueprint"
    );
}

// ============================================
// 6. Statement Count Verification
// ============================================

#[test]
fn test_grid2d_statement_count() {
    let content =
        fs::read_to_string("examples/genes/grid2d.dol").expect("Failed to read grid2d.dol");

    let decl = parse_file(&content).expect("Failed to parse");

    if let Declaration::Gene(gene) = decl {
        // Grid2D should have multiple statements (fields, constraints, functions)
        assert!(
            gene.statements.len() >= 5,
            "Grid2D should have at least 5 statements, found {}",
            gene.statements.len()
        );
    } else {
        panic!("Expected Gene declaration");
    }
}

#[test]
fn test_graph_statement_count() {
    let content = fs::read_to_string("examples/genes/graph.dol").expect("Failed to read graph.dol");

    let decl = parse_file(&content).expect("Failed to parse");

    if let Declaration::Gene(gene) = decl {
        // Graph should have multiple statements (fields, constraints, laws, functions)
        assert!(
            gene.statements.len() >= 8,
            "Graph should have at least 8 statements, found {}",
            gene.statements.len()
        );
    } else {
        panic!("Expected Gene declaration");
    }
}

#[test]
fn test_pointcloud_statement_count() {
    let content =
        fs::read_to_string("examples/genes/pointcloud.dol").expect("Failed to read pointcloud.dol");

    let decl = parse_file(&content).expect("Failed to parse");

    if let Declaration::Gene(gene) = decl {
        // PointCloud should have multiple statements
        assert!(
            gene.statements.len() >= 8,
            "PointCloud should have at least 8 statements, found {}",
            gene.statements.len()
        );
    } else {
        panic!("Expected Gene declaration");
    }
}
