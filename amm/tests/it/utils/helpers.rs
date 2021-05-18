use crate::utils::*;

pub fn alice() -> AccountId {
    "alice".to_string()
}

pub fn bob() -> AccountId {
    "bob".to_string()
}

pub fn carol() -> AccountId {
    "carol".to_string()
}

pub fn empty_string() -> String { "".to_string() }

pub fn empty_string_vec(len: u16) -> Vec<String> { 
    let mut tags: Vec<String> = vec![];
    for i in 0..len {
        tags.push(empty_string());
    }
    
    tags
}

pub fn env_time() -> U64{ 
    1609951265967.into()
}

pub fn fee() -> U128 {
    (10_u128.pow(24) / 50).into() // 2%
}

pub fn transfer_call_storage_amount() -> u128 {
    1
}