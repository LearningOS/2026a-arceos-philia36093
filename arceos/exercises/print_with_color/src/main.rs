#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::println;

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    // Print the greeting with an ANSI color escape sequence (red).
    println!("\x1b[31m[WithColor]: Hello, Arceos!\x1b[0m");
}
