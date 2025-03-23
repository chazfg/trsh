use crate::{AstError, ParsedPair, TrshResult, builtins::CmdName, prsr::Rule};

#[derive(Debug)]
pub struct SimpleCommand {
    pub name: CmdName,
    pub args: Vec<String>,
}

impl SimpleCommand {
    pub fn new(rule: ParsedPair<'_>) -> TrshResult<Self> {
        let mut parts = rule.into_inner();
        let name = CmdName::new(parts.next().unwrap());

        let mut args = Vec::new();
        for r in parts {
            args.push(r.as_str().to_owned());
        }
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
    fn new(rule: ParsedPair<'_>) -> TrshResult<Self> {
        let mut parts = rule.into_inner();
        let condition = Box::new(Command::new(
            parts.next().ok_or(AstError::IncompleteConditional)?,
        )?);
        let then_branch = Box::new(Command::new(
            parts.next().ok_or(AstError::IncompleteConditional)?,
        )?);
        let else_branch = Box::new(Command::new(
            parts.next().ok_or(AstError::IncompleteConditional)?,
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
    pub fn new(rule: ParsedPair<'_>) -> TrshResult<Self> {
        Ok(match rule.as_rule() {
            Rule::program => todo!(),
            Rule::command_list => {
                let mut v = Vec::new();
                for r in rule.into_inner() {
                    v.push(Self::new(r)?);
                }
                Self::Sequence(v)
            }
            Rule::if_clause => Self::Conditional(Conditional::new(rule)?),
            Rule::simple_command => Self::Simple(SimpleCommand::new(rule)?),
            Rule::WHITESPACE => todo!(),
            Rule::NEWLINE => todo!(),
            Rule::command => todo!(),
            Rule::command_name => todo!(),
            Rule::argument => todo!(),
            l => todo!("{:?}", l),
        })
    }
}
