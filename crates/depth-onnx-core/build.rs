fn main() {
    let manifest_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let ort_dylib = manifest_dir
        .join("../../third_party/onnxruntime-osx-arm64/lib/libonnxruntime.1.24.4.dylib");
    if ort_dylib.exists() {
        println!("cargo:rustc-env=ORT_DYLIB_PATH={}", ort_dylib.display());
    }

    let model_root = manifest_dir.join("../../model");
    if model_root.is_dir() {
        println!(
            "cargo:rustc-env=AE_DEPTH_ONNX_DEV_MODEL_DIR={}",
            model_root.display()
        );
    }
}
