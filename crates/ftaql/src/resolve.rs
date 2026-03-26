use dashmap::DashMap;
use rspack_resolver::{ResolveOptions, Resolver, TsconfigOptions, TsconfigReferences};
use std::default::Default;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct ResolverCache {
    cache: DashMap<PathBuf, Arc<Resolver>>,
}

impl ResolverCache {
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
        }
    }

    pub fn get_or_create(&self, file_path: &Path) -> Arc<Resolver> {
        let config_path_option = find_tsconfig_path(file_path);
        let cache_key = config_path_option.clone().unwrap_or_default();

        if let Some(resolver) = self.cache.get(&cache_key) {
            return Arc::clone(resolver.value());
        }

        log::debug!("Creating resolver for config: {:?}", &cache_key);

        let resolve_options = if let Some(config_file) = config_path_option {
            ResolveOptions {
                extensions: vec![
                    ".ts".to_string(),
                    ".tsx".to_string(),
                    ".js".to_string(),
                    ".jsx".to_string(),
                ],
                condition_names: vec!["node".to_string(), "import".to_string()],
                tsconfig: Some(TsconfigOptions {
                    config_file,
                    references: TsconfigReferences::Auto,
                }),
                ..Default::default()
            }
        } else {
            ResolveOptions {
                extensions: vec![
                    ".ts".to_string(),
                    ".tsx".to_string(),
                    ".js".to_string(),
                    ".jsx".to_string(),
                ],
                condition_names: vec!["node".to_string(), "import".to_string()],
                ..Default::default()
            }
        };

        let resolver = Arc::new(Resolver::new(resolve_options));
        self.cache.insert(cache_key, Arc::clone(&resolver));
        resolver
    }
}

fn find_tsconfig_path(start_path: &Path) -> Option<PathBuf> {
    let mut current_dir = start_path
        .is_dir()
        .then_some(start_path)
        .unwrap_or_else(|| start_path.parent().expect("Failed to get parent directory"));

    loop {
        let tsconfig_path = current_dir.join("tsconfig.json");
        if tsconfig_path.exists() {
            return Some(tsconfig_path);
        }

        let jsconfig_path = current_dir.join("jsconfig.json");
        if jsconfig_path.exists() {
            return Some(jsconfig_path);
        }

        if let Some(parent) = current_dir.parent() {
            current_dir = parent;
        } else {
            return None;
        }
    }
}
