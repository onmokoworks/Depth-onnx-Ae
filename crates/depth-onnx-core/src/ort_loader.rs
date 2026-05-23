use crate::engine::EngineError;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static ORT_READY: OnceLock<Result<(), String>> = OnceLock::new();

pub fn ensure_ort() -> Result<(), EngineError> {
    ORT_READY
        .get_or_init(|| init_ort().map_err(|e| e.to_string()))
        .clone()
        .map_err(EngineError::SessionInit)
}

fn init_ort() -> Result<(), EngineError> {
    let path = resolve_ort_dylib().ok_or_else(|| {
        EngineError::SessionInit(
            "ONNX Runtime dylib not found (expected Contents/Frameworks/libonnxruntime.1.24.4.dylib)"
                .into(),
        )
    })?;

    ort::init_from(&path)
        .map_err(|e| EngineError::SessionInit(format!("init_from failed: {e}")))?
        .with_name("DepthONNX")
        .commit();

    Ok(())
}

fn resolve_ort_dylib() -> Option<PathBuf> {
    if let Some(path) = ort_dylib_next_to_plugin() {
        if path.is_file() {
            return Some(path);
        }
    }

    if let Ok(path) = std::env::var("ORT_DYLIB_PATH") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Some(path);
        }
    }

    None
}

#[cfg(target_os = "macos")]
fn ort_dylib_next_to_plugin() -> Option<PathBuf> {
    use std::ffi::CStr;

    extern "C" {
        fn dladdr(addr: *const std::ffi::c_void, info: *mut DlInfo) -> i32;
    }

    #[repr(C)]
    struct DlInfo {
        dli_fname: *const i8,
        dli_fbase: *mut std::ffi::c_void,
        dli_sname: *const i8,
        dli_saddr: *mut std::ffi::c_void,
    }

    extern "C" fn plugin_anchor() {}

    let mut info = DlInfo {
        dli_fname: std::ptr::null(),
        dli_fbase: std::ptr::null_mut(),
        dli_sname: std::ptr::null(),
        dli_saddr: std::ptr::null_mut(),
    };

    let ok = unsafe {
        dladdr(
            plugin_anchor as extern "C" fn() as *const std::ffi::c_void,
            &mut info,
        )
    };
    if ok == 0 || info.dli_fname.is_null() {
        return None;
    }

    let plugin_path = unsafe { CStr::from_ptr(info.dli_fname) }.to_str().ok()?;
    let macos_dir = Path::new(plugin_path).parent()?;
    let candidate = macos_dir.join("../Frameworks/libonnxruntime.1.24.4.dylib");
    candidate.canonicalize().ok().or(Some(candidate))
}

#[cfg(not(target_os = "macos"))]
fn ort_dylib_next_to_plugin() -> Option<PathBuf> {
    None
}
