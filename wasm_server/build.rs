fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .format(true)
        // .out_dir("src")
        .compile(&["proto/wasm_rpc.proto"], &["proto"])?;
    Ok(())
}
