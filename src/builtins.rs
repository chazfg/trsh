use std::collections::HashMap;
use std::path::PathBuf;

use crate::ParsedPair;
use crate::ast::Command;
use crate::prsr::Rule;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Builtin {
    Colon, // :
    Dot,   // .
    Alias,
    Bind,
    Builtin,
    Caller,
    Command,
    Declare,
    Echo,
    Enable,
    Help,
    Let,
    Local,
    Logout,
    Mapfile,
    Printf,
    Readarray,
    Source,
    Shopt,
    Bg,
    Break,
    Cd,
    Continue,
    Eval,
    Exec,
    Exit,
    Export,
    Fc,
    Fg,
    Getopts,
    Hash,
    Jobs,
    Kill,
    Pwd,
    Read,
    Readonly,
    Return,
    Set,
    Shift,
    Test,
    Times,
    Trap,
    Type,
    Ulimit,
    Umask,
    Unalias,
    Unset,
    Wait,
}

impl Builtin {
    pub fn new(rule: Rule) -> Self {
        match rule {
            Rule::colon => Self::Colon,
            Rule::dot => Self::Dot,
            Rule::alias => Self::Alias,
            Rule::bg => Self::Bg,
            Rule::break_builtin => Self::Break,
            Rule::cd => Self::Cd,
            Rule::command_builtin => Self::Command,
            Rule::continue_builtin => Self::Continue,
            Rule::eval => Self::Eval,
            Rule::exec => Self::Exec,
            Rule::exit_builtin => Self::Exit,
            Rule::export => Self::Export,
            Rule::fc => Self::Fc,
            Rule::fg => Self::Fg,
            Rule::getopts => Self::Getopts,
            Rule::hash_builtin => Self::Hash,
            Rule::jobs => Self::Jobs,
            Rule::kill => Self::Kill,
            Rule::pwd => Self::Pwd,
            Rule::read => Self::Read,
            Rule::readonly => Self::Readonly,
            Rule::return_builtin => Self::Return,
            Rule::set_builtin => Self::Set,
            Rule::shift => Self::Shift,
            Rule::test_builtin => Self::Test,
            Rule::times => Self::Times,
            Rule::trap => Self::Trap,
            Rule::type_builtin => Self::Type,
            Rule::ulimit => Self::Ulimit,
            Rule::umask => Self::Umask,
            Rule::unalias => Self::Unalias,
            Rule::unset => Self::Unset,
            Rule::wait => Self::Wait,
            r => panic!("should not have got {r:?}"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum CmdName {
    Builtin(Builtin),
    Path(PathBuf),
    Alias(String),
    Function(String),
    Unknown(String),
}
