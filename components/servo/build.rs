fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    embedder_bridge_build::run(&out_dir);
}
