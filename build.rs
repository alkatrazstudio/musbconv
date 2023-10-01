// SPDX-License-Identifier: GPL-3.0-only
// 🄯 2021, Alexey Parfenov <zxed@alkatrazstudio.net>

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    // all
    clippy::needless_return,
)]

fn main() -> std::io::Result<()> {
    return built::write_built_file();
}
