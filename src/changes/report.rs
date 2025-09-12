use crate::changes::status::NextStepOwner;
use crate::context::ServiceContext;
use chrono::{DateTime, Utc};
use enum_map::{Enum, EnumMap};
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

/// A nested map structure that tracks counts by time interval and next step owner
#[derive(Debug, Default)]
pub struct ChangesByOwnerAndTime(
    EnumMap<TimeInterval, EnumMap<NextStepOwner, (u64, Vec<u64>)>>,
);

impl ChangesByOwnerAndTime {
    /// Increment the count for a specific time interval and next step owner combination
    pub fn increment(
        &mut self,
        time_interval: TimeInterval,
        owner: NextStepOwner,
        id_number: u64,
    ) {
        let i = &mut self.0[time_interval][owner];
        i.0 += 1;
        i.1.push(id_number);
    }

    /// Get the count for a specific time interval and next step owner combination
    pub fn get_count(
        &self,
        time_interval: TimeInterval,
        owner: NextStepOwner,
    ) -> u64 {
        self.0[time_interval][owner].0
    }

    pub fn get_changes(
        &self,
        time_interval: TimeInterval,
        owner: NextStepOwner,
    ) -> Vec<u64> {
        let mut result = self.0[time_interval][owner].1.clone();
        result.sort();
        result
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
pub fn report(
    context: &ServiceContext,
    project: Option<String>,
    owner: Option<String>,
) -> String {
    report_by_owner_time(&changes_by_owner_time(context, project, owner))
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
