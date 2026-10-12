// Copyright (C) 2026 Javad Rajabzadeh
// SPDX-License-Identifier: GPL-3.0-or-later

//! Embeds the Windows VERSIONINFO block into hydra.exe.

#[path = "../../scripts/windows/version_resource.rs"]
mod version_resource;

fn main() {
    version_resource::embed(&version_resource::Resource {
        description: "Hydra command-line interface",
        original_filename: "hydra.exe",
        internal_name: "hydra",
        icon: false,
        library: false,
    });
}
