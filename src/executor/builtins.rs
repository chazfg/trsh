use std::{
    os::unix::process::ExitStatusExt,
    process::{ExitStatus, Stdio},
};

use crate::{TrshError, TrshResult, ast::CmdArg, builtins::Builtin, executor::exit_zero};

use super::{
    Executor,
    utils::{BINARY_TESTS, UNARY_TESTS},
};

impl Executor {
    pub fn exec_builtin(
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
}
