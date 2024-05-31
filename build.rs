// SPDX-FileCopyrightText: 2021 - 2024 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::env;
use std::fs;
use std::path::Path;

const LICENSES_CACHE_FILE: &str = "resources/licenses-cache.bin.zstd";
const LICENSES_CACHE_URL: &str =
    "https://github.com/o2sh/onefetch/raw/main/resources/license.cache.zstd";

fn download_licenses_cache() -> Result<(), Box<dyn std::error::Error>> {
    let cache_file = Path::new(&env::var("OUT_DIR")?).join(LICENSES_CACHE_FILE);
    if !cache_file.exists() {
        fs::create_dir_all(cache_file.parent().unwrap())?;
        let url = reqwest::Url::parse(LICENSES_CACHE_URL)?;
        let content = reqwest::blocking::get(url)?.bytes()?;
        fs::write(cache_file, content)?;
    }
    println!("cargo:rerun-if-env-changed=OUT_DIR");
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // NOTE As <https://docs.rs> does not allow the build process to use the network,
    //      we have to disable downloading the licenses.
    if std::env::var("DOCS_RS").is_ok() {
        Ok(())
    } else {
        download_licenses_cache()
    }
}
