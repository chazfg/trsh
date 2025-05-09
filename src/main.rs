mod ast;
use clap::Parser as ClapParser;
mod builtins;
mod executor;
mod prsr;
use ast::Command;
use colored::Colorize;
use executor::Executor;
use pest::{Parser, iterators::Pair};
use prsr::{Rule, TrshPrsr};
use rustyline::{
    Config, Editor,
    history::{DefaultHistory, FileHistory, History},
};
use std::{
    collections::HashMap,
    fmt::Display,
    path::{Path, PathBuf},
};
type ParsedIterResult<'a> =
    std::result::Result<pest::iterators::Pairs<'a, prsr::Rule>, pest::error::Error<prsr::Rule>>;

type TrshResult<C> = Result<C, TrshError>;
type ParsedIter<'a> = pest::iterators::Pairs<'a, prsr::Rule>;
type ParsedPair<'a> = Pair<'a, Rule>;

fn main() {
    match CliTrshArgs::parse() {
        CliTrshArgs {
            script_file: None,
            cmd: None,
        } => repl(),
        CliTrshArgs {
            script_file: Some(script_file),
            cmd: None,
        } => {
            let s = std::fs::read_to_string(script_file).expect("failed to read file");
            run_once(&s);
        }
        CliTrshArgs {
            script_file: None,
            cmd: Some(cmd),
        } => run_once(&cmd),
        _ => panic!("got weird input"),
    }
}

fn repl() {
    let config = Config::builder().auto_add_history(true).build();
    let mut history = DefaultHistory::new();
    let h = Path::new("hist");
    history.load(h).unwrap();
    let mut rl: Editor<(), FileHistory> = Editor::with_history(config, history).unwrap();
    let mut executor = Executor::new();
    executor.load_trshrc();
    loop {
        let prompt = format!("{}{}{} ", "[trsh: ".cyan(), executor, "]$".cyan());
        match rl.readline(&prompt) {
            Ok(readline) => {
                TrshPrsr::parse(Rule::program, &readline)
                    // .inspect(|e| println!("{:?}", e))
                    .map_err(|e| TrshError::Pest(Box::new(e)))
                    .and_then(|mut r| {
                        Program::new(r.next().unwrap(), executor.env(), &mut Some(&mut rl))
                    })
                    .and_then(|prog| executor.exec(prog.0, None, None))
                    .map(|_| {})
                    .map_err(|e| eprintln!("trsh: full bubble {e:?}"))
                    .ok();
            }
            Err(readerr) => panic!("{readerr}"),
        }
    }
}

fn run_once(s: &str) {
    let mut executor = Executor::new();
    TrshPrsr::parse(Rule::program, s)
        // .inspect(|e| println!("{:?}", e))
        .map_err(|e| TrshError::Pest(Box::new(e)))
        .and_then(|mut r| Program::new(r.next().unwrap(), executor.env(), &mut None))
        .and_then(|prog| executor.exec(prog.0, None, None))
        .map(|_| {})
        .map_err(|e| eprintln!("{e:?}"))
        .ok();
}

#[derive(Debug)]
struct Program(pub Command);
impl Program {
    pub fn new(
        rule: ParsedPair<'_>,
        env: (&HashMap<String, String>, &HashMap<String, String>),
        rl: &mut Option<&mut Editor<(), FileHistory>>,
    ) -> TrshResult<Self> {
        let mut rules = rule.into_inner();
        Ok(Self(Command::new(rules.next().unwrap(), env, rl)?))
    }
}

#[derive(Debug)]
enum TrshError {
    Ast(AstError),
    Exec(ExecError),
    Pest(Box<pest::error::Error<prsr::Rule>>),
}

impl TrshError {
    pub fn gen_exec(name: &str, expl: &str) -> Self {
        Self::Exec(ExecError::new(name, expl))
    }
}

impl From<std::io::Error> for TrshError {
    fn from(value: std::io::Error) -> Self {
        Self::Exec(ExecError::IO(Box::new(value)))
    }
}

impl From<ExecError> for TrshError {
    fn from(value: ExecError) -> Self {
        Self::Exec(value)
    }
}
impl From<AstError> for TrshError {
    fn from(value: AstError) -> Self {
        Self::Ast(value)
    }
}

#[derive(Debug)]
enum AstError {
    IncompleteConditional,
}
#[derive(Debug)]
enum ExecError {
    Failed,
    UnknownCmd,
    General(Box<Expl>),
    IO(Box<std::io::Error>),
}

impl ExecError {
    pub fn new(name: &str, expl: &str) -> Self {
        Self::General(Box::new(Expl {
            name: name.to_owned(),
            expl: expl.to_owned(),
        }))
    }
}

#[derive(Debug)]
struct Expl {
    name: String,
    expl: String,
}

impl Display for Expl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { name, expl } = self;
        write!(f, "{name}: {expl}")
    }
}

#[derive(clap::Parser, Debug)]
struct CliTrshArgs {
    #[arg(conflicts_with("cmd"))]
    script_file: Option<PathBuf>,
    #[arg(short = 'c', conflicts_with("script_file"))]
    cmd: Option<String>,
}
