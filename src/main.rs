mod ast;
mod builtins;
mod executor;
mod prsr;
use std::path::Path;

use ast::Command;
use executor::Executor;
use pest::{Parser, iterators::Pair};
use prsr::{Rule, TrshPrsr};
use rustyline::{
    Config, Editor,
    history::{DefaultHistory, FileHistory, History},
};
type ParsedIterResult<'a> =
    std::result::Result<pest::iterators::Pairs<'a, prsr::Rule>, pest::error::Error<prsr::Rule>>;

type TrshResult<C> = Result<C, TrshError>;
type ParsedIter<'a> = pest::iterators::Pairs<'a, prsr::Rule>;
type ParsedPair<'a> = Pair<'a, Rule>;
fn main() {
    let config = Config::builder().auto_add_history(true).build();
    let mut history = DefaultHistory::new();
    let h = Path::new("hist");
    history.load(h).unwrap();
    let mut rl: Editor<(), FileHistory> = Editor::with_history(config, history).unwrap();
    let mut executor = Executor::new();
    loop {
        let prompt = format!("[trsh: {}]$ ", executor);
        match rl.readline(&prompt) {
            Ok(readline) => {
                TrshPrsr::parse(Rule::program, &readline)
                    // .inspect(|e| println!("{:?}", e))
                    .map_err(|e| TrshError::Pest(Box::new(e)))
                    .and_then(|mut r| Program::new(r.next().unwrap()))
                    .and_then(|prog| executor.exec(prog.0))
                    .map(|_| {})
                    .map_err(|e| eprintln!("{e:?}"))
                    .ok();
            }
            Err(readerr) => panic!("{readerr}"),
        }
    }
}
fn root_ast(rules: ParsedIterResult<'_>) {
    match rules {
        Ok(parsed_iter) => {
            for p in parsed_iter {
                println!("{p:?}");
            }
        }
        Err(_) => todo!(),
    }
}

#[derive(Debug)]
struct Program(pub Command);
impl Program {
    pub fn new(rule: ParsedPair<'_>) -> TrshResult<Self> {
        let mut rules = rule.into_inner();
        Ok(Self(Command::new(rules.next().unwrap())?))
    }
}

#[derive(Debug)]
enum TrshError {
    Ast(AstError),
    Exec(ExecError),
    Pest(Box<pest::error::Error<prsr::Rule>>),
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
}
