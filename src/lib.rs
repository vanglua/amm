
#![warn(clippy::all, clippy::pedantic)]
#![allow(dead_code, clippy::struct_excessive_bools, clippy::ptr_arg, clippy::tabs_in_doc_comments, clippy::too_many_arguments)]
#[cfg(feature = "wee_alloc")]

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


use uint::construct_uint;
construct_uint! {
    /// 256-bit unsigned integer.
    pub struct u256(4);
}

#[macro_use]
/** core */
mod pool_factory;
mod pool;
mod vault_token;

/** utils */
mod math;
mod logger;

mod constants;


#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests;