#![allow(unused)]

mod service;
mod storage;
mod model;
pub use service::*;

use colored::Colorize;
use clap::{Parser, ValueEnum};
use chrono::prelude::*;
use std::fs::{self, OpenOptions};
use anyhow::{Result, Error, bail};
use std::result::Result::Err;

pub fn run () -> Result<()> {
  greating();
  action()?;

  Ok(())
}

fn action() -> Result<()> {
  let args = Args::parse();
  args.build()?;

  if let Some(operate) = args.operate {
    if let Some(message) = args.message {
        match operate {
            CommandName::Get => {
              if message == "all" {
                match list_all() {
                  Ok(s) => log(&s),
                  Err(e) => eprint!("List all todos fail: {e}"),
                }
              } else {
                let id = message.parse::<u32>().unwrap();
                match list_by_id(id) {
                  Ok(s) => log(&s),
                  Err(e) => eprint!("List all todos fail: {e}"),
                }
              }
            }
            CommandName::Add => {
                match add_item(&message) {
                  Ok(s) => log(&s),
                  Err(e) => eprint!("Add '{message}' fail: {e} "),
                }
            }
            CommandName::Completed => {
              let id = message.parse::<u32>().unwrap();
              match complete_item(id) {
                Ok(s) => log(&s),
                Err(e) => eprintln!("Complete todo '{id}' fail: {e}"),
              }
            }
            CommandName::Uncompleted => {
              let id = message.parse::<u32>().unwrap();
              match uncomplete_item(id) {
                Ok(s) => log(&s),
                Err(e) => eprintln!("Uncomplete todo '{id}' fail: {e}"),
              }
            }
            CommandName::Restore => {
              let id = message.parse::<u32>().unwrap();
              match restore_item(id) {
                Ok(s) => log(&s),
                Err(e) => eprintln!("Restore todo '{id}' fail: {e}"),
              }
            }
            CommandName::Delete => {
              let id = message.parse::<u32>().unwrap();
              match delete_item(id) {
                Ok(s) => log(&s),
                Err(e) => eprintln!("Delete todo '{id}' fail: {e}"),
              }
            }
            CommandName::Destory => {
              let id = message.parse::<u32>().unwrap();
              match destroy_item(id) {
                Ok(s) => log(&s),
                Err(e) => eprintln!("Destroy todo '{id}' fail: {e}"),
              }
            }
            CommandName::Clear => {
              match clear() {
                Ok(s) => log(&s),
                Err(e) => eprintln!("Clear all todos fail: {e}"),
              };
            }
        }
    }
  }

  Ok(())
}

fn greating() {
  println!(r#"
  ,----,                               ,--,                             
  ,/   .`|                            ,---.'|                             
,`   .'  :                     ____   |   | :                             
;    ;     /   ,--,            ,'  , `. :   : |      ,--,                   
.'___,/    ,'  ,--.'|         ,-+-,.' _ | |   ' :    ,--.'|             ,--,  
|    :     |   |  |,       ,-+-. ;   , || ;   ; '    |  |,            ,'_ /|  
;    |.';  ;   `--'_      ,--.'|'   |  || '   | |__  `--'_       .--. |  | :  
`----'  |  |   ,' ,'|    |   |  ,', |  |, |   | :.'| ,' ,'|    ,'_ /| :  . |  
'   :  ;   '  | |    |   | /  | |--'  '   :    ; '  | |    |  ' | |  . .  
|   |  '   |  | :    |   : |  | ,     |   |  ./  |  | :    |  | ' |  | |  
'   :  |   '  : |__  |   : |  |/      ;   : ;    '  : |__  :  | : ;  ; |  
;   |.'    |  | '.'| |   | |`-'       |   ,/     |  | '.'| '  :  `--'   \ 
'---'      ;  :    ; |   ;/           '---'      ;  :    ; :  ,      .-./ 
           |  ,   /  '---'                       |  ,   /   `--`----'     
            ---`-'                                ---`-'                                                                                                                                                                    
  "#);

  println!("{}", "欢迎使用TODO命令行工具".bold().magenta().italic());
  println!("{}", "功能介绍:".green());
  println!("{} {}", "# 帮助信息:".bold().blue(), "-h/--help");
  println!("{} {}", "# 输入操作指令:".bold().blue(), "-o/--operate <<指令[possible values: get, add, completed, uncompleted, restore, delete, destory, clear]>>");
  println!("{} {}", "# 输入状态指令:".bold().blue(), "-s/--status <<指令[possible values: all, completed, uncompleted, deleted]>>");
  println!("{} {}", "# 操作指令参数:".bold().blue(), "-m/--message <<动作[possible values: all, todo事项的id, 事项描述等]>>");
}


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, value_name = "command-name")]
    pub operate: Option<CommandName>,

    #[arg(short, long, value_name = "command-value")]
    pub message: Option<String>,

    #[arg(short, long, value_name = "command-status")]
    pub status: Option<Status>,
}

impl Args {
    pub fn build(&self) -> Result<()> {
      if let None = self.operate {
        bail!("缺少operate命令")
      }
      if let None = self.message {
        bail!("缺少message参数")
      }

      Ok(())
    }
}

pub fn get_local_time() -> String {
  let local: DateTime<Local> = Local::now();
    let time = format!("{} {}", local.date_naive(), local.time());

    time
}

fn show_message_msg(msg: &str, action: &str) {
    log(&format!("{}【{}】事项", msg, action.blue()));
}

fn log (msg: &str) {
    println!("{} {} {}", get_local_time().green(), "操作记录\n".cyan().italic(), msg)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum CommandName {
    Get,
    Add,
    Completed,
    Uncompleted,
    Restore,
    Delete,
    Destory,
    Clear,
}

enum CommandValue {
    AnyValue(u32),
    SpecialValue(String),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Status {
    All,
    Completed,
    Uncompleted,
    Deleted,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn good_run() {
    assert_eq!(1, 1);
  }
}