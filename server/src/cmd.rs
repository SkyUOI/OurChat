//! 命令行控制系统
//! TODO:add command for set friends limit

use crate::{
    ShutdownRev,
    connection::WS,
    db::file_storage,
    shared_state::{self},
};
use colored::Colorize;
use parking_lot::Mutex;
use sea_orm::DatabaseConnection;
use std::{collections::BTreeMap, io::Write, str::FromStr, sync::Arc};
use tokio::{
    io::{self, AsyncBufReadExt, BufReader},
    net::{TcpListener, TcpStream},
    select,
    sync::{mpsc, oneshot},
};

type CheckFunc = fn(&InstManager, Vec<String>) -> Result<Option<String>, String>;

/// 储存一个指令的信息
struct Inst {
    _name: InstName,
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
    insts: Arc<Mutex<BTreeMap<InstName, Arc<Inst>>>>,
}

impl InstManager {
    fn new() -> Self {
        let insts = Arc::new(Mutex::new(collection_literals::collection! {
            InstName::Exit => Arc::new(Inst {
                _name: InstName::Exit,
                name_interbal: "exit",
                short_help: "Exit the server".to_string(),
                details_help: "Exit the server.Usage: exit".to_string(),
                command_process: exit_process,
            }),
            InstName::Help => Arc::new(Inst {
                _name: InstName::Help,
                name_interbal: "help",
                short_help: "Display the Help information".to_string(),
                details_help: "Displap the help information Help.Usage: help command1 command2".to_string(),
                command_process: help_process,
            }),
            InstName::Set => Arc::new(Inst {
                _name: InstName::Set,
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
            InstName::Get => Arc::new(Inst {
                _name: InstName::Get,
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
            InstName::CleanFS => Arc::new(Inst {
                _name: InstName::CleanFS,
                name_interbal: "cleanfs",
                short_help: "Clean the file system".to_string(),
                details_help: "Clean the file system. Usage: cleanfs".to_string(),
                command_process: cleanfs_process,
            }),

        }));
        Self { insts }
    }

    fn get_inst(&self, name: &InstName) -> Option<Arc<Inst>> {
        self.insts.lock().get(name).cloned()
    }

    fn get_map(&self) -> Arc<Mutex<BTreeMap<InstName, Arc<Inst>>>> {
        self.insts.clone()
    }
}

fn exit_process(_: &InstManager, argvs: Vec<String>) -> Result<Option<String>, String> {
    if argvs.is_empty() {
        tracing::info!("Exiting now...");
        return Ok(None);
    }
    Err("exit accept 0 args".to_string())
}

fn cleanfs_process(_: &InstManager, argvs: Vec<String>) -> Result<Option<String>, String> {
    if !argvs.is_empty() {
        return Err("cleanfs accept 0 args".to_string());
    }
    Ok(None)
}

fn help_process(insts: &InstManager, argvs: Vec<String>) -> Result<Option<String>, String> {
    let mut ret = String::new();
    if argvs.is_empty() {
        // 输出宽泛信息
        ret.push_str("There are commands supported by console:\n\n");
        for inst in insts.get_map().lock().values() {
            ret.push_str(&format!("{}: {}\n", inst.name_interbal, inst.short_help));
        }
        ret.push_str(&"\nRefer to \"https://ourchat.readthedocs.io/en/latest/docs/run/server_cmd.html\" for more information\n".to_string());
    } else {
        // 针对给定的参数输出帮助信息
        for name in argvs {
            match InstName::from_str(&name) {
                Ok(inst) => {
                    if let Some(inst) = insts.get_inst(&inst) {
                        ret.push_str(&format!("{}: {}\n", inst.name_interbal, inst.details_help));
                    }
                }
                Err(_) => {
                    ret.push_str(&format!(
                        "{}{}\n",
                        "ERROR:{}: Unknown command".red(),
                        name.red()
                    ));
                }
            }
        }
    }
    Ok(Some(ret))
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

fn set_process(_: &InstManager, argvs: Vec<String>) -> Result<Option<String>, String> {
    if argvs.len() != 2 {
        return Err("status accept 2 args".to_string());
    }

    let var = match Variable::from_str(&argvs[0]) {
        Ok(var) => var,
        Err(_) => {
            return Err(gen_error_msg_template("help set"));
        }
    };
    let mut ret = String::new();
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
                    if !shared_state::get_maintaining() {
                        shared_state::set_maintaining(true);
                        ret.push_str("Set server status to Maintaining");
                    } else {
                        ret.push_str("Server status is already Maintaining");
                    }
                }
                ServerStatus::Normal => {
                    if shared_state::get_maintaining() {
                        shared_state::set_maintaining(false);
                        ret.push_str("Set server status to Normal");
                    } else {
                        ret.push_str("Server status is already Normal");
                    }
                }
            }
        }
        Variable::AutoCleanCycle => shared_state::set_auto_clean_duration(match argvs[1].parse() {
            Ok(d) => d,
            Err(_) => {
                return Err(format!("Wrong number {}", argvs[1]));
            }
        }),
        Variable::FileSaveDays => shared_state::set_file_save_days(match argvs[1].parse() {
            Ok(d) => d,
            Err(_) => {
                return Err(format!("Wrong number {}", argvs[1]));
            }
        }),
    }
    Ok(Some(ret))
}

fn get_process(_: &InstManager, argvs: Vec<String>) -> Result<Option<String>, String> {
    if argvs.len() != 1 {
        return Err("getstatus accept 1 args".to_string());
    }

    let var = match Variable::from_str(&argvs[0]) {
        Ok(var) => var,
        Err(_) => {
            return Err(gen_error_msg_template("help get"));
        }
    };
    let mut ret = String::new();
    match var {
        Variable::Status => {
            if shared_state::get_maintaining() {
                ret.push_str("Server status is Maintaining");
            } else {
                ret.push_str("Server status is Normal");
            }
        }
        Variable::AutoCleanCycle => ret.push_str(&format!(
            "AutoCleanCycle: {}",
            shared_state::get_auto_clean_duration()
        )),
        Variable::FileSaveDays => ret.push_str(&format!(
            "FileSaveDays: {}",
            shared_state::get_file_save_days()
        )),
    }
    Ok(Some(ret))
}

pub type CommandTransmitData = (String, oneshot::Sender<Option<String>>);

pub async fn cmd_process_loop(
    mut db_conn: DatabaseConnection,
    mut command_rev: mpsc::Receiver<CommandTransmitData>,
    mut shutdown_rev: ShutdownRev,
) -> anyhow::Result<()> {
    tracing::info!("cmd process started");
    let insts = InstManager::new();
    let logic = async {
        while let Some((command, ret)) = command_rev.recv().await {
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
                            Ok(output) => {
                                ret.send(output).unwrap();
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
                                ret.send(Some(format!("{}: {}", command_name, e))).unwrap();
                            }
                        }
                    }
                }
                Err(_e) => {
                    ret.send(Some(format!(
                        "{}{}",
                        command_name.red(),
                        ": Unknown command".red()
                    )))
                    .unwrap();
                }
            };
        }
        anyhow::Ok(())
    };
    let ret = select! {
        ret=logic =>{ret},
        _=shutdown_rev.recv()=>{Ok(())}
    };
    tracing::info!("cmd process loop exited");
    ret
}

