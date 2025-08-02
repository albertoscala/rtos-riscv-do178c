fn main() {
    // Rebuild if assembly changes
    println!("cargo:rerun-if-changed=trap.S");

    // Use the `cc` crate to compile trap.S and link it
    cc::Build::new()
        .file("trap.S")
        .flag("-march=rv64imac")
        .compile("trap");
}