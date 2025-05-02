use candid::{CandidType, Deserialize, Principal};
use std::collections::HashSet;

// Constants for validation
pub const MIN_USERNAME_LENGTH: usize = 3;
pub const MAX_USERNAME_LENGTH: usize = 30;
pub const MAX_BIO_LENGTH: usize = 500;
pub const HANDLE_PATTERN: &str = r"^[a-zA-Z0-9_]{3,30}$";

// User data structures
#[derive(CandidType, Deserialize, Clone)]
pub struct User {
    pub principal: Principal,
    pub created_at: u64,
    pub last_login: u64,
    pub status: UserStatus,
    pub role: UserRole,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct UserProfile {
    pub principal: Principal,
    pub username: String,
    pub handle: String,
    pub bio: String,
    pub avatar: String,
    pub social_links: Vec<(String, String)>,
    pub interests: Vec<String>,
    pub followers: HashSet<Principal>,
    pub followed_users: HashSet<Principal>,
    pub followed_topics: HashSet<String>,
    pub followers_count: u64,
    pub following_count: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub privacy_settings: Option<UserPrivacySettings>,
}

// User Identifier
#[derive(CandidType, Deserialize, Clone)]
pub struct UserIdentifier {
    pub principal: Option<Principal>,
    pub handle: Option<String>,
}

// Request DTOs
#[derive(CandidType, Deserialize, Clone)]
pub struct RegisterUserRequest {
    pub username: String,
    pub handle: String,
    pub bio: String,
    pub avatar: String,
    pub social_links: Option<Vec<(String, String)>>,
    pub interests: Option<Vec<String>>,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct UpdateProfileRequest {
    pub username: Option<String>,
    pub handle: Option<String>,
    pub bio: Option<String>,
    pub avatar: Option<String>,
    pub social_links: Option<Vec<(String, String)>>,
    pub interests: Option<Vec<String>>,
    pub privacy_settings: Option<UserPrivacySettings>,
}

#[derive(CandidType, Deserialize, Clone, PartialEq)]
pub struct UserPrivacySettings {
    pub profile_visibility: ProfileVisibility,
    pub content_visibility: ContentVisibility,
    pub interaction_preferences: InteractionPreferences,
    pub notification_preferences: NotificationPreferences,
}

#[derive(CandidType, Deserialize, Clone, PartialEq)]
pub enum ProfileVisibility {
    Public,
    FollowersOnly,
    Private,
}

#[derive(CandidType, Deserialize, Clone, PartialEq)]
pub enum ContentVisibility {
    Public,
    FollowersOnly,
    Private,
}

#[derive(CandidType, Deserialize, Clone, PartialEq)]
pub struct InteractionPreferences {
    pub allow_comments: bool,
    pub allow_mentions: bool,
    pub allow_follows: bool,
    pub show_likes: bool,
}

#[derive(CandidType, Deserialize, Clone, PartialEq)]
pub struct NotificationPreferences {
    pub likes: bool,
    pub comments: bool,
    pub follows: bool,
    pub mentions: bool,
    pub system: bool,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct FollowUserRequest {
    pub user_to_follow: Principal,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct FollowTopicRequest {
    pub topic: String,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct UserStatusUpdateRequest {
    pub principal: Principal,
    pub status: UserStatus,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct UserRoleUpdateRequest {
    pub principal: Principal,
    pub role: UserRole,
}

// Response DTOs
#[derive(CandidType, Deserialize, Clone)]
pub struct UserProfileResponse {
    pub principal: Principal,
    pub username: String,
    pub handle: String,
    pub bio: String,
    pub avatar: String,
    pub social_links: Vec<(String, String)>,
    pub followers_count: u64,
    pub following_count: u64,
    pub registered_at: u64,
    pub last_login: u64,
    pub role: UserRole,
    pub status: UserStatus,
    pub is_following: bool,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UserSocialResponse {
    pub principal: Principal,
    pub username: String,
    pub handle: String,
    pub avatar: String,
    pub bio: String,
    pub followers_count: u64,
    pub following_count: u64,
    pub is_following: bool,
    pub is_followed_by_caller: bool,
}


#[derive(CandidType, Deserialize, Clone)]
pub struct UserResponse {
    pub principal: Principal,
    pub username: String,
    pub handle: String,
    pub bio: String,
    pub avatar: String,
    pub social_links: Vec<(String, String)>,
    pub interests: Vec<String>,
    pub followers_count: u64,
    pub following_count: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub status: UserStatus,
    pub role: UserRole,
    pub is_following: bool,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct UserLeaderboardItem {
    pub principal: Principal,
    pub username: String,
    pub handle: String,
    pub avatar: String,
    pub points: u64,
    pub rank: u64,
    pub last_claim_date: Option<u64>,
    pub consecutive_daily_logins: u64,
    pub post_count: u64,
    pub followers_count: u64,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct UserLeaderboardResponse {
    pub users: Vec<UserLeaderboardItem>,
    pub total_users: u64,
    pub has_more: bool,
    pub next_offset: u64,
}

// Notification structures
#[derive(CandidType, Deserialize, Clone)]
pub struct UserNotification {
    pub id: String,
    pub user_principal: Principal,
    pub notification_type: NotificationType,
    pub content: String,
    pub related_entity_id: Option<String>,
    pub related_user: Option<Principal>,
    pub is_read: bool,
    pub created_at: u64,
}

#[derive(CandidType, Deserialize, Clone, PartialEq)]
pub enum NotificationType {
    Follow,
    Like,
    Comment,
    Mention,
    System,
    Reward,
    ContentUpdate,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct NotificationResponse {
    pub id: String,
    pub notification_type: NotificationType,
    pub content: String,
    pub related_entity_id: Option<String>,
    pub related_user: Option<UserSocialResponse>,
    pub is_read: bool,
    pub created_at: u64,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct NotificationsResponse {
    pub notifications: Vec<NotificationResponse>,
    pub total_count: u64,
    pub unread_count: u64,
    pub has_more: bool,
    pub next_offset: u64,
}

// Pagination and Content types
#[derive(CandidType, Deserialize, Clone)]
pub struct PaginationParams {
    pub page: usize,
    pub page_size: usize,
}

// Enums
#[derive(CandidType, Deserialize, Clone, PartialEq)]
pub enum UserStatus {
    Active,
    Suspended,
    Banned,
    Restricted,
}

#[derive(CandidType, Deserialize, Clone, PartialEq)]
pub enum UserRole {
    User,
    Admin,
    Moderator,
    Creator,
}
