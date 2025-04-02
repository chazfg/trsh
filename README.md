## Trsh

its uh.. its a rust shell

todo:
aliases
variable expansion
.trshrc
redirection
pipelines

# trsh Development Roadmap

For most of these, take them as a _very_ loose interpretation of "Complete". Until I start digging into
phase 5 I don't think I'll worry much about complete end to end testing. So it's mostly just
"This works for a few of the commands I tried with it"

## Phase 1: Core (done or nearly done)

- [x] REPL w/ prompt
- [x] Command parsing
- [x] Builtin detection
- [x] Builtins: `cd`, `pwd`, `alias`, `unalias`, `export`, `unset`, `exit`
- [x] SIMPLE `.trshrc` support
- [x] `-c [script_file]`

## Phase 2: (Simple) Execution Semantics + (Simple) Control Flow

- [x] Append (Redirection): >>
- [x] Truncate (Redirection): >
- [x] HereDoc (Redirection): <<
- [x] Input (Redirection): <
- [x] Pipes
- [x] Command Sequencing
- [x] Conditionals (if, then, else, fi)
- [x] Conditionals (with test/ [)
- [x] Logic: Ands (&&), ors (||)

## Phase 3: Shell Language Features

- [x] Variable Expansion $VAR
- [ ] loops (while, for)
- [ ] Command Sub $(...)
- [ ] Comprehensive Quote Handling

## Phase 4: POSIX Compatibility & Testability

- [ ] arithmetic sub $((...))
- [ ] Incorporate POSIX compat testing
- [ ] Start polish/comprehensive testing
- [ ] Ensure correctness of exit codes
- [ ] Script-local scoping
- [ ] History and Job Control

## Phase 5: Stretch

- [ ] functions
- [ ] grouping
- [ ] $1 things for function args
- [ ] subshells
- [ ] i/o redir w/ FD
- [ ] !, history, things like that
- [ ] plugins?
