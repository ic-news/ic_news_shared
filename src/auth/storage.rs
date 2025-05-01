use std::cell::RefCell;
use std::collections::HashSet;
use candid::{CandidType, Deserialize, Principal};

thread_local! {
    pub static STORAGE: RefCell<Storage> = RefCell::new(Storage::default());
}

impl Default for Storage {
    fn default() -> Self {
        Storage {
            admin: None,
            managers: HashSet::new(),
        }
    }
}

#[derive(CandidType, Deserialize, Clone)]
pub struct Storage {
    // Admin and managers
    pub admin: Option<Principal>,
    pub managers: HashSet<Principal>,
}

