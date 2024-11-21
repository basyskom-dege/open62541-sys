use std::{
    env, fs, fs::{File, OpenOptions}, path::{Path, PathBuf}
};
use std::io::{self, Write, BufReader, BufRead};

/// Target path in CMake build for include files.
const CMAKE_INCLUDE: &str = "include";
/// Target path in CMake build for lib files.
const CMAKE_LIB: &str = "lib";

/// Name of target library from `open62541` build. This must be `open62541` as it is being generated
/// by the CMake build.
const LIB_BASE: &str = "open62541";
/// Name of library from `extern.c` and `wrapper.c` that holds additional helpers, in particular the
/// compilation of static (inline) functions from `open62541` itself. This may be an arbitrary name;
/// the `cc` build adds it as `rustc-link-lib` automatically.
const LIB_EXT: &str = "open62541-ext";

fn main() {
    let src = env::current_dir().unwrap();

    // Base path for nodesets
    let nodesets_dir = src.join("src/nodesets");

    // Get derived paths relative to `src`.
    let src_open62541 = src.join("open62541");
    let src_wrapper_c = src.join("wrapper.c");
    let src_wrapper_h = src.join("wrapper.h");

    // Parse nodeset files and include directories
    let (c_files, include_dirs, h_files) = parse_nodesets(&nodesets_dir);
    add_includes_to_wrapper(src_wrapper_h.clone(), h_files).expect("Failed to add includes");

    // Rerun build when files in `src` change.
    println!("cargo:rerun-if-changed={}", src_open62541.display());
    println!("cargo:rerun-if-changed={}", src_wrapper_c.display());
    println!("cargo:rerun-if-changed={}", src_wrapper_h.display());

    // Build bundled copy of `open62541` with CMake.
    let mut cmake = cmake::Config::new(src_open62541);
    cmake
        // Use explicit paths here to avoid generating files where we do not expect them below.
        .define("CMAKE_INSTALL_INCLUDEDIR", CMAKE_INCLUDE)
        // Some systems (Fedora) default to `lib64/` instead of `lib/` for 64-bit libraries.
        .define("CMAKE_INSTALL_LIBDIR", CMAKE_LIB)
        // Explicitly set C99 standard to force Windows variants of `vsnprintf()` to conform to this
        // standard. This also matches the expected (or supported) C standard of `open62541` itself.
        .define("C_STANDARD", "99")
        // Python defaults to creating bytecode in `__pycache__` directories. During build, this may
        // happen when the tool `nodeset_compiler` is called. When we package a crate, builds should
        // never modify files outside of `OUT_DIR`, so we disable the cache to prevent this.
        .env("PYTHONDONTWRITEBYTECODE", "1");

    if matches!(env::var("CARGO_CFG_TARGET_ENV"), Ok(env) if env == "musl") {
        let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
        // We require includes from the Linux headers which are not provided automatically when musl
        // is targeted (see https://github.com/open62541/open62541/issues/6360).
        // TODO: Remove this when `open62541` enables us to build without including Linux headers.
        cmake
            .cflag("-idirafter/usr/include")
            .cflag(format!("-idirafter/usr/include/{arch}-linux-gnu"));
    }

    // Full namespace zero is required for the compiled nodesets.
    cmake.define("UA_NAMESPACE_ZERO", "FULL");

    let dst = cmake.build();

    // Get derived paths relative to `dst`.
    let dst_include = dst.join(CMAKE_INCLUDE);
    let dst_lib = dst.join(CMAKE_LIB);

    if matches!(env::var("CARGO_CFG_TARGET_OS"), Ok(os) if os == "windows") {
        // We require the `Iphlpapi` library on Windows builds to avoid errors (regarding the use of
        // `if_nametoindex`, see https://github.com/open62541/open62541/issues/5622).
        println!("cargo:rustc-link-lib=Iphlpapi");
    }

    println!("cargo:rustc-link-search={}", dst_lib.display());
    println!("cargo:rustc-link-lib={LIB_BASE}");

    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Get derived paths relative to `out`.
    let out_bindings_rs = out.join("bindings.rs");
    let out_extern_c = out.join("extern.c");

    let mut builder = bindgen::Builder::default()
        // Include our wrapper functions.
        .allowlist_function("(__)?RS_.*")
        .allowlist_function("(__)?UA_.*")
        // Include the namespace functions of custom nodesets.
        .allowlist_function("namespace_.*")
        // Include our wrapper types.
        .allowlist_type("(__)?RS_.*")
        .allowlist_type("(__)?UA_.*")
        // Include our wrapper vars.
        .allowlist_var("(__)?RS_.*")
        .allowlist_var("(__)?UA_.*")
        // Explicitly set C99 standard to force Windows variants of `vsnprintf()` to conform to this
        // standard. This also matches the expected (or supported) C standard of `open62541` itself.
        .clang_arg("-std=c99")
        .clang_arg(format!("-I{}", dst_include.display()))
        .default_enum_style(bindgen::EnumVariation::NewType {
            is_bitfield: false,
            is_global: false,
        })
        // Use explicit Rust target version that matches or is older than the entry in `Cargo.toml`.
        .rust_target(bindgen::RustTarget::Stable_1_71)
        // Do not derive `Copy` because most of the data types are not copy-safe (they own memory by
        // pointers and need to be cloned manually to duplicate that memory).
        .derive_copy(false)
        // We want to initialize some types statically. This is used in `open62541`, we require that
        // as well to mirror some of the functionality.
        .derive_default(true)
        // The auto-derived comments are not particularly useful because they often do not match the
        // declaration they belong to.
        .generate_comments(false)
        .header(src_wrapper_h.to_str().unwrap())
        // Activate parse callbacks. This causes cargo to invalidate the generated bindings when any
        // of the included files change. It also enables us to rename items in the final bindings.
        .parse_callbacks(Box::new(CustomCallbacks { dst }))
        // We may use `core` instead of `std`. This might be useful for `no_std` environments.
        .use_core()
        // Wrap static functions. These are used in several places for inline helpers and we want to
        // preserve those in the generated bindings. This outputs `extern.c` which we compile below.
        .wrap_static_fns(true)
        // Make sure to specify the location of the resulting `extern.c`. By default `bindgen` would
        // place it in the temporary directory.
        .wrap_static_fns_path(out_extern_c.to_str().unwrap());

    for dir in &include_dirs {
       builder = builder.clang_arg(format!("-I{}", dir.display()));
    }

    let bindings = builder
        .generate()
        .expect("should generate `Bindings` instance");

    bindings
        .write_to_file(out_bindings_rs)
        .expect("should write `bindings.rs`");

    // Build `extern.c` and our custom `wrapper.c` that both hold additional helpers that we want to
    // link in addition to the base `open62541` library.
    let mut build = cc::Build::new();
    build
        .file(out_extern_c)
        .file(src_wrapper_c)
        .include(dst_include)
        // Disable warnings for `open62541`. Not much we can do anyway.
        .warnings(false)
        // Explicitly disable deprecation warnings (seem to be enabled even when other warnings have
        // been disabled above).
        .flag_if_supported("-Wno-deprecated-declarations")
        .flag_if_supported("-Wno-deprecated");
        // Add each .c file and include directory from the parsed nodesets
        for c_file in &c_files {
            build.file(c_file);
        }

        for include_dir in &include_dirs {
            build.include(include_dir);
        }
        // Compile the library
        build.compile(LIB_EXT);
}

