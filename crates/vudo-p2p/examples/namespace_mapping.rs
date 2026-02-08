//! Example: DOL namespace mapping to Willow namespaces.
//!
//! This example demonstrates how DOL systems and collections are mapped to
//! Willow's 3D namespace structure.

use vudo_p2p::willow_types::{NamespaceId, Path, SubspaceId};

fn main() {
    println!("=== DOL to Willow Namespace Mapping ===\n");

    // DOL System → Willow Namespace
    println!("1. DOL System → Willow Namespace ID\n");

    let dol_system = "myapp.v1";
    let namespace_id = NamespaceId::from_dol_namespace(dol_system);
    println!("   DOL System: {}", dol_system);
    println!("   Willow Namespace ID: {}\n", namespace_id);

    let dol_system2 = "myapp.v2";
    let namespace_id2 = NamespaceId::from_dol_namespace(dol_system2);
    println!("   DOL System: {}", dol_system2);
    println!("   Willow Namespace ID: {}\n", namespace_id2);

    // DOL Collection → Willow Subspace
    println!("2. DOL Collection → Willow Subspace ID\n");

    let collection = "users";
    let subspace_id = SubspaceId::from_dol_collection(collection);
    println!("   DOL Collection: {}", collection);
    println!("   Willow Subspace ID: {}\n", subspace_id);

    let collection2 = "posts";
    let subspace_id2 = SubspaceId::from_dol_collection(collection2);
    println!("   DOL Collection: {}", collection2);
    println!("   Willow Subspace ID: {}\n", subspace_id2);

    // DOL Document ID → Willow Path
    println!("3. DOL Document ID → Willow Path\n");

    let doc_id = "alice";
    let path = Path::from_dol_id(doc_id);
    println!("   DOL Document ID: {}", doc_id);
    println!("   Willow Path: {}\n", path);

    let doc_id_nested = "alice/posts/1";
    let path_nested = Path::from_dol_id(doc_id_nested);
    println!("   DOL Document ID (nested): {}", doc_id_nested);
    println!("   Willow Path: {}", path_nested);
    println!("   Path components: {:?}\n", path_nested.components());

    // Complete 3D Coordinate
    println!("4. Complete Willow 3D Coordinate\n");

    println!("   DOL: myapp.v1 / users / alice/posts/1");
    println!("   Willow 3D:");
    println!("     namespace_id: {}", namespace_id);
    println!("     subspace_id:  {}", subspace_id);
    println!("     path:         {}\n", path_nested);

    // Demonstrate path prefix matching
    println!("5. Path Prefix Matching\n");

    let prefix = Path::from_components(["alice"]);
    let full_path = Path::from_components(["alice", "posts", "1"]);
    let other_path = Path::from_components(["bob", "posts", "1"]);

    println!("   Prefix: {}", prefix);
    println!("   Full path: {}", full_path);
    println!("   Is prefix of full path? {}", prefix.is_prefix_of(&full_path));
    println!("   Other path: {}", other_path);
    println!("   Is prefix of other path? {}", prefix.is_prefix_of(&other_path));

    println!("\n=== Mapping Complete ===");
}
