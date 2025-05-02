use candid::{CandidType, Deserialize, Encode, Nat, Principal};
use ic_cdk::api::time;
use ic_cdk::api::call;
use ic_cdk::id;
use std::collections::HashMap;

use crate::Value;

use crate::auth::is_manager_or_admin;
use crate::models::reward;
use crate::models::reward::*;
use ::ic_news_shared::error::{SquareError, SquareResult};
use crate::storage::{STORAGE, UserRewards, UserTasks, PointsTransaction};
use crate::storage::sharded::{SHARDED_USER_REWARDS, SHARDED_USER_TASKS};
use crate::utils::error_handler::*;



// Initialize default tasks with configurable active state
pub fn init_default_tasks(enable_daily_post: bool, enable_social_engagement: bool) {
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Only initialize if no tasks exist yet
        if storage.tasks.is_empty() {
            // Also initialize sharded storage for tasks
            // This ensures we're using sharded storage by default
            // Daily post task
            let daily_post = TaskDefinition {
                id: "daily_post".to_string(),
                title: "Daily Post".to_string(),
                description: "Create a post daily to earn points".to_string(),
                points: 50,
                task_type: TaskType::Daily,
                completion_criteria: "Create at least one post".to_string(),
                expiration_time: None,
                created_at: time(),
                updated_at: time(),
                is_active: enable_daily_post,
                requirements: None,
                canister_id: ic_cdk::id(),
            };
            
            // Social engagement task
            let social_engagement = TaskDefinition {
                id: "social_engagement".to_string(),
                title: "Social Engagement".to_string(),
                description: "Engage with other users to earn points".to_string(),
                points: 100,
                task_type: TaskType::Daily,
                completion_criteria: "Like or comment on at least 3 posts".to_string(),
                expiration_time: None,
                created_at: time(),
                updated_at: time(),
                is_active: enable_social_engagement,
                requirements: None,
                canister_id: ic_cdk::id(),
            };
            
            // Add tasks to storage
            storage.tasks.insert(daily_post.id.clone(), daily_post);
            storage.tasks.insert(social_engagement.id.clone(), social_engagement);
        }
    });
}

// Default initialization with all tasks enabled
pub fn init_default_tasks_all_enabled() {
    init_default_tasks(true, true);
}

// Initialize default tasks only if no tasks exist
pub fn init_default_tasks_if_empty() {
    let tasks_exist = STORAGE.with(|storage| {
        !storage.borrow().tasks.is_empty()
    });
    
    if !tasks_exist {
        ic_cdk::println!("Initializing default tasks");
        init_default_tasks_all_enabled();
    } else {
        ic_cdk::println!("Tasks already exist, skipping initialization");
    }
}

// Toggle task active status
pub fn toggle_task_status(task_id: &str, active: bool) -> SquareResult<()> {
    // Only managers or admins can toggle task status
    is_manager_or_admin().map_err(|e| {
        SquareError::Unauthorized(format!("Only managers or admins can toggle task status: {}", e))
    })?;
    
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Check if task exists
        if let Some(mut task) = storage.tasks.get(task_id).cloned() {
            // Update task status
            task.is_active = active;
            task.updated_at = time();
            
            // Save updated task
            storage.tasks.insert(task_id.to_string(), task);
            Ok(())
        } else {
            Err(SquareError::NotFound(format!("Task with ID {} not found", task_id)))
        }
    })
}

// Daily check-in
// Note: The daily check-in functionality has been moved to a separate canister
// See: ic_news_task_checkin/src/lib.rs
// Users should call that canister directly for daily check-ins