/// Parses the nodesets directory and returns vectors for `.c` files, `.h` files,
/// and directories to include.
///
/// # Arguments
/// * `nodesets_dir` - The base path to the nodesets directory
///
/// # Returns
/// A tuple containing:
/// * `Vec<PathBuf>` - Paths to `.c` files
/// * `Vec<PathBuf>` - Paths to directories to include for headers
fn parse_nodesets(nodesets_dir: &Path) -> (Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>) {
    let mut c_files = Vec::new();
    let mut include_dirs = Vec::new();
    let mut h_files = Vec::new();

    // Scan each subdirectory in `nodesets_dir`
    for entry in fs::read_dir(nodesets_dir).expect("Failed to read nodesets directory") {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_dir() {
                let mut has_c_files = false;

                for file in fs::read_dir(&path).expect("Failed to read nodeset subdirectory") {
                    if let Ok(file) = file {
                        let file_path = file.path();
                        if let Some(ext) = file_path.extension() {
                            match ext.to_str().unwrap() {
                                "c" => {
                                    c_files.push(file_path.clone());
                                    println!("cargo:rerun-if-changed={}", file_path.display());
                                    has_c_files = true;
                                }
                                "h" => {
                                    h_files.push(file_path.clone());
                                    println!("cargo:rerun-if-changed={}", file_path.display());
                                }
                                _ => {}
                            }
                        }
                    }
                }

                if has_c_files {
                    include_dirs.push(path.clone());
                }
            }
        }
    }

    (c_files, include_dirs, h_files)
}


