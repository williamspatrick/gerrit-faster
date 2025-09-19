use crate::changes::status::NextStepOwner;
use crate::context::ServiceContext;
use chrono::{DateTime, Utc};
use enum_map::{Enum, EnumMap};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Enum)]
pub enum TimeInterval {
    Under24Hours,
    Under72Hours,
    Under2Weeks,
    Under8Weeks,
    Over8Weeks,
}

impl TimeInterval {
    pub fn from_timestamp(timestamp: DateTime<Utc>) -> Self {
        let now = Utc::now();
        let duration = now.signed_duration_since(timestamp);

        if duration.num_hours() < 24 {
            TimeInterval::Under24Hours
        } else if duration.num_hours() < 72 {
            TimeInterval::Under72Hours
        } else if duration.num_weeks() < 2 {
            TimeInterval::Under2Weeks
        } else if duration.num_weeks() < 8 {
            TimeInterval::Under8Weeks
        } else {
            TimeInterval::Over8Weeks
        }
    }
}

impl fmt::Display for TimeInterval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TimeInterval::Under24Hours => write!(f, "<24 hours"),
            TimeInterval::Under72Hours => write!(f, "<72 hours"),
            TimeInterval::Under2Weeks => write!(f, "<2 weeks"),
            TimeInterval::Under8Weeks => write!(f, "<8 weeks"),
            TimeInterval::Over8Weeks => write!(f, ">8 weeks"),
        }
    }
}

/// A map structure that tracks counts by next step owner
#[derive(Debug, Default)]
pub struct ChangesByOwner(EnumMap<NextStepOwner, (u64, Vec<u64>)>);

impl ChangesByOwner {
    /// Increment the count for a specific next step owner
    pub fn increment(&mut self, owner: NextStepOwner, id_number: u64) {
        let i = &mut self.0[owner];
        i.0 += 1;
        i.1.push(id_number);
    }

    /// Get the count for a specific next step owner
    pub fn get_count(&self, owner: NextStepOwner) -> u64 {
        self.0[owner].0
    }

    pub fn get_changes(&self, owner: NextStepOwner) -> Vec<u64> {
        let mut result = self.0[owner].1.clone();
        result.sort();
        result
    }
}

/// A nested map structure that tracks counts by time interval and next step owner
#[derive(Debug, Default)]
pub struct ChangesByOwnerAndTime(EnumMap<TimeInterval, ChangesByOwner>);

impl ChangesByOwnerAndTime {
    /// Increment the count for a specific time interval and next step owner combination
    pub fn increment(
        &mut self,
        time_interval: TimeInterval,
        owner: NextStepOwner,
        id_number: u64,
    ) {
        self.0[time_interval].increment(owner, id_number);
    }

    /// Get the count for a specific time interval and next step owner combination
    pub fn get_count(
        &self,
        time_interval: TimeInterval,
        owner: NextStepOwner,
    ) -> u64 {
        self.0[time_interval].get_count(owner)
    }

    pub fn get_changes(
        &self,
        time_interval: TimeInterval,
        owner: NextStepOwner,
    ) -> Vec<u64> {
        self.0[time_interval].get_changes(owner)
    }
}

/// A map structure that tracks changes by owner, organized by repository
#[derive(Debug, Default)]
pub struct ChangesByOwnerAndRepo(HashMap<String, ChangesByOwner>);

impl ChangesByOwnerAndRepo {
    /// Increment the count for a specific repository and owner
    pub fn increment(
        &mut self,
        repo: String,
        owner: NextStepOwner,
        id_number: u64,
    ) {
        self.0
            .entry(repo)
            .or_insert_with(ChangesByOwner::default)
            .increment(owner, id_number);
    }

    /// Get the changes by owner for a specific repository
    pub fn get_repo_changes(&self, repo: &str) -> Option<&ChangesByOwner> {
        self.0.get(repo)
    }

    /// Get all repositories
    pub fn get_repos(&self) -> Vec<&String> {
        self.0.keys().collect()
    }
}