pub async fn setup_stdin(
    commend_sdr: mpsc::Sender<CommandTransmitData>,
    mut shutdown_rev: ShutdownRev,
) -> anyhow::Result<()> {
    let mut console_reader = BufReader::new(io::stdin()).lines();
    let logic = async {
        loop {
            print!(">>> ");
            std::io::stdout().flush()?;
            let command;
            command = console_reader.next_line().await.unwrap_or_else(|e| {
                tracing::error!("stdin {}", e);
                None
            });
            let command = match command {
                None => {
                    break;
                }
                Some(d) => d,
            };
            if command.trim().is_empty() {
                continue;
            }
            let (ret_sdr, ret_rev) = oneshot::channel();
            commend_sdr.send((command, ret_sdr)).await?;
            match ret_rev.await {
                Err(e) => {
                    tracing::error!("stdin {}", e);
                }
                Ok(output) => {
                    if let Some(output) = output {
                        println!("{output}");
                    }
                }
            }
        }
        anyhow::Ok(())
    };
    select! {
        _=logic=>{},
        _=shutdown_rev.recv()=>{}
    }
    tracing::info!("stdin cmd source exited");
    Ok(())
}

struct _CmdConnection {
    socket: WS,
}

impl _CmdConnection {
    fn _new(socket: WS) -> Self {
        Self { socket }
    }
}

async fn handle_connection(_socket: TcpStream) -> anyhow::Result<()> {
    Ok(())
}

/// setup network cmd
pub async fn setup_network(
    cmd_port: u16,
    _command_sdr: mpsc::Sender<CommandTransmitData>,
    mut shutdown_rec: ShutdownRev,
) -> anyhow::Result<()> {
    let logic = async {
        let tcplistener = TcpListener::bind(format!("127.0.0.1:{}", cmd_port)).await?;
        loop {
            let connected_socket = match tcplistener.accept().await {
                Err(e) => {
                    tracing::error!("error when accepting socket in cmd {}", e);
                    continue;
                }
                Ok(data) => data,
            };
            if let Err(e) = handle_connection(connected_socket.0).await {
                tracing::error!("error in socket handle:{}", e);
            };
        }
    };
    let ret = select! {
        ret=logic=>{ ret},
        _=shutdown_rec.recv()=>{ Ok(())}
    };
    tracing::info!("network cmd source exited");
    ret
}