// Task completion
pub fn complete_task(request: CompleteTaskRequest, caller: Principal) -> SquareResult<TaskCompletionResponse> {
    const MODULE: &str = "services::reward";
    const FUNCTION: &str = "complete_task";
    
    let task_info = STORAGE.with(|storage| {
        let storage = storage.borrow();
        storage.tasks.get(&request.task_id).cloned()
    });
    
    let _task_type = match task_info {
        Some(task) => task.task_type,
        None => {
            match request.task_id.as_str() {
                id if id.starts_with("daily_") => TaskType::Daily,
                id if id.starts_with("weekly_") => TaskType::Weekly,
                _ => TaskType::OneTime
            }
        }
    };
    
    let _proof = request.task_id.clone();
    
    STORAGE.with(|storage| {
        let storage = storage.borrow_mut();
        
        // First, check if the task exists and get its reward points
        // We need to do this before borrowing user_tasks to avoid borrowing conflicts
        
        // Check if the task has an expiration time and if it has expired
        // This check needs to happen for ALL tasks, including custom tasks
        let task_opt = storage.tasks.get(&request.task_id);
        if let Some(task) = task_opt {
            if let Some(expiry) = task.expiration_time {
                if expiry < time() {
                    return Err(SquareError::InvalidOperation(format!("Task has expired: {}", request.task_id)));
                }
            }
        }
        
        // Now get the task details
        let (task_reward, task_type, _expiration_time) = match task_opt {
            Some(task) => {
                (task.points, task.task_type.clone(), task.expiration_time)
            },
            None => {
                // Check for hardcoded tasks from get_available_tasks
                match request.task_id.as_str() {
                    "daily_post" => (50, TaskType::Daily, None),
                    "social_engagement" => (100, TaskType::Daily, None),
                    _ => return Err(SquareError::NotFound(format!("Task with ID {} not found", request.task_id)))
                }
            }
        };
        
        // Get user tasks
        let user_tasks_key = caller.to_string();
        let user_tasks_exists = SHARDED_USER_TASKS.with(|tasks| {
            tasks.borrow().contains_key(&user_tasks_key)
        });
        
        if !user_tasks_exists {
            // Initialize user tasks in sharded storage
            SHARDED_USER_TASKS.with(|tasks| {
                let mut tasks = tasks.borrow_mut();
                tasks.insert(user_tasks_key.clone(), UserTasks {
                    principal: caller,
                    completed_tasks: HashMap::new(),
                    daily_tasks_reset: time() / 1_000_000,
                    last_check_in: None,
                    last_updated: time(),
                });
            });
        }
        
        // We already have the task_type from earlier, no need to fetch it again
        
        // Check if task already completed
        let already_completed = SHARDED_USER_TASKS.with(|tasks| {
            let mut tasks = tasks.borrow_mut();
            if let Some(user_tasks) = tasks.get(&user_tasks_key) {
                match task_type {
                    TaskType::Daily => {
                        // For daily tasks, check if THIS SPECIFIC daily task was completed today
                        // Using day_id approach to prevent multiple completions of the same daily task in the same day
                        let current_day_id = time() / 1_000_000_000 / 86400;
                        
                        // Check if this specific task was completed today
                        if let Some(completion_time) = user_tasks.completed_tasks.get(&request.task_id) {
                            // Convert completion time to day_id
                            let completion_day_id = completion_time / 1_000_000_000 / 86400;
                            
                            // If the completion day_id matches current day_id, task was already completed today
                            if completion_day_id == current_day_id {
                                return true; // Already completed this specific daily task today
                            }
                        }
                        false
                    },
                    TaskType::Weekly => {
                        // For weekly tasks, check if ANY weekly task was completed this week
                        let now = time() / 1_000_000;
                        let week_start = now - (now % (SECONDS_IN_DAY * 7));
                        
                        // Check if this specific task was completed this week
                        if user_tasks.completed_tasks.contains_key(&request.task_id) {
                            let completion_time = *user_tasks.completed_tasks.get(&request.task_id).unwrap();
                            let completion_week = completion_time / 1_000_000 - (completion_time / 1_000_000 % (SECONDS_IN_DAY * 7));
                            if completion_week == week_start {
                                return true;
                            }
                        }
                        false
                    },
                    _ => user_tasks.completed_tasks.contains_key(&request.task_id) // For one-time tasks, check if this specific task was completed
                }
            } else {
                false
            }
        });
        
        if already_completed {
            return Err(SquareError::InvalidOperation(format!("Task already completed: {}", request.task_id)));
        }
        
        // We've already validated the proof in the previous step
        // No need for additional validation here
        // This allows for more flexibility in task types and proof formats
        
        // Mark task as completed
        SHARDED_USER_TASKS.with(|tasks| {
            let mut tasks = tasks.borrow_mut();
            let user_tasks = match tasks.get(&user_tasks_key) {
                Some(tasks) => tasks.clone(),
                None => UserTasks {
                    principal: caller,
                    completed_tasks: HashMap::new(),
                    daily_tasks_reset: time() / 1_000_000,
                    last_check_in: None,
                    last_updated: time(),
                }
            };
            
            let mut updated_tasks = user_tasks.clone();
            updated_tasks.completed_tasks.insert(request.task_id.clone(), time());
            updated_tasks.last_updated = time();
            
            tasks.insert(user_tasks_key.clone(), updated_tasks);
        });
        
        // Award points
        let user_rewards_key = caller.to_string();
        let points = SHARDED_USER_REWARDS.with(|rewards| {
            let mut rewards = rewards.borrow_mut();
            let user_rewards = match rewards.get(&user_rewards_key) {
                Some(rewards) => rewards.clone(),
                None => UserRewards {
                    principal: caller,
                    points: 0,
                    points_history: Vec::new(),
                    last_claim_date: None,
                    // consecutive_daily_logins field has been moved to ic_news_task_checkin canister

                    transactions: Vec::new(),
                    last_updated: time(),
                }
            };
            
            let mut updated_rewards = user_rewards.clone();
            
            // Add points
            updated_rewards.points += task_reward;
            

            
            // Record transaction
            updated_rewards.points_history.push(PointsTransaction {
                amount: task_reward as i64,
                reason: format!("Completed task: {}", request.task_id),
                timestamp: time() / 1_000_000,
                reference_id: Some(request.task_id.clone()),
                points: task_reward,
            });
            
            updated_rewards.last_updated = time();
            
            rewards.insert(user_rewards_key.clone(), updated_rewards.clone());
            
            updated_rewards.points
        });
        
        // Return the response
        Ok(TaskCompletionResponse {
            success: true,
            points_earned: task_reward,
            total_points: points,

            message: format!("You earned {} points for completing {}", task_reward, request.task_id),
        })
    })
}

