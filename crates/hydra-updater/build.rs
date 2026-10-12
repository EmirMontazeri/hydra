// Copyright (C) 2026 Javad Rajabzadeh
// SPDX-License-Identifier: GPL-3.0-or-later

//! Embeds the Windows VERSIONINFO block into hydra-updater.exe.

#[path = "../../scripts/windows/version_resource.rs"]
mod version_resource;

fn main() {
    version_resource::embed(&version_resource::Resource {
        description: "Hydra update finisher",
        original_filename: "hydra-updater.exe",
        internal_name: "hydra-updater",
        icon: false,
        library: false,
    });
}
