use crate::{
    TrshResult,
    ast::{Command, SimpleCommand},
    builtins::Builtin,
};
use std::{
    collections::HashMap,
    fmt::Display,
    io::{self, Stdout, Write},
    path::{Path, PathBuf},
};

pub struct Executor {
    env_vars: HashMap<String, String>, // Shell's environment variables
    cwd: PathBuf,                      // Current working directory (internal state)
    home_dir: PathBuf,
    last_status: i32,                    // Last command's exit code ($?)
    aliases: HashMap<String, String>,    // Alias table
    functions: HashMap<String, Command>, // Optional later
    std_out: Stdout,
}
impl Display for Executor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.home_dir == self.cwd {
            write!(f, "~")
        } else {
            write!(f, "{}", self.cwd.file_name().unwrap().display())
        }
    }
}

impl Executor {
    pub fn new() -> Self {
        let cwd = std::env::current_dir().unwrap();
        let home_dir = dirs::home_dir().unwrap();
        let std_out = io::stdout();
        Self {
            env_vars: HashMap::new(),
            cwd,
            home_dir,
            last_status: 0,
            aliases: HashMap::new(),
            functions: HashMap::new(),
            std_out,
        }
    }
    pub fn exec(&mut self, cmd: Command) -> TrshResult<()> {
        // println!("{:?}", cmd);
        match cmd {
            crate::ast::Command::Simple(simple_command) => self.exec_simple(simple_command),
            crate::ast::Command::Conditional(conditional) => todo!(),
            crate::ast::Command::Sequence(commands) => {
                commands.into_iter().try_for_each(|c| self.exec(c))
            }
        }
    }

    fn exec_simple(&mut self, simple_command: SimpleCommand) -> TrshResult<()> {
        let SimpleCommand { name, args } = simple_command;
        match name {
            crate::builtins::CmdName::Builtin(builtin) => self.exec_builtin(builtin, args),
            crate::builtins::CmdName::Unknown(unknown_cmd) => self.exec_unknown(unknown_cmd, args),
        }
    }
    fn exec_builtin(&mut self, builtin: Builtin, args: Vec<String>) -> TrshResult<()> {
        match builtin {
            Builtin::Colon => todo!(),
            Builtin::Dot => todo!(),
            Builtin::Alias => todo!(),
            Builtin::Bg => todo!(),
            Builtin::Break => todo!(),
            Builtin::Cd => self.exec_cd(args),
            Builtin::Command => todo!(),
            Builtin::Continue => todo!(),
            Builtin::Eval => todo!(),
            Builtin::Exec => todo!(),
            Builtin::Exit => std::process::exit(0),
            Builtin::Export => todo!(),
            Builtin::Fc => todo!(),
            Builtin::Fg => todo!(),
            Builtin::Getopts => todo!(),
            Builtin::Hash => todo!(),
            Builtin::Jobs => todo!(),
            Builtin::Kill => todo!(),
            Builtin::Pwd => println!("{}", self.cwd.display()),
            Builtin::Read => todo!(),
            Builtin::Readonly => todo!(),
            Builtin::Return => todo!(),
            Builtin::Set => todo!(),
            Builtin::Shift => todo!(),
            Builtin::Test => todo!(),
            Builtin::Times => todo!(),
            Builtin::Trap => todo!(),
            Builtin::Type => todo!(),
            Builtin::Ulimit => todo!(),
            Builtin::Umask => todo!(),
            Builtin::Unalias => todo!(),
            Builtin::Unset => todo!(),
            Builtin::Wait => todo!(),
        }
        Ok(())
    }
    fn exec_unknown(&self, unknown: String, args: Vec<String>) -> TrshResult<()> {
        match self.lookup_command(&unknown) {
            Some(p) => {
                let out = std::process::Command::new(p)
                    .args(args)
                    .current_dir(&self.cwd)
                    .status();
            }
            None => eprintln!("trsh: command not found: {unknown}"),
        }
        Ok(())
    }
    fn exec_cd(&mut self, args: Vec<String>) {
        match args.len() {
            0 => {
                self.cwd = self.home_dir.clone();
            }
            1 => todo!(),
            _ => eprintln!("too many arguments"),
        }
    }

    fn lookup_command(&self, cmd_name: &str) -> Option<PathBuf> {
        if cmd_name.contains('/') {
            let path = PathBuf::from(cmd_name);
            if path.is_file() && is_executable(&path) {
                return Some(path);
            } else {
                return None;
            }
        }

        let paths = self
            .env_vars
            .get("PATH")
            .cloned()
            .unwrap_or("/usr/bin:/bin".to_string());
        for dir in paths.split(':') {
            let candidate = PathBuf::from(dir).join(cmd_name);
            if candidate.is_file() && is_executable(&candidate) {
                return Some(candidate);
            }
        }

        None
    }
}

fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|meta| meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}