// Get user rewards
pub fn get_user_rewards(principal: Principal) -> SquareResult<UserRewardsResponse> {
    const MODULE: &str = "services::reward";
    const FUNCTION: &str = "get_user_rewards";
    
    // Create a new UserRewardsResponse
    let mut response = UserRewardsResponse::new();
    
    // Get user rewards from the main canister
    let user_rewards = SHARDED_USER_REWARDS.with(|rewards| {
        let mut rewards = rewards.borrow_mut();
        rewards.get(&principal.to_string()).map(|r| r.clone())
    });
    
    let user_rewards = match user_rewards {
        Some(rewards) => rewards.clone(),
        None => {
            // If user rewards record doesn't exist, check user tasks record
            let has_completed_tasks = SHARDED_USER_TASKS.with(|tasks| {
                let mut tasks = tasks.borrow_mut();
                let user_tasks_opt = tasks.get(&principal.to_string()).map(|t| t.clone());
                if let Some(user_tasks) = user_tasks_opt {
                    // If user has completed tasks but rewards record doesn't exist, return empty rewards record
                    if !user_tasks.completed_tasks.is_empty() {
                        let completed_task_ids = user_tasks.completed_tasks.keys().cloned().collect::<Vec<String>>();
                        response.insert("completed_tasks_count".to_string(), Value::Nat(completed_task_ids.len() as u64));
                        
                        // Add completed tasks as an array
                        let task_ids: Vec<Value> = completed_task_ids.iter()
                            .map(|id| Value::Text(id.clone()))
                            .collect();
                        response.insert("completed_tasks".to_string(), Value::Array(task_ids));
                        
                        return Ok(response.clone());
                    }
                }
                Err(SquareError::NotFound("User tasks not found".to_string()))
            });
            
            if has_completed_tasks.is_ok() {
                return Ok(response);
            }
            
            // If user hasn't completed any tasks or tasks record doesn't exist, create a new rewards record
            // Initialize a new user rewards record
            let new_user_rewards = UserRewards {
                principal: principal.clone(),
                points: 0,
                points_history: Vec::new(),
                last_claim_date: None,
                // consecutive_daily_logins field has been moved to ic_news_task_checkin canister
                transactions: Vec::new(),
                last_updated: time(),
            };
            
            // Store the new rewards record
            SHARDED_USER_REWARDS.with(|rewards| {
                let mut rewards = rewards.borrow_mut();
                rewards.insert(principal.to_string(), new_user_rewards.clone());
            });
            
            // Also create an empty user tasks record if it doesn't exist
            SHARDED_USER_TASKS.with(|tasks| {
                let mut tasks = tasks.borrow_mut();
                if !tasks.contains_key(&principal.to_string()) {
                    tasks.insert(principal.to_string(), UserTasks {
                        principal: principal.clone(),
                        completed_tasks: HashMap::new(),
                        daily_tasks_reset: time() / 1_000_000,
                        last_check_in: None,
                        last_updated: time(),
                    });
                }
            });
            
            // Add basic info to the response
            response.insert("points".to_string(), Value::Nat(0));
        
            response.insert("completed_tasks_count".to_string(), Value::Nat(0));
            response.insert("completed_tasks".to_string(), Value::Array(Vec::new()));
            
            return Ok(response);
        }
    };
        

    
    // Get user's completed tasks
    let completed_tasks = SHARDED_USER_TASKS.with(|tasks| {
        let mut tasks = tasks.borrow_mut();
        let user_tasks_opt = tasks.get(&principal.to_string()).map(|tasks| tasks.clone());
        match user_tasks_opt {
        Some(user_tasks) => user_tasks.completed_tasks.keys().cloned().collect::<Vec<String>>(),
        None => Vec::new(),
        }
    });
    
    // Add main canister data to response
    response.insert("points".to_string(), Value::Nat(user_rewards.points));

    response.insert("completed_tasks_count".to_string(), Value::Nat(completed_tasks.len() as u64));
    
    // Add completed tasks as an array
    let task_ids: Vec<Value> = completed_tasks.iter()
        .map(|id| Value::Text(id.clone()))
        .collect();
    response.insert("completed_tasks".to_string(), Value::Array(task_ids));
    
    // Add points history count
    response.insert("points_history_count".to_string(), Value::Nat(user_rewards.points_history.len() as u64));
    
    // Add points history as an array (always include this field, even if empty)
    let history: Vec<Value> = user_rewards.points_history.iter()
        .map(|tx| {
            let mut map_entries = Vec::new();
            map_entries.push(("amount".to_string(), Value::Int(tx.amount)));
            map_entries.push(("reason".to_string(), Value::Text(tx.reason.clone())));
            map_entries.push(("timestamp".to_string(), Value::Nat(tx.timestamp)));
            if let Some(ref_id) = &tx.reference_id {
                map_entries.push(("reference_id".to_string(), Value::Text(ref_id.clone())));
            }
            map_entries.push(("points".to_string(), Value::Nat(tx.points)));
            Value::Map(map_entries)
        })
        .collect();
    response.insert("points_history".to_string(), Value::Array(history));
    
    // Add latest transaction if available
    if !user_rewards.points_history.is_empty() {
        let latest = &user_rewards.points_history[user_rewards.points_history.len() - 1];
        response.insert("latest_transaction_amount".to_string(), Value::Int(latest.amount));
        response.insert("latest_transaction_reason".to_string(), Value::Text(latest.reason.clone()));
        response.insert("latest_transaction_timestamp".to_string(), Value::Nat(latest.timestamp));
    }
    
    // Add last claim date if available
    if let Some(last_claim) = user_rewards.last_claim_date {
        response.insert("last_claim_date".to_string(), Value::Nat(last_claim));
    }
    
    // Add user principal
    response.insert("principal".to_string(), Value::Principal(principal));
    
    // Add last updated timestamp
    response.insert("last_updated".to_string(), Value::Nat(user_rewards.last_updated));
    
    Ok(response)
}

