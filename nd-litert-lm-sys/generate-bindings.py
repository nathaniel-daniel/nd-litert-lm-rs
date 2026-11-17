import requests
import os
import zipfile
import subprocess

URL = "https://github.com/google-ai-edge/LiteRT-LM/releases/download/v0.19.1/litert_lm_c_api-1.0.0.zip"
ZIP_FILE_NAME = "litert_lm_c_api-1.0.0.zip"
UNZIPPED_FILE_NAME = "litert_lm_c_api-1.0.0"
TEMP = "temp"


def try_download_zip(zip_path: str) -> None:
    if not os.path.exists(zip_path):
        print("Downloading...")
        with requests.get(URL, stream=True) as response:
            response.raise_for_status()

            with open(zip_path, "wb") as file:
                for chunk in response.iter_content(chunk_size=8192):
                    if chunk:
                        file.write(chunk)


def try_unzip_zip(zip_path: str, unzipped_path: str) -> None:
    if not os.path.exists(unzipped_path):
        print("Unzipping...")
        with zipfile.ZipFile(zip_path, "r") as zip:
            zip.extractall(unzipped_path)


def main() -> None:
    os.makedirs(TEMP, exist_ok=True)

    zip_path = os.path.join(TEMP, ZIP_FILE_NAME)
    try_download_zip(zip_path)

    unzipped_path = os.path.join(TEMP, UNZIPPED_FILE_NAME)
    try_unzip_zip(zip_path, unzipped_path)

    include_path = os.path.join(unzipped_path, "include")

    subprocess.run(
        [
            "bindgen",
            "bindings.h",
            "--allowlist-function",
            "litert_lm_.*",
            "--newtype-enum",
            "LiteRtLmStatusCode",
            "--newtype-enum",
            "LiteRtLmLogSeverity",
            "--override-abi",
            "litert_lm_.*=C-unwind",
            "-o",
            "src/bindings.rs",
            "--",
            f"-I{include_path}",
        ],
        check=True,
    )


if __name__ == "__main__":
    main()
