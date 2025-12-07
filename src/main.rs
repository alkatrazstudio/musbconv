// SPDX-License-Identifier: GPL-3.0-only
// 🄯 2023, Alexey Parfenov <zxed@alkatrazstudio.net>

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    // all
    clippy::needless_return,

    // pedantic
    clippy::module_name_repetitions,
    clippy::redundant_closure,
    clippy::redundant_closure_for_method_calls,
    clippy::similar_names,
    clippy::struct_excessive_bools,
    clippy::too_many_lines,

    // nursery
    clippy::cognitive_complexity,
    clippy::option_if_let_else,
)]

mod args;
mod concurrent_map;
mod convert;
mod cue;
mod entry;
mod files;
mod formats;
mod meta;
mod pics;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    return entry::main();
}
