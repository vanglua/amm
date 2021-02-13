#![warn(clippy::all, clippy::pedantic)]
#![allow(dead_code, clippy::must_use_candidate, clippy::too_many_lines, clippy::struct_excessive_bools, clippy::ptr_arg, clippy::tabs_in_doc_comments, clippy::too_many_arguments, clippy::cast_possible_truncation)]
#[cfg(feature = "wee_alloc")]

#[cfg(target = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


use uint::construct_uint;
construct_uint! {
    /// 256-bit unsigned integer.
    pub struct u256(4);
}

/** core */
pub mod protocol;
mod helper;
mod payload_structs;
mod pool_factory;
mod pool;
mod outcome_token;

/** utils */
pub mod math;
mod logger;

pub mod constants;