fn main() {
    // Rebuild if assembly changes
    println!("cargo:rerun-if-changed=trap.S");

    // Compile trap.S with Zicsr enabled
    cc::Build::new()
        .file("trap.S")
        .flag("-march=rv64imac_zicsr")  // ✅ add zicsr extension
        .flag("-mabi=lp64")
        .compile("trap");
}