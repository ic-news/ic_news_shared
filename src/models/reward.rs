use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::time;
use crate::models::value::Value;

// Define task types
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum TaskStatus {
    Active,
    Inactive,
    Completed,
    Expired,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum TaskType {
    DailyCheckIn,
    ContentCreation,
    SocialInteraction,
    CommunityEngagement,
    SpecialEvent,
    Custom,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum TaskDifficulty {
    Easy,
    Medium,
    Hard,
    VeryHard,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TaskDefinition {
    pub id: String,
    pub title: String,
    pub description: String,
    pub task_type: TaskType,
    pub difficulty: TaskDifficulty,
    pub points: u64,
    pub required_actions: u64,
    pub status: TaskStatus,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub created_at: u64,
    pub updated_at: u64,
    pub created_by: Principal,
    pub metadata: Option<Value>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TaskCompletionResponse {
    pub success: bool,
    pub task_id: String,
    pub points_earned: u64,
    pub total_points: u64,
    pub message: String,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TaskResponse {
    pub id: String,
    pub title: String,
    pub description: String,
    pub task_type: TaskType,
    pub difficulty: TaskDifficulty,
    pub points: u64,
    pub required_actions: u64,
    pub user_progress: Option<u64>,
    pub status: TaskStatus,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub created_at: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CompleteTaskRequest {
    pub task_id: String,
    pub action_count: Option<u64>,
    pub metadata: Option<Value>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct AwardPointsRequest {
    pub user_id: Principal,
    pub points: u64,
    pub reason: String,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: String,
    pub task_type: TaskType,
    pub difficulty: TaskDifficulty,
    pub points: u64,
    pub required_actions: u64,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub metadata: Option<Value>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UpdateTaskRequest {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub task_type: Option<TaskType>,
    pub difficulty: Option<TaskDifficulty>,
    pub points: Option<u64>,
    pub required_actions: Option<u64>,
    pub status: Option<TaskStatus>,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub metadata: Option<Value>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UserRewardsResponse {
    pub user_id: Principal,
    pub total_points: u64,
    pub completed_tasks: Vec<TaskResponse>,
    pub available_tasks: Vec<TaskResponse>,
    pub last_updated: u64,
}

// Task completion record
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TaskCompletionRecord {
    pub task_id: String,
    pub user_id: Principal,
    pub completed_at: u64,
    pub points_awarded: u64,
}

// User task progress
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UserTaskProgress {
    pub user_id: Principal,
    pub task_id: String,
    pub progress: u64,
    pub max_progress: u64,
    pub last_updated: u64,
}

impl UserTaskProgress {
    pub fn new(user_id: Principal, task_id: String, max_progress: u64) -> Self {
        Self {
            user_id,
            task_id,
            progress: 0,
            max_progress,
            last_updated: time(),
        }
    }
    
    pub fn update_progress(&mut self, progress: u64) {
        self.progress = progress;
        self.last_updated = time();
    }
    
    pub fn is_complete(&self) -> bool {
        self.progress >= self.max_progress
    }
}
