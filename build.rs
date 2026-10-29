// SPDX-FileCopyrightText: 2022 The ReGreet Authors
//
// SPDX-License-Identifier: 0BSD

// This file is a helper for auto-updating translations when any translation source is changed.

use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=po");

    // Automatically rebuild .po to .mo files while rebuilding the project
    let po_dir = Path::new("po");
    if po_dir.exists() {
        // Create target directory under target/. At packaging stage it should be installed
        // as $DATAROOTDIR/share/locale/$LANG/regreet.mo.
        let out_locale_dir = Path::new("target/locale");

        if let Ok(entries) = fs::read_dir(po_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                // Find all the files that contains a '.po' suffix (i.e., ru.po)
                if path.extension().and_then(|s| s.to_str()) == Some("po") {
                    let lang = path.file_stem().unwrap().to_str().unwrap();

                    // Path to save to: target/locale/ru/LC_MESSAGES/regreet.mo
                    let mo_dir = out_locale_dir.join(lang).join("LC_MESSAGES");
                    fs::create_dir_all(&mo_dir).unwrap();
                    let mo_path = mo_dir.join("regreet.mo");

                    // Call 'msgfmt' to complie translation
                    let status = Command::new("msgfmt")
                        .arg(&path)
                        .arg("-o")
                        .arg(&mo_path)
                        .status();

                    if let Err(e) = status {
                        println!(
                            "cargo:warning=Unable to complie translation {}: {}. Check that the 'gettext' package istalled.",
                            path.display(),
                            e
                        );
                    }
                }
            }
        }
    }
}
