use std::{collections::HashMap, fmt::Display, path::PathBuf};

use crate::{
    AstError, ParsedPair, TrshResult,
    builtins::{Builtin, CmdName},
    prsr::Rule,
};

#[derive(Debug)]
pub struct SimpleCommand {
    pub name: CmdName,
    pub args: Vec<ValidArg>,
}

#[derive(Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum ValidArg {
    Word(String),
    Quote(String),
    Assignment(String),
}

impl Display for ValidArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidArg::Word(s) | ValidArg::Quote(s) | ValidArg::Assignment(s) => write!(f, "{s}"),
        }
    }
}

impl ValidArg {
    pub fn new(a: ParsedPair) -> Self {
        match a.as_rule() {
            Rule::WORD => Self::Word(a.as_str().to_string()),
            Rule::QUOTE => Self::Quote(a.as_str().to_string()),
            Rule::ASSIGNMENT => Self::Assignment(a.as_str().to_string()),
            Rule::arg => Self::new(a.into_inner().next().unwrap()),
            r => panic!("{r:?}"),
        }
    }
    pub fn as_str(&self) -> &str {
        match self {
            ValidArg::Word(s) => s,
            ValidArg::Quote(s) => s,
            ValidArg::Assignment(s) => s,
        }
    }
}

impl AsRef<String> for ValidArg {
    fn as_ref(&self) -> &String {
        match self {
            ValidArg::Word(s) => s,
            ValidArg::Quote(s) => s,
            ValidArg::Assignment(s) => s,
        }
    }
}

impl std::borrow::Borrow<str> for ValidArg {
    fn borrow(&self) -> &str {
        match self {
            ValidArg::Word(s) => s,
            ValidArg::Quote(s) => s,
            ValidArg::Assignment(s) => s,
        }
    }
}

impl AsRef<std::ffi::OsStr> for ValidArg {
    fn as_ref(&self) -> &std::ffi::OsStr {
        match self {
            ValidArg::Word(s) => std::ffi::OsStr::new(s),
            ValidArg::Quote(s) => std::ffi::OsStr::new(s),
            ValidArg::Assignment(s) => std::ffi::OsStr::new(s),
        }
    }
}

impl SimpleCommand {
    pub fn new(
        rule: ParsedPair<'_>,
        env: (&HashMap<String, String>, &HashMap<String, String>),
    ) -> TrshResult<Self> {
        let mut parts = rule.into_inner();

        let parts_cmd = parts.next().unwrap();
        let parts_name = parts_cmd.as_str();
        let parts_rule = parts_cmd.as_rule();

        let name = if parts_name.contains("/") {
            CmdName::Path(PathBuf::from(parts_name))
        } else if let Some(cmd) = env.0.get(parts_name) {
            CmdName::Alias(cmd.clone())
        } else if let Some(func) = env.1.get(parts_name) {
            CmdName::Function(func.clone())
        } else {
            match parts_rule {
                Rule::command_name => CmdName::Unknown(parts_name.to_string()),
                Rule::builtin_command => CmdName::Builtin(Builtin::new(
                    parts_cmd.into_inner().next().unwrap().as_rule(),
                )),
                r => panic!("got {r:?} for a cmd name"),
            }
        };

        // let rule = name_rule.as_rule();
        let args = parts.map(|p| ValidArg::new(p)).collect();
        // let args = match parts.next().map(|s| s.as_str().trim()) {
        //     Some(s) => {
        //         if s.is_empty() {
        //             None
        //         } else {
        //             Some(s.to_string())
        //         }
        //     }
        //     None => None,
        // };
        Ok(Self { name, args })
    }
}

#[derive(Debug)]
pub struct Conditional {
    condition: Box<Command>,
    then_branch: Box<Command>,
    else_branch: Box<Command>,
}

impl Conditional {
    fn new(
        rule: ParsedPair<'_>,

        env: (&HashMap<String, String>, &HashMap<String, String>),
    ) -> TrshResult<Self> {
        let mut parts = rule.into_inner();
        let condition = Box::new(Command::new(
            parts.next().ok_or(AstError::IncompleteConditional)?,
            env,
        )?);
        let then_branch = Box::new(Command::new(
            parts.next().ok_or(AstError::IncompleteConditional)?,
            env,
        )?);
        let else_branch = Box::new(Command::new(
            parts.next().ok_or(AstError::IncompleteConditional)?,
            env,
        )?);
        Ok(Self {
            condition,
            then_branch,
            else_branch,
        })
    }
}

#[derive(Debug)]
pub enum Command {
    Simple(SimpleCommand),
    Conditional(Conditional),
    Sequence(Vec<Self>),
}
impl Command {
    pub fn new(
        rule: ParsedPair<'_>,
        env: (&HashMap<String, String>, &HashMap<String, String>),
    ) -> TrshResult<Self> {
        Ok(match rule.as_rule() {
            Rule::program => todo!(),
            Rule::command_list => {
                let mut v = Vec::new();
                for r in rule.into_inner() {
                    v.push(Self::new(r, env)?);
                }
                Self::Sequence(v)
            }
            Rule::if_clause => Self::Conditional(Conditional::new(rule, env)?),
            Rule::simple_command => Self::Simple(SimpleCommand::new(rule, env)?),
            Rule::WHITESPACE => todo!(),
            Rule::NEWLINE => todo!(),
            Rule::command => todo!(),
            Rule::command_name => todo!(),
            l => todo!("{:?}", l),
        })
    }
}
