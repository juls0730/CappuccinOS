#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(const_unsafecell_get_mut)]
#![feature(const_mut_refs)]

extern crate alloc;

#[macro_use]
pub mod api;

pub mod drivers;
pub mod libs;
pub mod usr;

pub mod arch;