pub fn changes_by_owner_time(
    context: &ServiceContext,
    project: Option<String>,
    owner: Option<String>,
) -> ChangesByOwnerAndTime {
    let mut changes = ChangesByOwnerAndTime::default();

    for (_, change) in &context.lock().unwrap().changes.changes {
        if let Some(ref project_name) = project
            && !change.change.project.eq(project_name)
        {
            continue;
        }
        if let Some(ref owner_name) = owner
            && !change.change.owner.username.eq(owner_name)
        {
            continue;
        }

        let time_unit =
            TimeInterval::from_timestamp(change.review_state_updated);
        let owner = NextStepOwner::from(change.review_state.clone());

        changes.increment(time_unit, owner, change.change.id_number);
    }

    changes
}

pub fn changes_by_owner_repo(
    context: &ServiceContext,
    owner: Option<String>,
) -> ChangesByOwnerAndRepo {
    let mut changes = ChangesByOwnerAndRepo::default();

    for (_, change) in &context.lock().unwrap().changes.changes {
        if let Some(ref owner_name) = owner
            && !change.change.owner.username.eq(owner_name)
        {
            continue;
        }

        let repo = change.change.project.clone();
        let owner = NextStepOwner::from(change.review_state.clone());

        changes.increment(repo, owner, change.change.id_number);
    }

    changes
}

pub fn report_by_time(
    context: &ServiceContext,
    project: Option<String>,
    owner: Option<String>,
) -> String {
    report_by_owner_time(&changes_by_owner_time(context, project, owner))
}

pub fn report_by_repo<F>(
    context: &ServiceContext,
    owner: Option<String>,
    repo_transform: Option<F>,
) -> String
where
    F: Fn(&str) -> String,
{
    report_by_owner_repo(&changes_by_owner_repo(context, owner), repo_transform)
}

pub fn report_by_owner_time(changes: &ChangesByOwnerAndTime) -> String {
    let mut table = comfy_table::Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS)
        .set_header(vec!["", "Community", "Maintainers", "Author"]);

    // Iterate over all time intervals and add rows dynamically
    for time_interval in [
        TimeInterval::Under24Hours,
        TimeInterval::Under72Hours,
        TimeInterval::Under2Weeks,
        TimeInterval::Under8Weeks,
        TimeInterval::Over8Weeks,
    ] {
        table.add_row(vec![
            time_interval.to_string(),
            changes
                .get_count(time_interval, NextStepOwner::Community)
                .to_string(),
            changes
                .get_count(time_interval, NextStepOwner::Maintainer)
                .to_string(),
            changes
                .get_count(time_interval, NextStepOwner::Author)
                .to_string(),
        ]);
    }

    table.to_string()
}

pub fn report_by_owner_repo<F>(
    changes: &ChangesByOwnerAndRepo,
    repo_transform: Option<F>,
) -> String
where
    F: Fn(&str) -> String,
{
    let mut table = comfy_table::Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS)
        .set_header(vec!["", "Community", "Maintainers", "Author"]);

    // Get all repos and sort them for consistent output
    let mut repos: Vec<&String> = changes.get_repos();
    repos.sort();

    // Add a row for each repository (using plain repo names)
    for repo in &repos {
        if let Some(repo_changes) = changes.get_repo_changes(repo) {
            table.add_row(vec![
                (*repo).clone(),
                repo_changes.get_count(NextStepOwner::Community).to_string(),
                repo_changes
                    .get_count(NextStepOwner::Maintainer)
                    .to_string(),
                repo_changes.get_count(NextStepOwner::Author).to_string(),
            ]);
        }
    }

    // Generate the table as string
    let table_string = table.to_string();

    // If we have a transform function, apply it post-generation
    if let Some(transform) = repo_transform {
        let mut result = table_string;
        // Replace each repo name with its transformed version
        // We need to be careful about partial matches, so we'll replace
        // with delimiters (whitespace) around the repo names
        for repo in &repos {
            let plain_repo = (*repo).to_string();
            let transformed_repo = transform(repo);

            // Replace with proper delimiters to avoid partial matches
            // comfy-table typically adds spaces around cell content
            result = result.replace(
                &format!(" {} ", plain_repo),
                &format!(" {} ", transformed_repo),
            );
        }
        return result;
    }

    table_string
}
