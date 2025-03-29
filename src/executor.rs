use pest::Parser;

use crate::{
    Program, TrshError, TrshResult,
    ast::{Command, Redirection, SimpleCommand, ValidArg},
    builtins::{Builtin, CmdName},
    prsr::{Rule, TrshPrsr},
};
use std::{
    collections::HashMap,
    fmt::Display,
    io::{self, Stdout, Write},
    path::{Path, PathBuf},
    process::Stdio,
};

pub struct Executor {
    env_vars: HashMap<String, String>, // Shell's environment variables
    cwd: PathBuf,                      // Current working directory (internal state)
    home_dir: PathBuf,
    last_status: i32,                   // Last command's exit code ($?)
    aliases: HashMap<String, String>,   // Alias table
    functions: HashMap<String, String>, // Optional later
    std_out: Stdout,
}
impl Display for Executor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.home_dir == self.cwd {
            write!(f, "~")
        } else {
            match self.cwd.file_name() {
                Some(cwd) => write!(f, "{}", cwd.display()),
                None => write!(f, "{}", self.cwd.display()),
            }
        }
    }
}

impl Executor {
    pub fn env(&self) -> (&HashMap<String, String>, &HashMap<String, String>) {
        (&self.aliases, &self.functions)
    }
    pub fn new() -> Self {
        let cwd = std::env::current_dir().unwrap();
        let home_dir = dirs::home_dir().unwrap();
        let std_out = io::stdout();
        let env_vars = std::env::vars().collect();
        Self {
            env_vars,
            cwd,
            home_dir,
            last_status: 0,
            aliases: HashMap::new(),
            functions: HashMap::new(),
            std_out,
        }
    }
    pub fn load_trshrc(&mut self) {
        let possible_trsh = self.home_dir.join(".trshrc");
        if possible_trsh.exists() {
            match std::fs::read_to_string(possible_trsh) {
                Ok(trshrc) => {
                    // won't work with declaring functions, fine for now though
                    for line in trshrc.lines() {
                        TrshPrsr::parse(Rule::program, line)
                            // .inspect(|e| println!("{:?}", e))
                            .map_err(|e| TrshError::Pest(Box::new(e)))
                            .and_then(|mut r| {
                                Program::new(r.next().unwrap(), self.env(), &mut None)
                            })
                            .and_then(|prog| self.exec(prog.0))
                            .map(|_| {})
                            .map_err(|e| eprintln!("{e:?}"))
                            .ok();
                    }
                }
                Err(e) => eprintln!("trsh: error parsing .trshrc: {e}"),
            }
        }
    }
    pub fn exec(&mut self, cmd: Command) -> TrshResult<()> {
        match cmd {
            crate::ast::Command::Simple(simple_command) => self.exec_simple(simple_command),
            crate::ast::Command::Conditional(conditional) => todo!(),
            crate::ast::Command::Sequence(commands) => {
                commands.into_iter().try_for_each(|c| self.exec(c))
            }
        }
    }

