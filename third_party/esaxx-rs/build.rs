#[cfg(feature = "cpp")]
#[cfg(not(target_os = "macos"))]
fn main() {
    cc::Build::new()
        .cpp(true)
        .flag("-std=c++11")
        // .static_crt(true)  // Removed to use /MD (dynamic CRT) instead of /MT (static CRT)
        // This fixes the linker error: RuntimeLibrary mismatch between MD_DynamicRelease and MT_StaticRelease
        .file("src/esaxx.cpp")
        .include("src")
        .compile("esaxx");
}

#[cfg(feature = "cpp")]
#[cfg(target_os = "macos")]
fn main() {
    cc::Build::new()
        .cpp(true)
        .flag("-std=c++11")
        .flag("-stdlib=libc++")
        // .static_crt(true)  // Removed to use /MD (dynamic CRT) instead of /MT (static CRT)
        // This fixes the linker error: RuntimeLibrary mismatch between MD_DynamicRelease and MT_StaticRelease
        .file("src/esaxx.cpp")
        .include("src")
        .compile("esaxx");
}

#[cfg(not(feature = "cpp"))]
fn main() {}
