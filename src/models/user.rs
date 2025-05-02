use candid::{CandidType, Deserialize, Principal};
use std::collections::HashMap;
use ic_cdk::api::time;

// Define types that match those in user_center
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UserSocialResponse {
    pub principal_id: Principal,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub follower_count: u64,
    pub following_count: u64,
    pub is_following: bool,
    pub created_at: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UserProfileResponse {
    pub principal_id: Principal,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub website: Option<String>,
    pub email: Option<String>,
    pub follower_count: u64,
    pub following_count: u64,
    pub post_count: u64,
    pub comment_count: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub is_following: bool,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UserLeaderboardResponse {
    pub users: Vec<UserSocialResponse>,
    pub total: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct RegisterUserRequest {
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub website: Option<String>,
    pub email: Option<String>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub website: Option<String>,
    pub email: Option<String>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct FollowUserRequest {
    pub user_to_follow: Principal,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum NotificationType {
    Follow,
    Like,
    Comment,
    Mention,
    System,
    RewardEarned,
}

// User notification
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UserNotification {
    pub id: String,
    pub user_id: Principal,
    pub notification_type: NotificationType,
    pub content: String,
    pub related_entity_id: Option<String>,
    pub created_at: u64,
    pub read: bool,
}

// User privacy settings
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UserPrivacySettings {
    pub show_likes: bool,
    pub show_comments: bool,
    pub show_posts: bool,
    pub show_followers: bool,
    pub show_following: bool,
    pub last_updated: u64,
}

impl Default for UserPrivacySettings {
    fn default() -> Self {
        Self {
            show_likes: true,
            show_comments: true,
            show_posts: true,
            show_followers: true,
            show_following: true,
            last_updated: time(),
        }
    }
}