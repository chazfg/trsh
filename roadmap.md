# trsh Development Roadmap

## Phase 1: Core (done or nearly done)

- [x] REPL w/ prompt
- [x] Command parsing
- [x] Builtin detection
- [x] Builtins: `cd`, `pwd`, `alias`, `unalias`, `export`, `unset`, `exit`
- [ ] `.trshrc` support
- [x] `-c` / `[script_file]`

## Phase 2: (Simple) Execution Semantics

- [x] Append (Redirection): >>
- [x] Truncate (Redirection): >
- [ ] HereDoc (Redirection): <<
- [x] Input (Redirection): <
- [ ] Pipes
- [ ] Command Sequencing

## Phase 3: Additional Shell Language

- [ ] Conditionals (if, then, else, fi)
- [ ] loops (while, for)
- [ ] functions

## Phase 4: Usability & Scripting

- [ ] Variable Expansion $VAR
- [ ] Command Sub $(...)
- [ ] Comprehensive Quote Handling
- [ ] History and Job Control

## Phase 5: POSIX Compatibility & Testability

- [ ] Incorporate POSIX compat testing
- [ ] Ensure correctness of exit codes
- [ ] Script-local scoping

## Stretch Goals

- [ ] subshells
- [ ] i/o redir w/ FD
- [ ] !, history, things like that
- [ ] plugins?
