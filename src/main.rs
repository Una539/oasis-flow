// Copyright (c) 2026
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use clap::Parser;
use std::error::Error;
// 使用你的项目名来引入 lib 中的逻辑
use oasis_flow::run;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long)]
    add: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let file_path = "todos.toml";

    // 仅仅调用 lib.rs 暴露的接口
    run(file_path, args.add)?;

    Ok(())
}
