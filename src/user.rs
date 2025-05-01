use candid::{CandidType, Deserialize, Principal};

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UserProfile {
    pub user_id: Principal,
    pub username: String,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub points: i64,
    pub created_at: u64,
    pub updated_at: u64,
}