/// TODO remove old include statements from previous builds
/// Adds unique include statements to the specified `wrapper.h` file after existing includes.
///
/// # Arguments
///
/// * `wrapper_path` - Path to the existing `wrapper.h` file.
/// * `include_files` - Vector of paths to specific header files to include.
fn add_includes_to_wrapper(wrapper_path: PathBuf, include_files: Vec<PathBuf>) -> io::Result<()> {
    // Read all lines from `wrapper.h` to identify existing includes and preserve other content
    let wrapper_file = File::open(&wrapper_path)?;
    let reader = BufReader::new(wrapper_file);

    // Separate the existing includes and other content
    let mut initial_includes = vec![];
    let mut other_content = vec![];
    let mut found_non_include = false;

    for line in reader.lines() {
        let line = line?;
        // Identify initial includes
        if line.starts_with("#include") && !found_non_include {
            initial_includes.push(line);
        } else {
            found_non_include = true;
            other_content.push(line);
        }
    }

    // Prepare new includes, ensuring no duplicates with the existing ones
    let mut new_includes = vec![];
    for include_file in include_files {
        let include_statement = format!("#include \"{}\"", include_file.display());
        if !initial_includes.contains(&include_statement) && !other_content.contains(&include_statement) {
            new_includes.push(include_statement);
        }
    }

    // Re-write `wrapper.h` with the original includes, followed by new includes, and the rest of the content
    let mut wrapper_file = File::create(&wrapper_path)?;
    for line in &initial_includes {
        writeln!(wrapper_file, "{}", line)?;
    }
    for line in &new_includes {
        writeln!(wrapper_file, "{}", line)?;
    }
    for line in &other_content {
        writeln!(wrapper_file, "{}", line)?;
    }

    Ok(())
}

#[derive(Debug)]
struct CustomCallbacks {
    /// Destination of CMake build of `open62541`.
    dst: PathBuf,
}

impl CustomCallbacks {
    /// Checks if `filename` is inside CMake destination.
    ///
    /// This may be used to ensure that we do not run a rebuild when files generated by CMake change
    /// (it is not necessary to include those files because we already watch the CMake _sources_ and
    /// trigger a rebuild when they change).
    fn inside_dst(&self, filename: &str) -> bool {
        Path::new(filename).starts_with(&self.dst)
    }
}

// Include `cargo:rerun-if` instructions just like `bindgen::CargoCallbacks` does. In addition, make
// necessary adjustments to the names of items for the final bindings.
impl bindgen::callbacks::ParseCallbacks for CustomCallbacks {
    fn header_file(&self, filename: &str) {
        // Make sure to rerun build when dependency outside of `dst/` changes.
        if !self.inside_dst(filename) {
            println!("cargo:rerun-if-changed={filename}");
        }
    }

    fn include_file(&self, filename: &str) {
        // Make sure to rerun build when dependency outside of `dst/` changes.
        if !self.inside_dst(filename) {
            println!("cargo:rerun-if-changed={filename}");
        }
    }

    fn read_env_var(&self, key: &str) {
        // Make sure to rerun build when environment variable changes.
        println!("cargo:rerun-if-env-changed={key}");
    }

    fn item_name(&self, original_item_name: &str) -> Option<String> {
        // Rename our wrapped custom exports to their intended names.
        original_item_name.strip_prefix("RS_").map(str::to_owned)
    }
}
