use anyhow::{Context, Result};
use ghostfs_registry::{NpmRegistryClient, PackageMetadata};
use semver::{Version, VersionReq};
use std::collections::{BTreeMap, HashMap, HashSet};
use tracing::debug;

/// A fully resolved package in the dependency graph.
#[derive(Debug, Clone)]
pub struct ResolvedPackage {
    pub name: String,
    pub version: String,
    pub tarball: String,
    pub shasum: String,
    pub integrity: Option<String>,
    pub dependencies: BTreeMap<String, String>,
}

/// Resolves dependency trees from semver constraints against the npm registry.
pub struct DependencyResolver {
    registry: NpmRegistryClient,
    metadata_cache: HashMap<String, PackageMetadata>,
}

impl DependencyResolver {
    pub fn new(registry: NpmRegistryClient) -> Self {
        Self {
            registry,
            metadata_cache: HashMap::new(),
        }
    }

    /// Resolve all dependencies (and transitive deps) from a name→version_req map.
    pub async fn resolve(
        &mut self,
        dependencies: &BTreeMap<String, String>,
    ) -> Result<Vec<ResolvedPackage>> {
        let mut resolved: BTreeMap<String, ResolvedPackage> = BTreeMap::new();
        let mut queue: Vec<(String, String)> = dependencies
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let mut visited: HashSet<String> = HashSet::new();

        while let Some((name, version_req_str)) = queue.pop() {
            if visited.contains(&name) {
                continue;
            }
            visited.insert(name.clone());

            debug!("Resolving {} @ {}", name, version_req_str);

            let metadata = self.fetch_metadata(&name).await?;
            let version = Self::resolve_best_version(&metadata, &version_req_str)?;

            let version_meta = metadata
                .versions
                .get(&version)
                .with_context(|| format!("Version {} not found for {}", version, name))?;

            let deps: BTreeMap<String, String> = version_meta
                .dependencies
                .clone()
                .unwrap_or_default()
                .into_iter()
                .collect();

            let pkg = ResolvedPackage {
                name: name.clone(),
                version: version.clone(),
                tarball: version_meta.dist.tarball.clone(),
                shasum: version_meta.dist.shasum.clone(),
                integrity: version_meta.dist.integrity.clone(),
                dependencies: deps.clone(),
            };

            resolved.insert(name.clone(), pkg);

            // Enqueue transitive dependencies
            for (dep_name, dep_version) in &deps {
                if !visited.contains(dep_name) {
                    queue.push((dep_name.clone(), dep_version.clone()));
                }
            }
        }

        Ok(resolved.into_values().collect())
    }

    /// Fetch metadata from registry (with in-memory cache).
    async fn fetch_metadata(&mut self, name: &str) -> Result<PackageMetadata> {
        if let Some(cached) = self.metadata_cache.get(name) {
            return Ok(cached.clone());
        }
        let metadata = self.registry.get_package_metadata(name).await?;
        self.metadata_cache
            .insert(name.to_string(), metadata.clone());
        Ok(metadata)
    }

    /// Find the highest version that satisfies a semver constraint.
    fn resolve_best_version(metadata: &PackageMetadata, version_req_str: &str) -> Result<String> {
        // Handle "latest" or empty constraint
        let req_str = version_req_str.trim();
        if req_str.is_empty() || req_str == "latest" || req_str == "*" {
            if let Some(latest) = metadata.dist_tags.get("latest") {
                return Ok(latest.clone());
            }
        }

        // Parse one or more semver requirements, handling npm-specific range format
        let mut parsed_reqs = Vec::new();
        for alt in req_str.split("||") {
            // Convert hyphen ranges: "1.2.3 - 2.3.4" -> ">=1.2.3 <=2.3.4"
            let mut normalized = alt.trim().to_string();
            if let Some((left, right)) = normalized.split_once(" - ") {
                normalized = format!(">={} <={}", left.trim(), right.trim());
            }

            // Remove spaces after operators
            for op in &[">=", "<=", ">", "<", "=", "^", "~"] {
                normalized = normalized.replace(&format!("{} ", op), op);
            }

            // Split into parts by whitespace to clean up and strip 'v' prefix
            let parts: Vec<&str> = normalized.split_whitespace().collect();
            let mut final_parts = Vec::new();
            for part in parts {
                let mut cleaned = part.to_string();
                if cleaned.starts_with('v')
                    && cleaned.chars().nth(1).is_some_and(|c| c.is_ascii_digit())
                {
                    cleaned = cleaned[1..].to_string();
                }
                for op in &[">=", "<=", ">", "<", "=", "^", "~"] {
                    if cleaned.starts_with(op) {
                        let op_len = op.len();
                        if cleaned[op_len..].starts_with('v')
                            && cleaned[op_len + 1..]
                                .chars()
                                .next()
                                .is_some_and(|c| c.is_ascii_digit())
                        {
                            cleaned = format!("{}{}", op, &cleaned[op_len + 1..]);
                        }
                    }
                }
                final_parts.push(cleaned);
            }

            let joined = final_parts.join(",");
            if let Ok(req) = VersionReq::parse(&joined) {
                parsed_reqs.push(req);
            }
        }

        if parsed_reqs.is_empty() {
            // Fall back to direct parse to produce standard parse error
            let _ = VersionReq::parse(req_str).with_context(|| {
                format!(
                    "Invalid version requirement '{}' for {}",
                    req_str, metadata.name
                )
            })?;
        }

        let mut matching_versions: Vec<Version> = metadata
            .versions
            .keys()
            .filter_map(|v| Version::parse(v).ok())
            .filter(|v| parsed_reqs.iter().any(|req| req.matches(v)))
            .collect();

        matching_versions.sort();

        matching_versions
            .last()
            .map(|v| v.to_string())
            .with_context(|| {
                format!(
                    "No version of '{}' matches requirement '{}'",
                    metadata.name, req_str
                )
            })
    }
}
