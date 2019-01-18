#![feature(box_patterns)]
#![feature(if_while_or_patterns)]
#![feature(repeat_generic_slice)]

#[macro_use]
pub mod vm;
#[macro_use]
pub mod util;
pub mod builtin;
pub mod builtins;
pub mod bytecode_gen;
pub mod gc;
pub mod id;
pub mod jit;
pub mod lexer;
pub mod node;
pub mod parser;
pub mod test;
pub mod token;
pub mod vm_codegen;

extern crate ansi_term;
extern crate chrono;
extern crate encoding;
extern crate libc;
extern crate libloading;
extern crate llvm_sys as llvm;
extern crate nix;
extern crate rand;
extern crate rustc_hash;
extern crate rustyline;
// extern crate cpuprofiler;
