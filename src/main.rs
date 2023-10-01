// SPDX-License-Identifier: GPL-3.0-only
// 🄯 2023, Alexey Parfenov <zxed@alkatrazstudio.net>

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    // all
    clippy::needless_return,

    // pedantic
    clippy::struct_excessive_bools,
    clippy::module_name_repetitions,
    clippy::too_many_lines,
    clippy::redundant_closure,
    clippy::redundant_closure_for_method_calls,
    clippy::similar_names,

    // nursery
    clippy::cognitive_complexity,
    clippy::option_if_let_else,
)]

mod concurrent_map;
mod meta;
mod pics;
mod args;
mod cue;
mod convert;
mod files;
mod formats;
mod entry;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    return entry::main();
}
