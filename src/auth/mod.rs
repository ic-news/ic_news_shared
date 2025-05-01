use candid::Principal;
use ic_cdk::caller;
use crate::auth::storage::STORAGE;
use crate::error::{SquareError, SquareResult};

pub mod permissions;
pub mod storage;

pub fn init_admin() {
    STORAGE.with(|storage| {
        storage.borrow_mut().admin = Some(caller());
    });
}

pub fn init_admin_if_empty() {
    STORAGE.with(|storage| {
        let mut store = storage.borrow_mut();
        if store.admin.is_none() {
            ic_cdk::println!("Initializing admin as caller");
            store.admin = Some(caller());
        }
    });
}

pub fn is_admin() -> Result<(), String> {
    let caller = caller();
    STORAGE.with(|storage| {
        if storage.borrow().admin == Some(caller) {
            Ok(())
        } else {
            Err("Only admin can perform this action".to_string())
        }
    })
}

pub fn is_manager_or_admin() -> Result<(), String> {
    let caller = caller();
    STORAGE.with(|storage| {
        let s = storage.borrow();
        if s.admin == Some(caller) || s.managers.contains(&caller) {
            Ok(())
        } else {
            Err("Only managers or admin can perform this action".to_string())
        }
    })
}

#[ic_cdk_macros::update]
pub fn add_manager(manager: Principal) -> Result<(), String> {
    is_admin()?;
    STORAGE.with(|storage| {
        storage.borrow_mut().managers.insert(manager);
        Ok(())
    })
}

#[ic_cdk_macros::update]
pub fn remove_manager(manager: Principal) -> Result<(), String> {
    is_admin()?;
    STORAGE.with(|storage| {
        storage.borrow_mut().managers.remove(&manager);
        Ok(())
    })
}

#[ic_cdk_macros::query]
pub fn list_managers() -> Result<Vec<Principal>, String> {
    is_admin()?;
    Ok(STORAGE.with(|storage| {
        storage.borrow().managers.iter().cloned().collect()
    }))
}

// Get the authenticated caller or return an error
pub fn get_authenticated_caller() -> SquareResult<Principal> {
    let caller = caller();
    if caller == Principal::anonymous() {
        return Err(SquareError::Unauthorized("Anonymous principal not allowed".to_string()));
    }
    Ok(caller)
}

// Get the target principal if provided, otherwise return the authenticated caller
pub fn get_target_or_caller(principal: Option<Principal>) -> SquareResult<Principal> {
    match principal {
        Some(p) => Ok(p),
        None => get_authenticated_caller(),
    }
}