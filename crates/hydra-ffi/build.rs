// Copyright (C) 2026 Javad Rajabzadeh
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Embeds the Windows VERSIONINFO block into hydra.dll.

#[path = "../../scripts/windows/version_resource.rs"]
mod version_resource;

fn main() {
    version_resource::embed(&version_resource::Resource {
        description: "Hydra download engine library",
        original_filename: "hydra.dll",
        internal_name: "hydra",
        icon: false,
        library: true,
    });
}
