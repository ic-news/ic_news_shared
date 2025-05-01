use candid::{CandidType, Deserialize, Principal};
use crate::auth::permissions::{authenticated_call, CanisterRole};
use crate::content::{ContentVisibility, Post};

#[derive(CandidType, Deserialize, Clone)]
pub struct CreatePostParams {
    pub content: String,
    pub media_urls: Vec<String>,
    pub hashtags: Vec<String>,
    pub visibility: ContentVisibility,
}

// Square Interface
#[derive(CandidType, Deserialize)]
pub struct SquareInterface {
    pub canister_id: Principal,
}

impl SquareInterface {
    pub fn new(canister_id: Principal) -> Self {
        Self { canister_id }
    }

    // Create a new post with authentication
    pub async fn create_post(&self, params: CreatePostParams) -> Result<String, String> {
        let result: Result<(String,), _> = authenticated_call(
            self.canister_id,
            "create_post",
            (params,),
            CanisterRole::Square,
        )
        .await;
        result.map(|(post_id,)| post_id)
            .map_err(|e| format!("Failed to create post: {}", e.1))
    }

    // Get a post by ID with authentication
    pub async fn get_post(&self, post_id: String) -> Result<Option<Post>, String> {
        let result: Result<(Option<Post>,), _> = authenticated_call(
            self.canister_id,
            "get_post",
            (post_id,),
            CanisterRole::Square,
        )
        .await;
        result.map(|(post,)| post)
            .map_err(|e| format!("Failed to get post: {}", e.1))
    }

    // Delete a post with authentication
    pub async fn delete_post(&self, post_id: String) -> Result<(), String> {
        authenticated_call(
            self.canister_id,
            "delete_post",
            (post_id,),
            CanisterRole::Square,
        )
        .await
        .map(|_: ()| ())
        .map_err(|e| format!("Failed to delete post: {}", e.1))
    }

    // Get user posts with authentication
    pub async fn get_user_posts(
        &self,
        user_id: Principal,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<Post>, String> {
        let result: Result<(Vec<Post>,), _> = authenticated_call(
            self.canister_id,
            "get_user_posts",
            (user_id, offset, limit),
            CanisterRole::Square,
        )
        .await;
        result.map(|(posts,)| posts)
            .map_err(|e| format!("Failed to get user posts: {}", e.1))
    }
}
