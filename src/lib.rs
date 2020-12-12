#![warn(clippy::all, clippy::pedantic)]
#![allow(dead_code, clippy::struct_excessive_bools, clippy::ptr_arg, clippy::tabs_in_doc_comments, clippy::too_many_arguments)]
#[cfg(feature = "wee_alloc")]

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[macro_use]
#[allow(dead_code)]
mod vault_token;
mod pool;
mod pool_factory;
mod constants;
mod math;

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests;