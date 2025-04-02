use std::{collections::HashMap, ffi::OsString, fmt::Display, path::PathBuf};

use rustyline::{Editor, history::FileHistory};

use crate::{
    AstError, ParsedPair, TrshResult,
    builtins::{BUILTINS, CmdName},
    prsr::Rule,
};

#[derive(Debug, Clone)]
pub struct SimpleCommand {
    pub name: CmdName,
    pub args: Vec<CmdArg>,
    pub redirections: Vec<Redirection>,
}

#[derive(Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Clone)]
pub enum Token {
    Word(String),
    Quote(String),
    VarExp(String),
    Eq,
    Neq,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum Redirection {
    AppendRight(String),
    Input(String),
    TruncRight(String),
    HereDoc(String),
}

impl Redirection {
    pub fn load_heredoc(delim: String, rl: &mut Option<&mut Editor<(), FileHistory>>) -> Self {
        if let Some(r) = rl {
            let mut input_str = String::new();
            loop {
                match r.readline("> ") {
                    Ok(s) => {
                        if s != delim {
                            input_str.push_str(&s);
                            input_str.push('\n');
                        } else {
                            break Self::HereDoc(input_str);
                        }
                    }
                    Err(e) => eprintln!("trsh: heredoc err: {e}"),
                }
            }
        } else {
            panic!("didn't get the prompt")
        }
    }
}

impl Display for Redirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Redirection::AppendRight(s) => write!(f, ">> {s}"),
            Redirection::Input(s) => write!(f, "< {s}"),
            Redirection::TruncRight(s) => write!(f, "> {s}"),
            Redirection::HereDoc(s) => write!(f, "<<{s}"),
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Eq => write!(f, "="),
            Token::Neq => write!(f, "!="),
            Token::VarExp(s) | Token::Word(s) | Token::Quote(s) => write!(f, "{s}"),
        }
    }
}

impl Token {
    pub fn new(a: ParsedPair) -> Self {
        match a.as_rule() {
            Rule::WORD => Self::Word(a.as_str().to_string()),
            Rule::QUOTE => Self::Quote(a.as_str().trim_matches('"').to_string()),
            Rule::EQ => Self::Eq,
            Rule::NEQ => Self::Neq,
            Rule::arg => Self::new(a.into_inner().next().unwrap()),
            Rule::VARIABLE_EXPANSION => {
                Self::VarExp(a.as_str().strip_prefix("$").unwrap().to_string())
            }
            r => panic!("{r:?}"),
        }
    }
    pub fn as_str(&self) -> &str {
        match self {
            Token::Word(s) => s,
            Token::VarExp(s) => s,
            Token::Quote(s) => s,
            Token::Eq => "=",
            Token::Neq => "!=",
        }
    }
}

impl std::borrow::Borrow<str> for Token {
    fn borrow(&self) -> &str {
        match self {
            Token::Word(s) => s,
            Token::VarExp(s) => s,
            Token::Quote(s) => s,
            Token::Eq => "=",
            Token::Neq => "!=",
        }
    }
}

impl AsRef<std::ffi::OsStr> for Token {
    fn as_ref(&self) -> &std::ffi::OsStr {
        match self {
            Token::Word(s) => std::ffi::OsStr::new(s),
            Token::Quote(s) => std::ffi::OsStr::new(s),
            Token::VarExp(s) => std::ffi::OsStr::new(s),
            Token::Eq => std::ffi::OsStr::new("="),
            Token::Neq => std::ffi::OsStr::new("!="),
            // ValidArg::Assignment(left, right) => std::ffi::OsStr::new(&format!("{left}={right}")),
        }
    }
}

impl SimpleCommand {
    pub fn new(
        rule: ParsedPair<'_>,
        env: (&HashMap<String, String>, &HashMap<String, String>),
        rl: &mut Option<&mut Editor<(), FileHistory>>,
    ) -> TrshResult<Self> {
        let mut parts = rule.into_inner();

        let parts_cmd = parts.next().unwrap();
        let parts_name = parts_cmd.as_str().trim();
        // println!("{parts_name}");
        let name = if parts_name.contains("/") {
            CmdName::Path(PathBuf::from(parts_name))
        } else if let Some(cmd) = env.0.get(parts_name) {
            CmdName::Alias(cmd.clone())
        } else if let Some(func) = env.1.get(parts_name) {
            CmdName::Function(func.clone())
        } else {
            match BUILTINS.get(parts_name) {
                Some(builtin) => CmdName::Builtin(*builtin),
                None => CmdName::Unknown(parts_name.to_owned()),
            }
        };

        let mut redirections = Vec::new();
        let mut tokens = Vec::new();
        for p in parts {
            match p.as_rule() {
                Rule::arg | Rule::VARIABLE_EXPANSION => tokens.push(Token::new(p)),
                Rule::APPEN_R => redirections.push(Redirection::AppendRight(
                    p.into_inner().next().unwrap().as_str().to_owned(),
                )),
                Rule::INPUT => redirections.push(Redirection::Input(
                    p.into_inner().next().unwrap().as_str().to_owned(),
                )),
                Rule::TRUNC_R => redirections.push(Redirection::TruncRight(
                    p.into_inner().next().unwrap().as_str().to_owned(),
                )),
                Rule::HEREDOC => redirections.push(Redirection::load_heredoc(
                    p.into_inner().next().unwrap().as_str().to_owned(),
                    rl,
                )),
                r => todo!("{r:?}"),
            }
        }
        let mut args: Vec<CmdArg> = Vec::new();
        // let mut assigns = Vec::new();
        let mut i = 0;
        while i < tokens.len() {
            match &tokens[i..] {
                [Token::Word(key), Token::Eq, Token::Quote(val)]
                | [Token::Word(key), Token::Eq, Token::Word(val)] => {
                    args.push(CmdArg::Assignment(key.clone(), val.clone()));
                    i += 3;
                }
                [Token::Word(word)] => {
                    args.push(CmdArg::Arg(word.clone()));
                    i += 1;
                }
                [Token::Quote(quote)] => {
                    args.push(CmdArg::Quoted(quote.clone()));
                    i += 1;
                }
                _ => break,
            }
        }
        tokens.drain(i..).for_each(|t| match t {
            Token::Word(s) => args.push(CmdArg::Arg(s)),
            Token::Quote(q) => args.push(CmdArg::Quoted(q)),
            Token::Eq => args.push(CmdArg::OpEq),
            Token::Neq => args.push(CmdArg::OpNeq),
            Token::VarExp(v) => args.push(CmdArg::Variable(v)),
        });
        Ok(Self {
            name,
            args,
            redirections,
        })
    }
}

#[derive(Clone, Debug)]
pub enum CmdArg {
    /// Regular positional argument like `ls`, `-l`, or `file.txt`
    Arg(String),

    /// A variable assignment like `FOO=bar`
    Assignment(String, String),

    /// A quoted argument like `"foo bar"` (preserves space, no expansion yet)
    Quoted(String),

    /// Logical operators used in `test` or `[` expressions
    OpEq,
    OpNeq,

    /// (Optional/future) Variable expansion like `$FOO`
    Variable(String),

    /// (Optional/future) Command substitution like `$(ls)`
    CommandSub(String),
}
impl CmdArg {
    pub fn as_str(&self) -> &str {
        match self {
            CmdArg::Arg(s) => s,
            CmdArg::Assignment(_, _) => todo!(),
            CmdArg::Quoted(_) => todo!(),
            CmdArg::OpEq => todo!(),
            CmdArg::OpNeq => todo!(),
            CmdArg::Variable(_) => todo!(),
            CmdArg::CommandSub(_) => todo!(),
        }
    }
    pub fn as_os_string(&self) -> OsString {
        match self {
            CmdArg::Arg(s) => std::ffi::OsString::from(s.to_string()),
            CmdArg::Assignment(l, r) => std::ffi::OsString::from(format!("{l}=\"{r}\"")),
            CmdArg::Quoted(q) => std::ffi::OsString::from(format!("\"{q}\"")),
            CmdArg::OpEq => std::ffi::OsString::from("="),
            CmdArg::OpNeq => std::ffi::OsString::from("!="),
            CmdArg::Variable(v) => std::ffi::OsString::from(v.to_string()),
            CmdArg::CommandSub(c) => std::ffi::OsString::from(c.to_string()),
        }
    }
}

impl Display for CmdArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CmdArg::Arg(s) => write!(f, "{s}"),
            CmdArg::Assignment(l, r) => write!(f, "{l}=\"{r}\""),
            CmdArg::Quoted(q) => write!(f, "\"{q}\""),
            CmdArg::OpEq => write!(f, "="),
            CmdArg::OpNeq => write!(f, "!="),
            CmdArg::Variable(v) => write!(f, "{v}"),
            CmdArg::CommandSub(c) => write!(f, "{c}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Conditional {
    pub condition: Box<Command>,
    pub then_branch: Box<Command>,
    pub else_branch: Option<Box<Command>>,
}

impl Conditional {
    fn new(
        rule: ParsedPair<'_>,
        env: (&HashMap<String, String>, &HashMap<String, String>),
        rl: &mut Option<&mut Editor<(), FileHistory>>,
    ) -> TrshResult<Self> {
        let mut parts = rule.into_inner();
        let condition = Box::new(Command::new(
            parts.next().ok_or(AstError::IncompleteConditional)?,
            env,
            rl,
        )?);
        let then_branch = Box::new(Command::new(
            parts.next().ok_or(AstError::IncompleteConditional)?,
            env,
            rl,
        )?);
        let else_branch = parts
            .next()
            .map(|p| Box::new(Command::new(p, env, rl).unwrap()));
        Ok(Self {
            condition,
            then_branch,
            else_branch,
        })
    }
}

#[derive(Debug, Clone)]
pub struct WhileLoop {
    pub condition: Box<Command>,
    pub body: Box<Command>,
}

#[derive(Debug, Clone)]
pub enum Command {
    Simple(SimpleCommand),
    Conditional(Conditional),
    Sequence(Vec<Self>),
    Pipeline(Box<Self>, Box<Self>),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
    WhileLoop(WhileLoop),
}
impl Command {
    pub fn new(
        rule: ParsedPair<'_>,
        env: (&HashMap<String, String>, &HashMap<String, String>),
        rl: &mut Option<&mut Editor<(), FileHistory>>,
    ) -> TrshResult<Self> {
        Ok(match rule.as_rule() {
            Rule::program => todo!(),
            Rule::command_list => {
                let mut v = Vec::new();
                for r in rule.into_inner() {
                    v.push(Self::new(r, env, rl)?);
                }
                Self::Sequence(v)
            }
            Rule::if_clause => Self::Conditional(Conditional::new(rule, env, rl)?),
            Rule::simple_command => Self::Simple(SimpleCommand::new(rule, env, rl)?),
            Rule::WHITESPACE => todo!(),
            Rule::NEWLINE => todo!(),
            Rule::command => todo!(),
            Rule::command_name => todo!(),
            Rule::pipeline => {
                let mut segments = rule.into_inner().map(|r| Command::new(r, env, rl));
                let first = segments.next().unwrap()?;
                segments.try_fold(first, |left, right_res| -> TrshResult<Self> {
                    let right = right_res?;
                    Ok(Command::Pipeline(Box::new(left), Box::new(right)))
                })?
                // todo!()
            }
            Rule::and_or => {
                // println!("{rule:?}");
                let mut iter = rule.into_inner();
                let mut left = Command::new(iter.next().unwrap(), env, rl)?;

                while let Some(op) = iter.next() {
                    let right = Command::new(iter.next().unwrap(), env, rl)?;
                    left = match op.as_str() {
                        "&&" => Command::And(Box::new(left), Box::new(right)),
                        "||" => Command::Or(Box::new(left), Box::new(right)),
                        _ => unreachable!(),
                    };
                }
                left
            }
            Rule::test_cond => Self::Simple(SimpleCommand::new(rule, env, rl)?),
            Rule::while_loop => {
                let mut iter = rule.into_inner();
                Self::WhileLoop(WhileLoop {
                    condition: Box::new(Self::new(iter.next().unwrap(), env, rl)?),
                    body: Box::new(Self::new(iter.next().unwrap(), env, rl)?),
                })
            }
            // Rule::pipe_segment => Self::new(rule.into_inner().next().unwrap(), env, rl)?,
            l => todo!("{:?}", l),
        })
    }
}
// fn parse_pipeline(pair: Pair<Rule>) -> AstResult<Command> {
//     let mut segments = pair.into_inner().map(Command::new);
//
//     let first = segments.next().unwrap()?; // always exists (leftmost command)
//
//     segments.try_fold(first, |left, right_res| {
//         let right = right_res?;
//         Ok(Command::Pipeline(Box::new(left), Box::new(right)))
//     })
// }
