use crate::changes::status::NextStepOwner;
use crate::context::ServiceContext;
use chrono::Utc;

struct ChangesAge {
    under_24_hrs: u64,
    under_72_hrs: u64,
    under_2_weeks: u64,
    under_8_weeks: u64,
    over_8_weeks: u64,
}

impl ChangesAge {
    fn new() -> ChangesAge {
        ChangesAge {
            under_24_hrs: 0,
            under_72_hrs: 0,
            under_2_weeks: 0,
            under_8_weeks: 0,
            over_8_weeks: 0,
        }
    }
}

pub fn report(context: &ServiceContext, project: Option<String>) -> String {
    let mut author = ChangesAge::new();
    let mut community = ChangesAge::new();
    let mut maintainers = ChangesAge::new();

    for (_, change) in &context.lock().unwrap().changes.changes {
        if let Some(ref project_name) = project
            && !change.change.project.eq(project_name)
        {
            continue;
        }

        let change_type: &mut ChangesAge =
            match NextStepOwner::from(change.review_state.clone()) {
                NextStepOwner::Author => &mut author,
                NextStepOwner::Community => &mut community,
                NextStepOwner::Maintainer => &mut maintainers,
            };
        let now = Utc::now();

        if change.review_state_updated > now - chrono::Duration::hours(24) {
            change_type.under_24_hrs += 1;
        } else if change.review_state_updated
            > now - chrono::Duration::hours(72)
        {
            change_type.under_72_hrs += 1;
        } else if change.review_state_updated > now - chrono::Duration::weeks(2)
        {
            change_type.under_2_weeks += 1;
        } else if change.review_state_updated > now - chrono::Duration::weeks(8)
        {
            change_type.under_8_weeks += 1;
        } else {
            change_type.over_8_weeks += 1;
        }
    }
    let mut table = comfy_table::Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS)
        .set_header(vec!["", "Community", "Maintainers", "Author"])
        .add_row(vec![
            "<24 hrs".to_string(),
            community.under_24_hrs.to_string(),
            maintainers.under_24_hrs.to_string(),
            author.under_24_hrs.to_string(),
        ])
        .add_row(vec![
            "<72 hrs".to_string(),
            community.under_72_hrs.to_string(),
            maintainers.under_72_hrs.to_string(),
            author.under_72_hrs.to_string(),
        ])
        .add_row(vec![
            "<2 weeks".to_string(),
            community.under_2_weeks.to_string(),
            maintainers.under_2_weeks.to_string(),
            author.under_2_weeks.to_string(),
        ])
        .add_row(vec![
            "<8 weeks".to_string(),
            community.under_8_weeks.to_string(),
            maintainers.under_8_weeks.to_string(),
            author.under_8_weeks.to_string(),
        ])
        .add_row(vec![
            ">8 weeks".to_string(),
            community.over_8_weeks.to_string(),
            maintainers.over_8_weeks.to_string(),
            author.over_8_weeks.to_string(),
        ]);

    table.to_string()
}
