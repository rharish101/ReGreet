// SPDX-FileCopyrightText: 2022 The ReGreet Authors
//
// SPDX-License-Identifier: 0BSD

// This file is a helper for auto-updating translations when any translation source is changed.

use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=po");
    println!("cargo:rerun-if-env-changed=TARGET_LOCALE_DIR");

    // Automatically rebuild .po to .mo files while rebuilding the project
    let po_dir = Path::new("po");
    if po_dir.exists() {
        // Create target directory under `$OUT_DIR/target`. At packaging stage it should be installed
        // as $DATAROOTDIR/share/locale/$LANG/regreet.mo.
        let target_locale_var = env::var("TARGET_LOCALE_DIR")
            .unwrap_or_else(|_| format!("{}/locale", env::var("OUT_DIR").unwrap()));

        let out_locale_dir = Path::new(&target_locale_var);

        if let Ok(entries) = fs::read_dir(po_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                // Find all the files that contains a '.po' suffix
                if path.extension().and_then(|s| s.to_str()) == Some("po") {
                    let lang = path.file_stem().unwrap().to_str().unwrap();

                    // Path to save to: target/locale/$LANG/LC_MESSAGES/regreet.mo
                    let mo_dir = out_locale_dir.join(lang).join("LC_MESSAGES");
                    fs::create_dir_all(&mo_dir).expect("Cannot create locale directories");
                    let mo_path = mo_dir.join("regreet.mo");

                    // Call 'msgfmt' to compile translation
                    let status = Command::new("msgfmt")
                        .arg(&path)
                        .arg("-o")
                        .arg(&mo_path)
                        .status();

                    match status {
                        Ok(s) if s.success() => {
                            println!(
                                "cargo:warning=Translation {} rebuilt successfully",
                                path.display(),
                            );
                        }
                        Ok(s) => {
                            panic!(
                                "Failed to compile translation {}. 'msgfmt' exited with status: {}.",
                                path.display(),
                                s
                            );
                        }
                        Err(e) => {
                            println!(
                                "cargo:warning=Unable to execute 'msgfmt' for {}: {}. Check that the 'gettext' package is installed.",
                                path.display(),
                                e
                            );
                        }
                    }
                }
            }
        }
    }
}
