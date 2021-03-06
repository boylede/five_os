use std::{error::Error, path::PathBuf};

use cc::Build;

fn main() -> Result<(), Box<dyn Error>> {
    let asm_dir = PathBuf::from(r"./src/assembly");

    println!("cargo:rustc-link-search=linker/");
    let assembly_files = vec!["boot.riscv", "trap.riscv", "cpu.riscv"];

    let mut builder = Build::new();

    for file in assembly_files.iter() {
        let filename = asm_dir.join(file);
        builder.file(filename);
        // println!("cargo:rerun-if-changed={}", filename);
    }

    builder.compile("asm");

    Ok(())
}
