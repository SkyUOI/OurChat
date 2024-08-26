//! 命令行控制系统

use crate::{ShutdownRev, MAINTAINING};
use colored::Colorize;
use static_keys::static_branch_unlikely;
use std::{cell::RefCell, collections::BTreeMap, io::Write, rc::Rc, str::FromStr};
use tokio::io::{self, AsyncBufReadExt, BufReader};

type CheckFunc = fn(&InstManager, Vec<String>) -> Result<(), String>;

/// 储存一个指令的信息
struct Inst {
    name: InstName,
    name_interbal: &'static str,
    short_help: String,
    details_help: String,
    pub command_process: CheckFunc,
}

#[derive(strum::EnumString, PartialEq, Eq, PartialOrd, Ord)]
enum InstName {
    #[strum(ascii_case_insensitive)]
    Help,
    #[strum(ascii_case_insensitive)]
    Exit,
    #[strum(ascii_case_insensitive)]
    SetStatus,
    #[strum(ascii_case_insensitive)]
    GetStatus,
}

struct InstManager {
    // 使用Map，因为需要稳定的输出
    insts: Rc<RefCell<BTreeMap<InstName, Rc<Inst>>>>,
}

impl InstManager {
    fn new() -> Self {
        let insts = Rc::new(RefCell::new(collection_literals::collection! {
            InstName::Exit => Rc::new(Inst {
                name: InstName::Exit,
                name_interbal: "exit",
                short_help: "Exit the server".to_string(),
                details_help: "Exit the server.Usage: exit".to_string(),
                command_process: exit_process,
            }),
            InstName::Help => Rc::new(Inst {
                name: InstName::Help,
                name_interbal: "help",
                short_help: "Display the Help information".to_string(),
                details_help: "Displap the help information Help.Usage: help command1 command2".to_string(),
                command_process: help_process,
            }),
            InstName::SetStatus => Rc::new(Inst {
                name: InstName::SetStatus,
                name_interbal: "setstatus",
                short_help: "Set the status of the server".to_string(),
                details_help: "Set the status of the server.Usage: status maintaining(m)|normal(n)".to_string(),
                command_process: set_status_process,
            }),
            InstName::GetStatus => Rc::new(Inst {
                name: InstName::GetStatus,
                name_interbal: "getstatus",
                short_help: "Get the status of the server".to_string(),
                details_help: "Get the status of the server.Usage: getstatus".to_string(),
                command_process: get_status_process,
            }),
        }));
        Self { insts }
    }

    fn get_inst(&self, name: &InstName) -> Option<Rc<Inst>> {
        self.insts.borrow().get(name).cloned()
    }

    fn get_map(&self) -> Rc<RefCell<BTreeMap<InstName, Rc<Inst>>>> {
        self.insts.clone()
    }
}

fn exit_process(_: &InstManager, argvs: Vec<String>) -> Result<(), String> {
    if argvs.is_empty() {
        tracing::info!("Exiting now...");
        return Ok(());
    }
    Err("exit accept 0 args".to_string())
}

fn help_process(insts: &InstManager, argvs: Vec<String>) -> Result<(), String> {
    if argvs.is_empty() {
        // 输出宽泛信息
        println!("There are commands supported by console:\n");
        for inst in insts.get_map().borrow().values() {
            println!("{}: {}", inst.name_interbal, inst.short_help);
        }
    } else {
        // 针对给定的参数输出帮助信息
        for name in argvs {
            match InstName::from_str(&name) {
                Ok(inst) => {
                    if let Some(inst) = insts.get_inst(&inst) {
                        println!("{}: {}", inst.name_interbal, inst.details_help);
                    }
                }
                Err(_) => {
                    println!("{}{}", "ERROR:{}: Unknown command".red(), name.red());
                }
            }
        }
    }
    Ok(())
}

/// 服务器状态
#[derive(strum::EnumString)]
enum ServerStatus {
    #[strum(ascii_case_insensitive, serialize = "m")]
    Maintaining,
    #[strum(ascii_case_insensitive, serialize = "n")]
    Normal,
}

fn set_status_process(_: &InstManager, argvs: Vec<String>) -> Result<(), String> {
    if argvs.len() != 1 {
        return Err("status accept 1 args".to_string());
    }
    let status = match ServerStatus::from_str(&argvs[0]) {
        Ok(status) => status,
        Err(_) => {
            return Err("status accept maintaining(m)|normal(n)".to_string());
        }
    };
    match status {
        ServerStatus::Maintaining => {
            if !static_branch_unlikely!(MAINTAINING) {
                unsafe { MAINTAINING.enable() }
                println!("Set server status to Maintaining");
            } else {
                println!("Server status is already Maintaining");
            }
        }
        ServerStatus::Normal => {
            if static_branch_unlikely!(MAINTAINING) {
                unsafe { MAINTAINING.disable() }
                println!("Set server status to Normal");
            } else {
                println!("Server status is already Normal");
            }
        }
    }
    Ok(())
}

fn get_status_process(_: &InstManager, argvs: Vec<String>) -> Result<(), String> {
    if !argvs.is_empty() {
        return Err("getstatus accept 0 args".to_string());
    }
    if static_branch_unlikely!(MAINTAINING) {
        println!("Server status is Maintaining");
    } else {
        println!("Server status is Normal");
    }
    Ok(())
}

pub async fn cmd_process_loop(mut shutdown_receiver: ShutdownRev) -> anyhow::Result<()> {
    let mut console_reader = BufReader::new(io::stdin()).lines();
    let insts = InstManager::new();
    loop {
        print!(">>> ");
        std::io::stdout().flush().unwrap();
        let command = match console_reader.next_line().await {
            Ok(d) => match d {
                Some(data) => data,
                None => {
                    tracing::info!("Without stdin");
                    shutdown_receiver.recv().await.unwrap();
                    String::default()
                }
            },
            Err(e) => {
                tracing::error!("stdin {}", e);
                break;
            }
        };
        let command = command.trim();
        let mut command = command.split_whitespace();
        let command_name = match command.next().to_owned() {
            Some(name) => name,
            None => continue,
        };
        match InstName::from_str(command_name) {
            Ok(inst_enum) => {
                let command_list = command.map(|d| d.to_owned()).collect();
                if let Some(inst) = insts.get_inst(&inst_enum) {
                    match (inst.command_process)(&insts, command_list) {
                        Ok(_) => {
                            // 指令运行成功，运行接下来的操作
                            match inst_enum {
                                InstName::Exit => {
                                    return Ok(());
                                }
                                _ => {}
                            }
                        }
                        Err(e) => {
                            println!("{}: {}", command_name, e);
                        }
                    }
                }
            }
            Err(_e) => {
                println!("{}{}", command_name.red(), ": Unknown command".red());
            }
        };
    }
    Ok(())
}
