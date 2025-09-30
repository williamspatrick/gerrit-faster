use crate::gerrit::data::ChangeInfo;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct CommunityPatternsConfig {
    rejected_repos: Vec<String>,
    rejected_project_regex: Vec<String>,
    rejected_files: Vec<String>,
    rejected_file_regex: HashMap<String, Vec<String>>,
}

impl CommunityPatternsConfig {
    fn from_str(yaml_str: &str) -> Self {
        serde_yaml::from_str(yaml_str)
            .expect("Failed to parse rejected patterns config")
    }
}

fn community_repo(
    change: &ChangeInfo,
    config: &CommunityPatternsConfig,
) -> bool {
    if config.rejected_repos.contains(&change.project) {
        return false;
    }

    for pattern in &config.rejected_project_regex {
        if let Ok(regex) = Regex::new(pattern) {
            if regex.is_match(&change.project) {
                return false;
            }
        }
    }

    true
}

fn community_file(
    change: &ChangeInfo,
    config: &CommunityPatternsConfig,
) -> bool {
    let revision = match change.revisions.get(&change.current_revision) {
        Some(rev) => rev,
        None => return true,
    };

    // Compile all regex patterns for this change using iterator chains
    let all_regex_patterns: Vec<Regex> = config
        .rejected_file_regex
        .get("all")
        .into_iter()
        .flatten()
        .chain(
            config
                .rejected_file_regex
                .get(&change.project)
                .into_iter()
                .flatten(),
        )
        .filter_map(|pattern| Regex::new(pattern).ok())
        .collect();

    // Check if all files match rejected patterns
    // Return false only if ALL files are rejected
    for (file_path, _file_info) in &revision.files {
        // If this file is not rejected, include the change
        if !config.rejected_files.contains(&file_path.to_string())
            && !all_regex_patterns
                .iter()
                .any(|regex| regex.is_match(file_path))
        {
            return true;
        }
    }

    // If we get here, all files were rejected
    false
}

pub fn should_include_change(change: &ChangeInfo) -> bool {
    let config = CommunityPatternsConfig::from_str(include_str!(
        "../../config/rejected_patterns.yaml"
    ));

    community_repo(change, &config) && community_file(change, &config)
}
