const RESOURCE_FILE: &str = "../../assets/windows/rustpaper.rc";

fn main() {
    println!("cargo:rerun-if-changed={RESOURCE_FILE}");
    println!("cargo:rerun-if-changed=../../assets/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    embed_resource::compile(RESOURCE_FILE, embed_resource::NONE)
        .manifest_optional()
        .expect("compilation des ressources Windows");
}