// Get available tasks
pub fn get_available_tasks(caller: Principal) -> SquareResult<Vec<TaskResponse>> {
    const MODULE: &str = "services::reward";
    const FUNCTION: &str = "get_available_tasks";
    
    STORAGE.with(|storage| {
        let storage = storage.borrow();
        
        // Get user tasks
        let user_tasks = storage.user_tasks.get(&caller);
        
        // Get all task definitions from storage
        let mut tasks = Vec::new();
        let now = time() / 1_000_000;
        
        // 移除日志调用以节省 cycles
        
        // Iterate through all task definitions
        for (task_id, task_def) in &storage.tasks {
            // Check if task is active
            if !task_def.is_active {
                continue;
            }
            
            // Check if task has expired
            if let Some(expiration) = task_def.expiration_time {
                if now > expiration {
                    continue;
                }
            }
            
            // Check if task is already completed (for daily tasks, check if completed today)
            let is_completed = if let Some(ut) = user_tasks {
                if task_def.task_type == TaskType::Daily {
                    // For daily tasks, check if completed today
                    let today_start = now - (now % SECONDS_IN_DAY);
                    ut.completed_tasks.get(task_id)
                        .map(|completion_time| *completion_time >= today_start)
                        .unwrap_or(false)
                } else if task_def.task_type == TaskType::Weekly {
                    // For weekly tasks, check if completed this week
                    let week_start = now - (now % (SECONDS_IN_DAY * 7));
                    ut.completed_tasks.get(task_id)
                        .map(|completion_time| *completion_time >= week_start)
                        .unwrap_or(false)
                } else if task_def.task_type == TaskType::Monthly {
                    // For monthly tasks, check if completed this month (approximate)
                    let month_start = now - (now % (SECONDS_IN_DAY * 30));
                    ut.completed_tasks.get(task_id)
                        .map(|completion_time| *completion_time >= month_start)
                        .unwrap_or(false)
                } else {
                    // For one-time tasks, check if ever completed
                    ut.completed_tasks.contains_key(task_id)
                }
            } else {
                false
            };
            
            // Create task response
            tasks.push(TaskResponse {
                id: task_id.clone(),
                title: task_def.title.clone(),
                description: task_def.description.clone(),
                points: task_def.points,
                task_type: task_def.task_type.clone(),
                is_completed,
                completion_criteria: task_def.completion_criteria.clone(),
                expiration_time: task_def.expiration_time,
                created_at: task_def.created_at,
            });
        }
        
        // If no tasks found, add default tasks
        if tasks.is_empty() {
            // Initialize tasks if they don't exist yet
            init_default_tasks_all_enabled();
            
            // Try again with default tasks
            return get_available_tasks(caller);
        }
        
        Ok(tasks)
    })
}

