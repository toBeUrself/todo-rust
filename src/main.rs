#![allow(unused)]

use std::process;

use anyhow::{Result, Ok};
use todo::run;

fn main() -> Result<()> {
    if let Err(e) = run() {
        println!("程序运行出错：{e}");
        process::exit(1);
    }

    Ok(())
}
