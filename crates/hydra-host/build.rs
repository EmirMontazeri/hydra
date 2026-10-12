// Copyright (C) 2026 Javad Rajabzadeh
// SPDX-License-Identifier: GPL-3.0-or-later

//! Embeds the Windows VERSIONINFO block into hydra-host.exe.

#[path = "../../scripts/windows/version_resource.rs"]
mod version_resource;

fn main() {
    version_resource::embed(&version_resource::Resource {
        description: "Hydra browser native-messaging host",
        original_filename: "hydra-host.exe",
        internal_name: "hydra-host",
        icon: false,
        library: false,
    });
}