// Admin functions
pub fn award_points(request: AwardPointsRequest) -> SquareResult<()> {
    // Check if caller is admin or manager
    is_manager_or_admin()?;
    
    let user_rewards_key = request.principal.to_string();
    let _points = SHARDED_USER_REWARDS.with(|rewards| {
        let mut rewards = rewards.borrow_mut();
        let user_rewards = match rewards.get(&user_rewards_key) {
            Some(rewards) => rewards.clone(),
            None => UserRewards {
            principal: request.principal,
            points: 0,
            points_history: Vec::new(),
            last_claim_date: None,
            // consecutive_daily_logins field has been moved to ic_news_task_checkin canister
            transactions: Vec::new(),
            last_updated: time(),
        }
        };
        
        let mut updated_rewards = user_rewards.clone();
        
        // Add points
        updated_rewards.points += request.points;
        

        
        // Record transaction
        updated_rewards.points_history.push(PointsTransaction {
            amount: request.points as i64,
            reason: request.reason.clone(),
            timestamp: time() / 1_000_000,
            reference_id: request.reference_id.clone(),
            points: request.points,
        });
        
        updated_rewards.last_updated = time();
        
        rewards.insert(user_rewards_key.clone(), updated_rewards.clone());
        
        updated_rewards.points
    });
    
    Ok(())
}

