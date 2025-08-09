fn main() {
    // Rebuild if assembly changes
    println!("cargo:rerun-if-changed=trap.S");
    println!("cargo:rerun-if-changed=boot.S");

    // Compile trap.S
    cc::Build::new()
        .file("trap.S")
        .file("boot.S")                   // ✅ compile the boot helper
        .flag("-march=rv64imac_zicsr")    // CSR instructions
        .flag("-mabi=lp64")               // ABI for RV64
        .compile("asm");                  // output name (merged)
}