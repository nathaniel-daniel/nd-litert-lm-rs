use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

const URL: &str = "https://github.com/google-ai-edge/LiteRT-LM/releases/download/v0.19.1/litert_lm_c_api-1.0.0.zip";
const ZIP_FILE_NAME: &str = "litert_lm_c_api-1.0.0.zip";
const UNZIPPED_FILE_NAME: &str = "litert_lm_c_api-1.0.0";

fn main() {
    let out_dir = std::env::var_os("OUT_DIR").expect("Missing OUT_DIR");
    let out_dir = PathBuf::from(out_dir);

    let zip_path = out_dir.join(ZIP_FILE_NAME);

    if !zip_path
        .try_exists()
        .expect("Failed to check if zip path exists")
    {
        let mut reader = ureq::get(URL)
            .call()
            .expect("Failed to make request")
            .into_body()
            .into_reader();

        let temp_zip_path = zip_path.with_added_extension("temp");
        let mut file = File::create(&temp_zip_path).expect("Failed to open temp zip file");
        std::io::copy(&mut reader, &mut file).expect("Failed to copy");
        file.flush().expect("Failed to flush");
        file.sync_all().expect("Failed to sync all");

        std::fs::rename(&temp_zip_path, &zip_path).expect("Failed to rename");

        drop(file);
    }

    let unzipped_path = out_dir.join(UNZIPPED_FILE_NAME);
    if !unzipped_path
        .try_exists()
        .expect("Failed to check if unzipped path exists")
    {
        let temp_unzipped_path = unzipped_path.with_added_extension("temp");

        let mut file = File::open(&zip_path).expect("Failed to open zip file");
        {
            let mut zip = zip::ZipArchive::new(&mut file).expect("Failed to reat zip file");
            zip.extract(&temp_unzipped_path).expect("Failed to extract");
        }
        drop(file);

        std::fs::rename(&temp_unzipped_path, &unzipped_path)
            .expect("Failed to rename temp unzipped path");
    }

    let target = std::env::var("TARGET").expect("Missing TARGET");
    let lib_path = unzipped_path.join("lib");
    let dll_path = match target.as_str() {
        "x86_64-pc-windows-msvc" => {
            let windows_folder = lib_path.join("windows_x86_64");
            let lib_dir = windows_folder.join("lib");
            let dll_path = windows_folder.join("bin").join("litert-lm.dll");

            println!("cargo::rustc-link-search={}", lib_dir.display());

            dll_path
        }
        "aarch64-unknown-linux-gnu" => {
            let dir = lib_path.join("linux_arm64");
            let dll_path = dir.join("liblitert-lm.so");

            println!("cargo::rustc-link-search=native={}", dir.display());

            dll_path
        }
        "x86_64-unknown-linux-gnu" => {
            let dir = lib_path.join("linux_x86_64");
            let dll_path = dir.join("liblitert-lm.so");

            println!("cargo::rustc-link-search=native={}", dir.display());

            dll_path
        }
        _ => panic!("Unsupported target \"{target}\""),
    };

    println!("cargo::rustc-link-lib=litert-lm");

    let dll_name = dll_path.file_name().expect("Missing file name");
    let target_dir = out_dir
        .ancestors()
        .nth(4)
        .expect("Failed to find target profile directory");
    let out_dll_path = target_dir.join(dll_name);

    if !out_dll_path
        .try_exists()
        .expect("Failed to check if dll path exists")
    {
        #[expect(clippy::collapsible_if)]
        if let Err(error) = std::fs::copy(&dll_path, &out_dll_path) {
            panic!(
                "Failed to copy dll from \"{}\" to \"{}\": {}",
                dll_path.display(),
                out_dll_path.display(),
                error
            );
        }
    }
    println!("cargo::rerun-if-changed={}", out_dll_path.display());
}