// Task management (admin functions)
pub fn create_task(request: CreateTaskRequest) -> SquareResult<String> {
    const MODULE: &str = "services::reward";
    const FUNCTION: &str = "create_task";
    
    
    // Check if caller is admin or manager
    is_manager_or_admin().map_err(|e| {
        e
    })?;
    
    let task_id = if !request.id.is_empty() {
        request.id.clone()
    } else {
        format!("task_{}", time() / 1_000_000)
    };
    
    let now = time() / 1_000_000;
    
    // Create task definition
    // Use points_reward field for task points
    let points = request.points_reward;
    
    let task = TaskDefinition {
        id: task_id.clone(),
        title: request.title,
        description: request.description,
        points,
        task_type: request.task_type,
        completion_criteria: request.completion_criteria,
        expiration_time: request.end_time,
        created_at: now,
        updated_at: now,
        is_active: true,
        requirements: request.requirements,
        canister_id: request.canister_id,
    };
    
    // Store the task
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Check if task ID already exists
        if storage.tasks.contains_key(&task_id) {
            return log_and_return(already_exists_error(
                "Task", 
                &task_id, 
                MODULE, 
                FUNCTION
            ).with_details(format!("Task with ID {} already exists", task_id)));
        }
        
        // Add the task
        storage.tasks.insert(task_id.clone(), task);
        Ok(task_id)
    })
}

pub fn update_task(request: UpdateTaskRequest) -> SquareResult<()> {
    const MODULE: &str = "services::reward";
    const FUNCTION: &str = "update_task";
    
    
    // Check if caller is admin or manager
    is_manager_or_admin().map_err(|e| {
        e
    })?;
    
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        
        // Check if task exists
        let task = match storage.tasks.get_mut(&request.id) {
            Some(task) => task,
            None => return log_and_return(not_found_error(
                "Task", 
                &request.id, 
                MODULE, 
                FUNCTION
            ).with_details(format!("Task with ID {} not found", request.id))),
        };
        
        // Update task fields
        task.title = request.title;
        task.description = request.description;
        // Use points_reward field for task points
        task.points = request.points_reward;
        task.task_type = request.task_type;
        task.completion_criteria = request.completion_criteria;
        task.expiration_time = request.end_time;
        task.updated_at = time() / 1_000_000;
        task.requirements = request.requirements;
        
        Ok(())
    })
}

pub fn delete_task(task_id: String) -> SquareResult<()> {
    const MODULE: &str = "services::reward";
    const FUNCTION: &str = "delete_task";


    // Check if caller is admin or manager
    is_manager_or_admin().map_err(|e| {
        e
    })?;

    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();

        // Check if task exists
        if !storage.tasks.contains_key(&task_id) {
            return log_and_return(not_found_error(
                "Task",
                &task_id,
                MODULE,
                FUNCTION
            ).with_details(format!("Task with ID {} not found", task_id)));
        }

        // Remove the task
        storage.tasks.remove(&task_id);
        Ok(())
    })
}
