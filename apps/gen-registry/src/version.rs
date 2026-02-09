//! Version resolution and dependency management

use crate::{
    error::{Error, Result},
    models::{Dependency, GenModule},
};
use semver::{Version, VersionReq};
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::{debug, warn};

/// Version requirement
pub struct VersionRequirement {
    pub module_id: String,
    pub requirement: VersionReq,
}

impl VersionRequirement {
    pub fn parse(module_id: impl Into<String>, req: &str) -> Result<Self> {
        Ok(Self {
            module_id: module_id.into(),
            requirement: VersionReq::parse(req)?,
        })
    }

    pub fn matches(&self, version: &str) -> Result<bool> {
        let v = Version::parse(version)?;
        Ok(self.requirement.matches(&v))
    }
}

/// Resolved dependency
#[derive(Debug, Clone)]
pub struct ResolvedDependency {
    pub module_id: String,
    pub version: String,
}

/// Version resolver
pub struct VersionResolver {
    // Dependency graph for cycle detection
    graph: HashMap<String, HashSet<String>>,
}

impl VersionResolver {
    pub fn new() -> Self {
        Self {
            graph: HashMap::new(),
        }
    }

    /// Resolve dependencies for a module
    pub async fn resolve_dependencies(
        &self,
        module: &GenModule,
        version: &str,
    ) -> Result<Vec<ResolvedDependency>> {
        debug!("Resolving dependencies for {}@{}", module.id, version);

        let mut resolved = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        // Add initial module
        queue.push_back((module.id.clone(), version.to_string()));

        while let Some((current_id, current_version)) = queue.pop_front() {
            if visited.contains(&current_id) {
                continue;
            }
            visited.insert(current_id.clone());

            // Add to resolved
            resolved.push(ResolvedDependency {
                module_id: current_id.clone(),
                version: current_version.clone(),
            });

            // Process dependencies
            for dep in &module.dependencies {
                // Check for cycles
                if self.has_cycle(&current_id, &dep.module_id)? {
                    return Err(Error::DependencyCycle(format!(
                        "{} -> {}",
                        current_id, dep.module_id
                    )));
                }

                // Resolve version
                let dep_version = self.resolve_version(&dep.module_id, &dep.version_requirement)?;

                if !visited.contains(&dep.module_id) {
                    queue.push_back((dep.module_id.clone(), dep_version));
                }
            }
        }

        debug!("Resolved {} dependencies", resolved.len());
        Ok(resolved)
    }

    /// Resolve a specific version from requirement
    fn resolve_version(&self, module_id: &str, requirement: &str) -> Result<String> {
        // Parse requirement
        let req = VersionReq::parse(requirement)?;

        // In real implementation:
        // 1. Fetch all available versions from registry
        // 2. Find latest version matching requirement
        // 3. Return that version

        // For now, extract version from requirement if it's exact
        if requirement.starts_with('=') {
            return Ok(requirement[1..].to_string());
        }

        // Return a placeholder for now
        warn!(
            "Version resolution not fully implemented for {} {}",
            module_id, requirement
        );
        Ok("1.0.0".to_string())
    }

    /// Check for dependency cycles
    fn has_cycle(&self, from: &str, to: &str) -> Result<bool> {
        let mut visited = HashSet::new();
        let mut stack = vec![to];

        while let Some(node) = stack.pop() {
            if node == from {
                return Ok(true);
            }

            if visited.contains(node) {
                continue;
            }
            visited.insert(node);

            if let Some(deps) = self.graph.get(node) {
                stack.extend(deps.iter().map(|s| s.as_str()));
            }
        }

        Ok(false)
    }

    /// Add dependency to graph
    pub fn add_dependency(&mut self, from: impl Into<String>, to: impl Into<String>) {
        self.graph
            .entry(from.into())
            .or_insert_with(HashSet::new)
            .insert(to.into());
    }

    /// Topological sort for install order
    pub fn topological_sort(&self, modules: &[String]) -> Result<Vec<String>> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();

        for module in modules {
            if !visited.contains(module) {
                self.visit_topo(module, &mut visited, &mut stack, &mut result)?;
            }
        }

        result.reverse();
        Ok(result)
    }

    fn visit_topo(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        stack: &mut HashSet<String>,
        result: &mut Vec<String>,
    ) -> Result<()> {
        if stack.contains(node) {
            return Err(Error::DependencyCycle(node.to_string()));
        }

        if visited.contains(node) {
            return Ok(());
        }

        stack.insert(node.to_string());

        if let Some(deps) = self.graph.get(node) {
            for dep in deps {
                self.visit_topo(dep, visited, stack, result)?;
            }
        }

        stack.remove(node);
        visited.insert(node.to_string());
        result.push(node.to_string());

        Ok(())
    }
}

impl Default for VersionResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_requirement() {
        let req = VersionRequirement::parse("io.univrs.user", "^1.0.0").unwrap();
        assert!(req.matches("1.2.3").unwrap());
        assert!(!req.matches("2.0.0").unwrap());
    }

    #[test]
    fn test_cycle_detection() {
        let mut resolver = VersionResolver::new();
        resolver.add_dependency("A", "B");
        resolver.add_dependency("B", "C");
        resolver.add_dependency("C", "A");

        assert!(resolver.has_cycle("A", "B").unwrap());
    }

    #[test]
    fn test_topological_sort() {
        let mut resolver = VersionResolver::new();
        resolver.add_dependency("A", "B");
        resolver.add_dependency("B", "C");

        let sorted = resolver
            .topological_sort(&["A".to_string(), "B".to_string(), "C".to_string()])
            .unwrap();

        // C should come before B, B before A
        assert_eq!(sorted, vec!["C", "B", "A"]);
    }
}
