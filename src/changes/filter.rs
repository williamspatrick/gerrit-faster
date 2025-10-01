use crate::gerrit::data::ChangeInfo;
use fancy_regex::Regex;
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
        let regex = Regex::new(pattern).unwrap_or_else(|_| {
            panic!("Failed to compile regex pattern: {}", pattern)
        });
        if regex.is_match(&change.project).unwrap_or(false) {
            return false;
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

    // Collect all regex patterns for this change
    let all_regex_patterns: Vec<&String> = config
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
        .collect();

    // Compile all regex patterns, aborting on any failure
    let all_regex_patterns: Vec<Regex> = all_regex_patterns
        .into_iter()
        .map(|pattern| {
            Regex::new(pattern).unwrap_or_else(|_| {
                panic!("Failed to compile regex pattern: {}", pattern)
            })
        })
        .collect();

    // Check if all files match rejected patterns
    // Return false only if ALL files are rejected
    for (file_path, _file_info) in &revision.files {
        // If this file is not rejected, include the change
        if !config.rejected_files.contains(&file_path.to_string())
            && !all_regex_patterns
                .iter()
                .any(|regex| regex.is_match(file_path).unwrap_or(false))
        {
            return true;
        }
    }

    // If we get here, all files were rejected
    false
}

fn autobump_topic(change: &ChangeInfo) -> bool {
    change.topic == "autobump"
}

pub fn should_include_change(change: &ChangeInfo) -> bool {
    let config = CommunityPatternsConfig::from_str(include_str!(
        "../../config/rejected_patterns.yaml"
    ));

    community_repo(change, &config)
        && community_file(change, &config)
        && !autobump_topic(change)
}
