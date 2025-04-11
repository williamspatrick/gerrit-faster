use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ChangeInfo {
    pub change_id: String,
}
