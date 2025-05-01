use candid::{CandidType, Deserialize, Principal};

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum ContentVisibility {
    Public,
    Private,
    Followers,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Post {
    pub id: String,
    pub author: Principal,
    pub content: String,
    pub media_urls: Vec<String>,
    pub hashtags: Vec<String>,
    pub likes: u64,
    pub comments: u64,
    pub visibility: ContentVisibility,
    pub created_at: u64,
    pub updated_at: Option<u64>,
}
