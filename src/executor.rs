use pest::Parser;

use crate::{
    ExecError, Program, TrshError, TrshResult,
    ast::{CmdArg, Command, Conditional, Redirection, SimpleCommand, Token},
    builtins::{Builtin, CmdName},
    prsr::{Rule, TrshPrsr},
};
use std::{
    collections::HashMap,
    fmt::Display,
    io::{self, Stdout, Write},
    os::unix::process::ExitStatusExt,
    path::{Path, PathBuf},
    process::{ExitStatus, Stdio},
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

    // fn exec_pipeline(&mut self, left: Command, right: Command) -> Result<ExitStatus> {
    //     let mut left_proc = match left {
    //         Command::Simple(SimpleCommand {
    //             name,
    //             args,
    //             redirections,
    //         }) => std::process::Command::new(&sc.name)
    //             .args(&sc.args)
    //             .stdout(Stdio::piped())
    //             .spawn()?,
    //         _ => return Err(Error::new(ErrorKind::Other, "nested pipeline unsupported")),
    //     };
    //
    //     let left_stdout = left_proc.stdout.take().expect("missing stdout");
    //
    //     let status = match right {
    //         Command::Simple(sc) => Command::new(&sc.name)
    //             .args(&sc.args)
    //             .stdin(Stdio::from(left_stdout))
    //             .status()?,
    //         _ => return Err(Error::new(ErrorKind::Other, "nested pipeline unsupported")),
    //     };
    //
    //     left_proc.wait()?; // ensure first process completes
    //     Ok(status)
    // }
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
            CmdName::Path(path_buf) => todo!(),
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
    fn exec_builtin(
        &mut self,
        builtin: Builtin,
        args: Vec<CmdArg>,
        stdin: Option<Stdio>,
        stdout: Option<Stdio>,
    ) -> TrshResult<ExitStatus> {
        match builtin {
            Builtin::Colon => todo!(),
            Builtin::Dot => todo!(),
            Builtin::Alias => self.handle_alias(args),
            Builtin::Bg => todo!(),
            Builtin::Break => todo!(),
            Builtin::Cd => self.exec_cd(args),
            Builtin::Command => todo!(),
            Builtin::Continue => todo!(),
            Builtin::Eval => todo!(),
            Builtin::Exec => todo!(),
            Builtin::Exit => std::process::exit(0),
            Builtin::Export => self.handle_export(args),
            Builtin::Fc => todo!(),
            Builtin::Fg => todo!(),
            Builtin::Getopts => todo!(),
            Builtin::Hash => todo!(),
            Builtin::Jobs => todo!(),
            Builtin::Kill => todo!(),
            Builtin::Pwd => {
                println!("{}", self.cwd.display());
                Ok(exit_zero())
            }
            Builtin::Read => todo!(),
            Builtin::Readonly => todo!(),
            Builtin::Return => todo!(),
            Builtin::Set => todo!(),
            Builtin::Shift => todo!(),
            Builtin::Test => self.handle_test(args),
            Builtin::Times => todo!(),
            Builtin::Trap => todo!(),
            Builtin::Type => todo!(),
            Builtin::Ulimit => todo!(),
            Builtin::Umask => todo!(),
            Builtin::Unalias => self.unalias(args),
            Builtin::Unset => self.unset(args),
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
    }
    fn handle_test(&self, args: Vec<CmdArg>) -> TrshResult<ExitStatus> {
        // println!("{args:?}");
        match args.len() {
            1 => todo!("{args:?}"),
            2 => UNARY_TESTS
                .get(args[0].as_str())
                .map(|t| t(args[1].as_str()))
                .map(|tf| {
                    if tf {
                        exit_zero()
                    } else {
                        ExitStatus::from_raw(1)
                    }
                })
                .ok_or(TrshError::gen_exec(
                    "test",
                    &format!("invalid test: {}", args[0]),
                )),
            3 => BINARY_TESTS
                .get(args[1].as_str())
                .map(|bt| bt.compare(args[0].as_str(), args[2].as_str()))
                .unwrap_or(Err(TrshError::gen_exec("test", "invalid cmd"))),
            _ => Err(TrshError::gen_exec(
                "test",
                &format!("can't do anything with {args:?} yet"),
            )),
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

    fn handle_alias(&mut self, args: Vec<CmdArg>) -> TrshResult<ExitStatus> {
        println!("{args:?}");
        if args.is_empty() {
            self.aliases.iter().for_each(|(k, v)| {
                println!("alias {k}=\"{v}\"");
            });
            Ok(ExitStatus::from_raw(0))
        } else {
            for a in args {
                match a {
                    CmdArg::Arg(a) => match self.aliases.get(a.as_str()) {
                        Some(v) => println!("alias {a}=\"{v}\""),
                        None => eprintln!("trsh: alias: {a}: not found"),
                    },
                    CmdArg::Assignment(l, r) => {
                        self.aliases.insert(l, r);
                    }
                    CmdArg::Quoted(_) => todo!(),
                    CmdArg::OpEq => todo!(),
                    CmdArg::OpNeq => todo!(),
                    CmdArg::Variable(_) => todo!(),
                    CmdArg::CommandSub(_) => todo!(),
                }
                // match a {
                //     Token::Word(s) | Token::Quote(s) => match self.aliases.get(s.as_str()) {
                //         Some(a) => println!("alias {s}=\"{a}\""),
                //         None => eprintln!("trsh: alias: {s}: not found"),
                //     },
                //     Token::Eq | Token::Neq => {
                //         // return Err(TrshError::gen_exec("export", "invalid 'equate'"));
                //     }
                //     Token::Assignment(left, right) => {
                //         self.aliases
                //             .insert(left.as_str().to_string(), right.as_str().to_string());
                //     }
                // };
            }
            Ok(exit_zero())
        }
    }

    fn handle_export(&mut self, args: Vec<CmdArg>) -> TrshResult<ExitStatus> {
        if args.is_empty() {
            self.env_vars.iter().for_each(|(k, v)| {
                println!("declare -x {k}=\"{v}\"");
            });
            Ok(exit_zero())
        } else {
            for a in args {
                match a {
                    CmdArg::Arg(a) => {
                        self.env_vars.entry(a).or_default();
                    }
                    CmdArg::Assignment(l, r) => {
                        self.env_vars.insert(l, r);
                    }
                    CmdArg::Quoted(_) => todo!(),
                    CmdArg::OpEq => todo!(),
                    CmdArg::OpNeq => todo!(),
                    CmdArg::Variable(_) => todo!(),
                    CmdArg::CommandSub(_) => todo!(),
                }
                // match a {
                //     Token::Quote(_) => (),
                //     Token::Word(s) => {
                //         if !self.env_vars.contains_key(s.as_str()) {
                //             self.env_vars.insert(s.to_string(), "".to_string());
                //         }
                //     }
                //     e @ (Token::Eq | Token::Neq) => {
                //         eprintln!("trsh: export: {e}: not a valid identifier")
                //     }
                //     Token::Assignment(left, right) => {
                //         self.env_vars.insert(left.to_string(), right.to_string());
                //     }
                // };
            }
            Ok(exit_zero())
        }
    }
    fn unalias(&mut self, args: Vec<CmdArg>) -> TrshResult<ExitStatus> {
        if args.is_empty() {
            println!("nalias: usage: unalias [-a] name [name ...]");
            Ok(exit_zero())
        } else {
            for a in args {
                match self.aliases.remove(a.as_str()) {
                    Some(_) => (),
                    None => return Err(TrshError::gen_exec("unalias", &format!("{a}: not found"))),
                }
            }
            Ok(exit_zero())
        }
        // if let Some(a) = args {
        //     match self.aliases.remove(&a) {
        //         Some(_) => (),
        //         None => eprintln!("trsh: unalias: {a}: not found"),
        //     }
        // }
    }

    fn unset(&mut self, args: Vec<CmdArg>) -> TrshResult<ExitStatus> {
        if !args.is_empty() {
            for a in args {
                self.env_vars.remove(a.as_str());
            }
        }
        Ok(exit_zero())
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

fn exit_zero() -> ExitStatus {
    ExitStatus::from_raw(0)
}

fn exit_num(i: i32) -> ExitStatus {
    ExitStatus::from_raw(i)
}

static BINARY_TESTS: phf::Map<&'static str, BinaryTest> = phf::phf_map! {
    "=" => BinaryTest::Eq,
    "!=" => BinaryTest::Neq,
    "-eq" => BinaryTest::Eq,
    "-ne" => BinaryTest::Neq,
    "-gt" => BinaryTest::Gt,
    "-lt" => BinaryTest::Lt,
    "-ge" => BinaryTest::GtEq,
    "-le" => BinaryTest::LtEq,
};

enum BinaryTest {
    Eq,
    Neq,
    Gt,
    GtEq,
    Lt,
    LtEq,
}

impl BinaryTest {
    pub fn compare(&self, left: &str, right: &str) -> TrshResult<ExitStatus> {
        let left_num = match left.parse::<i64>() {
            Ok(i) => i,
            Err(_) => return Ok(exit_num(2)),
        };
        let right_num = match right.parse::<i64>() {
            Ok(i) => i,
            Err(_) => return Ok(exit_num(2)),
        };
        if match self {
            BinaryTest::Eq => left_num == right_num,
            BinaryTest::Neq => left_num != right_num,
            BinaryTest::Gt => left_num > right_num,
            BinaryTest::GtEq => left_num >= right_num,
            BinaryTest::Lt => left_num < right_num,
            BinaryTest::LtEq => left_num <= right_num,
        } {
            Ok(exit_zero())
        } else {
            Ok(exit_num(1))
        }
    }
}

static UNARY_TESTS: phf::Map<&'static str, fn(&str) -> bool> = phf::phf_map! {
    "-f" => |path| Path::new(path).is_file(),
    "-d" => |path| Path::new(path).is_dir(),
    "-e" => |path| Path::new(path).exists(),
    "-n" => |s| !s.is_empty(),
    "-z" => |s| s.is_empty(),
};
