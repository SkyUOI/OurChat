//! 命令行控制系统

use crate::{
    db::file_storage,
    share_state::{self},
    ShutdownRev,
};
use colored::Colorize;
use sea_orm::DatabaseConnection;
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
    Set,
    #[strum(ascii_case_insensitive)]
    Get,
    #[strum(ascii_case_insensitive)]
    CleanFS,
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
            InstName::Set => Rc::new(Inst {
                name: InstName::Set,
                name_interbal: "set",
                short_help: "Set variable of the server".to_string(),
                details_help: r#"Set variable of the server.
Usage: set varname value

variables are: Status, AutoCleanCycle, FileSaveDays

Status(The status of server): Maintaining(m) or Normal(n)
AutoCleanCycle(How long will be cleaned): Number of days
FileSaveDays(How long the files will be kept): Number of days"#.to_string(),
                command_process: set_process,
            }),
            InstName::Get => Rc::new(Inst {
                name: InstName::Get,
                name_interbal: "get",
                short_help: "Get variable of the server".to_string(),
                details_help: "Get variable of the server.
Usage: get varname

variables are: Status, AutoCleanCycle, FileSaveDays

Status(The status of server): Maintaining(m) or Normal(n)
AutoCleanCycle(How long will be cleaned): Number of days
FileSaveDays(How long the files will be kept): Number of day".to_string(),
                command_process: get_process,
            }),
            InstName::CleanFS => Rc::new(Inst {
                name: InstName::CleanFS,
                name_interbal: "cleanfs",
                short_help: "Clean the file system".to_string(),
                details_help: "Clean the file system. Usage: cleanfs".to_string(),
                command_process: cleanfs_process,
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

fn cleanfs_process(_: &InstManager, argvs: Vec<String>) -> Result<(), String> {
    if !argvs.is_empty() {
        return Err("cleanfs accept 0 args".to_string());
    }
    Ok(())
}

fn help_process(insts: &InstManager, argvs: Vec<String>) -> Result<(), String> {
    if argvs.is_empty() {
        // 输出宽泛信息
        println!("There are commands supported by console:\n");
        for inst in insts.get_map().borrow().values() {
            println!("{}: {}", inst.name_interbal, inst.short_help);
        }
        println!("\nRefer to \"https://ourchat.readthedocs.io/en/latest/docs/run/server_cmd.html\" for more information");
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

#[derive(strum::EnumString)]
enum Variable {
    #[strum(ascii_case_insensitive)]
    Status,
    #[strum(ascii_case_insensitive)]
    AutoCleanCycle,
    #[strum(ascii_case_insensitive)]
    FileSaveDays,
}

fn gen_error_msg_template(help_msg: &str) -> String {
    format!(
        "Please input right variables,use '{}' for more information",
        help_msg
    )
}

fn set_process(_: &InstManager, argvs: Vec<String>) -> Result<(), String> {
    if argvs.len() != 2 {
        return Err("status accept 2 args".to_string());
    }

    let var = match Variable::from_str(&argvs[0]) {
        Ok(var) => var,
        Err(_) => {
            return Err(gen_error_msg_template("help set"));
        }
    };
    match var {
        Variable::Status => {
            let status = match ServerStatus::from_str(&argvs[1]) {
                Ok(status) => status,
                Err(_) => {
                    return Err(gen_error_msg_template("help set"));
                }
            };
            match status {
                ServerStatus::Maintaining => {
                    if !share_state::get_maintaining() {
                        unsafe { share_state::set_maintaining(true) };
                        println!("Set server status to Maintaining");
                    } else {
                        println!("Server status is already Maintaining");
                    }
                }
                ServerStatus::Normal => {
                    if share_state::get_maintaining() {
                        unsafe { share_state::set_maintaining(false) }
                        println!("Set server status to Normal");
                    } else {
                        println!("Server status is already Normal");
                    }
                }
            }
        }
        Variable::AutoCleanCycle => share_state::set_auto_clean_duration(match argvs[1].parse() {
            Ok(d) => d,
            Err(_) => {
                return Err(format!("Wrong number {}", argvs[1]));
            }
        }),
        Variable::FileSaveDays => share_state::set_file_save_days(match argvs[1].parse() {
            Ok(d) => d,
            Err(_) => {
                return Err(format!("Wrong number {}", argvs[1]));
            }
        }),
    }

    Ok(())
}

fn get_process(_: &InstManager, argvs: Vec<String>) -> Result<(), String> {
    if argvs.len() != 1 {
        return Err("getstatus accept 1 args".to_string());
    }

    let var = match Variable::from_str(&argvs[0]) {
        Ok(var) => var,
        Err(_) => {
            return Err(gen_error_msg_template("help get"));
        }
    };
    match var {
        Variable::Status => {
            if share_state::get_maintaining() {
                println!("Server status is Maintaining");
            } else {
                println!("Server status is Normal");
            }
        }
        Variable::AutoCleanCycle => {
            println!("AutoCleanCycle: {}", share_state::get_auto_clean_duration())
        }
        Variable::FileSaveDays => println!("FileSaveDays: {}", share_state::get_file_save_days()),
    }
    Ok(())
}

pub async fn cmd_process_loop(
    mut shutdown_receiver: ShutdownRev,
    mut db_conn: DatabaseConnection,
) -> anyhow::Result<()> {
    let mut console_reader = BufReader::new(io::stdin()).lines();
    let insts = InstManager::new();
    loop {
        print!(">>> ");
        std::io::stdout().flush()?;
        let command = match console_reader.next_line().await {
            Ok(d) => match d {
                Some(data) => data,
                None => {
                    tracing::info!("Without stdin");
                    shutdown_receiver.recv().await?;
                    String::default()
                }
            },
            Err(e) => {
                tracing::error!("stdin {}", e);
                break;
            }
        };
        let command = command.trim();
        tracing::debug!("cmd: {}", command);
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
                                InstName::CleanFS => {
                                    match file_storage::clean_files(&mut db_conn).await {
                                        Ok(_) => {}
                                        Err(e) => {
                                            tracing::error!("CleanFS: {}", e);
                                        }
                                    }
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
