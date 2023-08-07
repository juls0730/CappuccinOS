#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(generic_arg_infer)]

extern crate alloc;

#[macro_use]
pub mod api;

pub mod drivers;
pub mod libs;
pub mod usr;

pub mod arch;