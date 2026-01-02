fn main() {
    // Rebuild if any assembly changes
    println!("cargo:rerun-if-changed=asm/trap.S");
    println!("cargo:rerun-if-changed=asm/boot.S");
    // (nice to also watch the whole dir in case you add files)
    println!("cargo:rerun-if-changed=asm");

    // Compile trap.S and boot.S from the asm/ folder
    let mut build = cc::Build::new();
    build
        .files(["asm/trap.S", "asm/boot.S"])
        .flag("-march=rv64imac_zicsr")  // CSR + RV64IMAC
        .flag("-mabi=lp64")             // 64-bit ABI
        .compile("riscv_asm");          // static lib name (libriscv_asm.a)

    // If you ever add headers for macros in asm/, uncomment:
    // build.include("asm");
}