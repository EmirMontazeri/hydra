// Copyright (C) 2026 Javad Rajabzadeh
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Shared by the build.rs of every crate that ships a Windows binary: embeds a
//! VERSIONINFO block (and optionally the application icon) so each file carries
//! the product name and version SignPath Foundation restricts signing to.
//! Included with `#[path]`, so it compiles into each build script.

use std::path::PathBuf;

pub struct Resource {
    pub description: &'static str,
    pub original_filename: &'static str,
    pub internal_name: &'static str,
    pub icon: bool,
    pub library: bool,
}

const PRODUCT_NAME: &str = "Hydra Download Manager";

pub fn embed(resource: &Resource) {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_manifest = manifest_dir.join("../../Cargo.toml");
    println!("cargo:rerun-if-changed={}", workspace_manifest.display());
    let version = product_version(&std::fs::read_to_string(&workspace_manifest).unwrap());

    let mut nums = version.split('.').map(|p| {
        p.split('-')
            .next()
            .and_then(|n| n.parse::<u16>().ok())
            .unwrap_or(0)
    });
    let (maj, min, pat) = (
        nums.next().unwrap_or(0),
        nums.next().unwrap_or(0),
        nums.next().unwrap_or(0),
    );

    let icon_line = if resource.icon {
        let ico = manifest_dir
            .join("../../scripts/windows/hydra.ico")
            .canonicalize()
            .expect("scripts/windows/hydra.ico not found");
        println!("cargo:rerun-if-changed={}", ico.display());
        // rc string literals treat backslash as an escape; a Windows-host path
        // needs them doubled (the macOS/Linux cross-build path has none).
        format!(
            "1 ICON \"{}\"\n",
            ico.to_string_lossy().replace('\\', "\\\\")
        )
    } else {
        String::new()
    };

    let Resource {
        description,
        original_filename,
        internal_name,
        ..
    } = resource;
    let file_type = if resource.library { "2" } else { "1" };
    let rc = format!(
        r#"{icon_line}1 VERSIONINFO
FILEVERSION {maj},{min},{pat},0
PRODUCTVERSION {maj},{min},{pat},0
FILETYPE {file_type}
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "040904B0"
    BEGIN
      VALUE "ProductName", "{PRODUCT_NAME}"
      VALUE "FileDescription", "{description}"
      VALUE "FileVersion", "{version}"
      VALUE "ProductVersion", "{version}"
      VALUE "CompanyName", "Javad Rajabzadeh"
      VALUE "LegalCopyright", "(C) 2026 Javad Rajabzadeh. GPL-3.0-or-later."
      VALUE "OriginalFilename", "{original_filename}"
      VALUE "InternalName", "{internal_name}"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x409, 1200
  END
END
"#
    );

    let rc_path = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("hydra.rc");
    std::fs::write(&rc_path, rc).unwrap();
    let compiled = if resource.library {
        embed_resource::compile_for_cdylib(&rc_path, embed_resource::NONE)
    } else {
        embed_resource::compile(&rc_path, embed_resource::NONE)
    };
    compiled.manifest_required().unwrap();
}

fn product_version(workspace_manifest: &str) -> String {
    let package = workspace_manifest
        .split("[workspace.package]")
        .nth(1)
        .expect("[workspace.package] missing from the workspace manifest");
    package
        .lines()
        .find_map(|line| {
            let value = line
                .strip_prefix("version")?
                .trim_start()
                .strip_prefix('=')?;
            Some(value.trim().trim_matches('"').to_owned())
        })
        .expect("workspace.package.version missing")
}
