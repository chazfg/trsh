use pest::Parser;
use utils::{exit_zero, is_executable};
mod builtins;
mod utils;

use crate::{
    ExecError, Program, TrshError, TrshResult,
    ast::{CmdArg, Command, Conditional, Redirection, SimpleCommand, WhileLoop},
    builtins::CmdName,
    prsr::{Rule, TrshPrsr},
};
use std::{
    collections::HashMap,
    fmt::Display,
    io::{self, Stdout, Write},
    path::PathBuf,
    process::{ExitStatus, Stdio},
};

pub struct Executor {
    env_vars: HashMap<String, String>,
    vars: HashMap<String, String>,
    cwd: PathBuf,
    home_dir: PathBuf,
    last_status: i32,
    aliases: HashMap<String, String>,
    functions: HashMap<String, String>,
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
        let vars = HashMap::new();
        Self {
            env_vars,
            vars,
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
                            .and_then(|prog| self.exec(prog.0, None, None))
                            .map(|_| {})
                            .map_err(|e| eprintln!("{e:?}"))
                            .ok();
                    }
                }
                Err(e) => eprintln!("trsh: error parsing .trshrc: {e}"),
            }
        }
    }

    pub fn exec(
        &mut self,
        cmd: Command,
        stdin: Option<Stdio>,
        stdout: Option<Stdio>,
    ) -> TrshResult<ExitStatus> {
        match cmd {
            Command::Simple(simple_command) => self.exec_simple(simple_command, stdin, stdout),
            Command::Conditional(conditional) => self.exec_conditional(conditional, stdin, stdout),
            Command::Sequence(commands) => {
                let r = commands
                    .into_iter()
                    .try_for_each(|c| self.exec(c, None, None).map(|_| ()));
                r.map(|_| exit_zero())
            }
            Command::Pipeline(left, right) => {
                let (in_pipe, out_pipe) = os_pipe::pipe().unwrap();
                self.exec(*left, None, Some(out_pipe.into()))?;
                self.exec(*right, Some(in_pipe.into()), None)
                // Ok(())
            }
            Command::And(left, right) => {
                let left_status = self.exec(*left, None, None)?;
                if left_status.success() {
                    self.exec(*right, None, None)
                } else {
                    Ok(left_status)
                }
            }
            Command::Or(left, right) => {
                let left_status = self.exec(*left, None, None)?;
                if !left_status.success() {
                    self.exec(*right, None, None)
                } else {
                    Ok(left_status)
                }
            }
            Command::WhileLoop(WhileLoop { condition, body }) => {
                while self
                    .exec(*condition.clone(), None, None)
                    .is_ok_and(|tf| tf.success())
                {
                    self.exec(*body.clone(), None, None)?;
                }
                Ok(exit_zero())
                // let r = self.exec(*condition, None, None);
                // println!("{r:?}");
            }
        }
    }
    fn exec_conditional(
        &mut self,
        cond: Conditional,
        stdin: Option<Stdio>,
        stdout: Option<Stdio>,
    ) -> TrshResult<ExitStatus> {
        let Conditional {
            condition,
            then_branch,
            else_branch,
        } = cond;
        let status = self.exec(*condition, None, None)?;
        if status.success() {
            self.exec(*then_branch, None, None)
        } else if let Some(eb) = else_branch {
            self.exec(*eb, None, None)
        } else {
            Ok(status)
        }
        // todo!()
    }

    fn exec_simple(
        &mut self,
        simple_command: SimpleCommand,
        stdin: Option<Stdio>,
        stdout: Option<Stdio>,
    ) -> TrshResult<ExitStatus> {
        let SimpleCommand {
            name,
            args,
            redirections,
        } = simple_command;
        match name {
            CmdName::Builtin(builtin) => self.exec_builtin(builtin, args, stdin, stdout),
            CmdName::Unknown(unknown_cmd) => {
                self.exec_unknown(unknown_cmd, args, redirections, stdin, stdout)
            }
            CmdName::Path(_path_buf) => todo!(),
            CmdName::Alias(a) => {
                println!("{a}");
                TrshPrsr::parse(Rule::program, &a)
                    .map_err(|e| TrshError::Pest(Box::new(e)))
                    .and_then(|mut r| Program::new(r.next().unwrap(), self.env(), &mut None))
                    .and_then(|prog| self.exec(prog.0, stdin, stdout))
            }
            CmdName::Function(_) => todo!(),
        }
    }
    fn exec_unknown(
        &self,
        unknown: String,
        args: Vec<CmdArg>,
        redirs: Vec<Redirection>,
        stdin: Option<Stdio>,
        stdout: Option<Stdio>,
    ) -> TrshResult<ExitStatus> {
        match self.lookup_command(&unknown) {
            Some(p) => {
                let mut process = std::process::Command::new(p);
                process.args(args.iter().map(CmdArg::as_os_string));
                process.current_dir(&self.cwd);
                for d in redirs {
                    match d {
                        Redirection::AppendRight(s) => {
                            let f = std::fs::OpenOptions::new()
                                .create(true)
                                .append(true)
                                .open(s)?;
                            if stdout.is_none() {
                                process.stdout(Stdio::from(f));
                            }
                        }
                        Redirection::Input(s) => {
                            let file = std::fs::File::open(s)?;
                            if stdin.is_none() {
                                process.stdin(Stdio::from(file));
                            }
                        }
                        Redirection::TruncRight(s) => {
                            let file = std::fs::File::create(s)?;
                            if stdout.is_none() {
                                process.stdout(Stdio::from(file));
                            }
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
                if let Some(s_in) = stdin {
                    process.stdin(s_in);
                }
                if let Some(s_out) = stdout {
                    process.stdout(s_out);
                } else {
                    process.stdout(Stdio::inherit());
                }
                Ok(process.status()?)
                // match process.status() {
                //     Ok(_) => (),
                //     Err(e) => eprintln!("trsh: {}: exec error", e),
                // }
            }
            None => Err(TrshError::Exec(ExecError::UnknownCmd)),
        }
        //Ok(())
    }
    fn exec_cd(&mut self, args: Vec<CmdArg>) -> TrshResult<ExitStatus> {
        if args.is_empty() {
            self.cwd = self.home_dir.clone();
            Ok(exit_zero())
        } else if args.len() == 1 {
            let test_new = self
                .cwd
                .join::<&std::ffi::OsStr>(&args[0].as_os_string())
                .canonicalize();
            match test_new {
                Ok(o) => {
                    self.cwd = o;
                    Ok(exit_zero())
                }
                Err(e) => Err(TrshError::gen_exec("cd", &format!("{e}"))),
            }
        } else {
            Err(TrshError::gen_exec("cd", "too many arguments"))
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
