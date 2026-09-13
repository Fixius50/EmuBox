fn main() {
    println!("cargo:rerun-if-changed=../solid/dist");
    tauri_build::build();
}
