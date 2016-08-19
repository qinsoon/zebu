extern crate gcc;

#[cfg(target_os = "macos")]
#[cfg(target_arch = "x86_64")]
fn main() {
    gcc::compile_library("libruntime.a", &["src/runtime/runtime_x64_macos.c"]); 
}