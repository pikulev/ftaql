use crate::structs::{FtaQlConfigOptional, FtaQlConfigResolved};
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;

mod tests;

#[derive(Debug, Clone)]
pub struct ConfigError {
    message: String,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ConfigError! {}", self.message)
    }
}

impl From<FtaQlConfigOptional> for FtaQlConfigResolved {
    fn from(opt_config: FtaQlConfigOptional) -> Self {
        let default_config = get_default_config();
        FtaQlConfigResolved {
            includes: opt_config.includes.unwrap_or(default_config.includes),
            excludes: opt_config.excludes.unwrap_or(default_config.excludes),
            score_cap: opt_config.score_cap.unwrap_or(default_config.score_cap),
            include_comments: opt_config
                .include_comments
                .unwrap_or(default_config.include_comments),
            exclude_under: opt_config
                .exclude_under
                .unwrap_or(default_config.exclude_under),
        }
    }
}

pub fn get_default_config() -> FtaQlConfigResolved {
    FtaQlConfigResolved {
        includes: vec![
            "**/*.js".to_string(),
            "**/*.jsx".to_string(),
            "**/*.ts".to_string(),
            "**/*.tsx".to_string(),
        ],
        excludes: vec![
            "**/*.d.ts".to_string(),
            "**/*.min.js".to_string(),
            "**/*.bundle.js".to_string(),
            "dist/**".to_string(),
            "bin/**".to_string(),
            "build/**".to_string(),
        ],
        score_cap: 1000,
        include_comments: false,
        exclude_under: 6,
    }
}

pub fn read_config(
    config_path: String,
    path_specified_by_user: bool,
) -> Result<FtaQlConfigResolved, ConfigError> {
    let default_config = get_default_config();
    if Path::new(&config_path).exists() {
        let mut file = File::open(config_path).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        let provided_config: FtaQlConfigOptional =
            serde_json::from_str(&content).unwrap_or_default();

        // User-provided includes/excludes replace defaults entirely.
        return Result::Ok(FtaQlConfigResolved {
            includes: provided_config.includes.unwrap_or(default_config.includes),
            excludes: provided_config.excludes.unwrap_or(default_config.excludes),
            score_cap: provided_config
                .score_cap
                .unwrap_or(default_config.score_cap),
            exclude_under: provided_config
                .exclude_under
                .unwrap_or(default_config.exclude_under),
            include_comments: provided_config
                .include_comments
                .unwrap_or(default_config.include_comments),
        });
    }

    if !path_specified_by_user {
        return Result::Ok(default_config);
    }

    Result::Err(ConfigError {
        message: format!("Config file not found at file path: {}", config_path),
    })
}