    fn exec_simple(&mut self, simple_command: SimpleCommand) -> TrshResult<()> {
        let SimpleCommand {
            name,
            args,
            redirections,
        } = simple_command;
        match name {
            CmdName::Builtin(builtin) => self.exec_builtin(builtin, args),
            CmdName::Unknown(unknown_cmd) => self.exec_unknown(unknown_cmd, args, redirections),
            CmdName::Path(path_buf) => todo!(),
            CmdName::Alias(a) => TrshPrsr::parse(Rule::program, &a)
                .map_err(|e| TrshError::Pest(Box::new(e)))
                .and_then(|mut r| Program::new(r.next().unwrap(), self.env(), &mut None))
                .and_then(|prog| self.exec(prog.0)),
            CmdName::Function(_) => todo!(),
        }
    }
    fn exec_builtin(&mut self, builtin: Builtin, args: Vec<ValidArg>) -> TrshResult<()> {
        match builtin {
            Builtin::Colon => todo!(),
            Builtin::Dot => todo!(),
            Builtin::Alias => self.handle_alias(args)?,
            Builtin::Bg => todo!(),
            Builtin::Break => todo!(),
            Builtin::Cd => self.exec_cd(args),
            Builtin::Command => todo!(),
            Builtin::Continue => todo!(),
            Builtin::Eval => todo!(),
            Builtin::Exec => todo!(),
            Builtin::Exit => std::process::exit(0),
            Builtin::Export => self.handle_export(args)?,
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
            Builtin::Unalias => self.unalias(args)?,
            Builtin::Unset => self.unset(args)?,
            Builtin::Wait => todo!(),
            Builtin::Bind => todo!(),
            Builtin::Builtin => todo!(),
            Builtin::Caller => todo!(),
            Builtin::Declare => todo!(),
            Builtin::Echo => todo!(),
            Builtin::Enable => todo!(),
            Builtin::Help => todo!(),
            Builtin::Let => todo!(),
            Builtin::Local => todo!(),
            Builtin::Logout => todo!(),
            Builtin::Mapfile => todo!(),
            Builtin::Printf => todo!(),
            Builtin::Readarray => todo!(),
            Builtin::Source => todo!(),
            Builtin::Shopt => todo!(),
        }
        Ok(())
    }
    fn exec_unknown(
        &self,
        unknown: String,
        args: Vec<ValidArg>,
        redirs: Vec<Redirection>,
    ) -> TrshResult<()> {
        match self.lookup_command(&unknown) {
            Some(p) => {
                let mut process = std::process::Command::new(p);
                process.args(args);
                process.current_dir(&self.cwd);
                for d in redirs {
                    match d {
                        Redirection::AppendRight(s) => {
                            let f = std::fs::OpenOptions::new()
                                .create(true)
                                .append(true)
                                .open(s)?;
                            process.stdout(Stdio::from(f));
                        }
                        Redirection::Input(s) => {
                            let file = std::fs::File::open(s)?;
                            process.stdin(Stdio::from(file));
                        }
                        Redirection::TruncRight(s) => {
                            let file = std::fs::File::create(s)?;
                            process.stdout(Stdio::from(file));
                        }
                        Redirection::HereDoc(s) => {
                            let mut child_proc = process.stdin(Stdio::piped()).spawn()?;
                            if let Some(mut stdin) = child_proc.stdin.take() {
                                stdin.write_all(s.as_bytes())?;
                            }
                            child_proc.wait()?;
                        }
                    }
                }

                match process.status() {
                    Ok(_) => (),
                    Err(e) => eprintln!("trsh: {}: exec error", e),
                }
            }
            None => eprintln!("trsh: command not found: {unknown}"),
        }
        Ok(())
    }
    fn exec_cd(&mut self, args: Vec<ValidArg>) {
        if args.is_empty() {
            self.cwd = self.home_dir.clone();
        } else if args.len() == 1 {
            let test_new = self
                .cwd
                .join::<&std::ffi::OsStr>(args[0].as_ref())
                .canonicalize();
            match test_new {
                Ok(o) => {
                    self.cwd = o;
                }
                Err(e) => eprintln!("trsh: cd: {e}"),
            }
        } else {
            eprintln!("trsh: cd: too many arguments");
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

    fn handle_alias(&mut self, args: Vec<ValidArg>) -> TrshResult<()> {
        if args.is_empty() {
            self.aliases.iter().for_each(|(k, v)| {
                println!("alias {k}=\"{v}\"");
            });
        } else if args.len() == 1 {
            match &args[0] {
                // ValidArg::Redirection(s) => eprintln!("trsh: alias: invalid {s}"),
                ValidArg::Filename(s) | ValidArg::Word(s) | ValidArg::Quote(s) => {
                    eprintln!("trsh: alias: invalid {s}")
                }
                ValidArg::Assignment(s) => {
                    let (lhs, rhs) = split_assignment(s);
                    self.aliases.insert(lhs.to_string(), rhs.to_string());
                }
            }
        } else {
            eprintln!("trsh: alias: too many arguments");
        }
        Ok(())
    }

    fn handle_export(&mut self, args: Vec<ValidArg>) -> TrshResult<()> {
        if args.is_empty() {
            self.env_vars.iter().for_each(|(k, v)| {
                println!("declare -x {k}=\"{v}\"");
            });
        } else if args.len() == 1 {
            match &args[0] {
                // ValidArg::Redirection(s) => eprintln!("trsh: alias: invalid {s}"),
                ValidArg::Filename(s) | ValidArg::Word(s) | ValidArg::Quote(s) => {
                    eprintln!("trsh: export: invalid {s}")
                }
                ValidArg::Assignment(s) => {
                    let (lhs, rhs) = split_assignment(s);
                    self.env_vars.insert(lhs.to_string(), rhs.to_string());
                }
            }
        } else {
            eprintln!("trsh: export: too many arguments");
        }
        Ok(())
    }
    fn unalias(&mut self, args: Vec<ValidArg>) -> TrshResult<()> {
        if args.is_empty() {
            println!("nalias: usage: unalias [-a] name [name ...]");
        } else {
            for a in args {
                match self.aliases.remove(a.as_str()) {
                    Some(_) => (),
                    None => eprintln!("trsh: unalias: {a}: not found"),
                }
            }
        }
        // if let Some(a) = args {
        //     match self.aliases.remove(&a) {
        //         Some(_) => (),
        //         None => eprintln!("trsh: unalias: {a}: not found"),
        //     }
        // }
        Ok(())
    }

    fn unset(&mut self, args: Vec<ValidArg>) -> TrshResult<()> {
        if !args.is_empty() {
            for a in args {
                self.env_vars.remove(a.as_str());
            }
        }
        Ok(())
    }
}

fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|meta| meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

fn split_assignment(s: &str) -> (&str, &str) {
    let (lhs, rhs) = match s.split_once("=") {
        Some((l, r)) => (l, r),
        None => panic!("{s}"),
    };
    (lhs.trim(), rhs.trim().trim_matches('"'))
}
