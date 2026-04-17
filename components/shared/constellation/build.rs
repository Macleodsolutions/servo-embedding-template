fn main() {
    let out = std::env::var("OUT_DIR").unwrap();
    embedder_bridge_build::run(&out);
}
