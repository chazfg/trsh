# trsh Development Roadmap

---

## Phase 1: Core (done or nearly done)

| Feature                                                              | Status                                     |
| -------------------------------------------------------------------- | ------------------------------------------ |
| REPL w/ prompt                                                       | `rustyline` [x]                            |
| Command parsing                                                      | `pest` grammar + AST [x]                   |
| Builtin detection                                                    | Using `phf`, dispatched in executor [x]    |
| Builtins: `cd`, `pwd`, `alias`, `unalias`, `export`, `unset`, `exit` | Implemented [x]                            |
| `.trshrc` support                                                    | Optional config loading [ ]                |
| `-c` / `[script_file]`                                               | Non-interactive mode / shebang support [x] |

---

## Phase 2: (Simple) Execution Semantics

| Feature                   | Status |
| ------------------------- | ------ |
| Append (Redirection): >>  | [x]    |
| Truncate (Redirection): > | [x]    |
| HereDoc (Redirection): << | [ ]    |
| Input (Redirection): <    | [x]    |
| Pipes                     | [ ]    |
| Command Sequencing        | [ ]    |

---

## Phase 3: Additional Shell Language

| Feature                           | Status |
| --------------------------------- | ------ |
| Conditionals (if, then, else, fi) | [ ]    |
| loops (while, for)                | [ ]    |
| functions                         | [ ]    |

- ***

## Phase 4: Usability & Scripting

| Feature                      | Status |
| ---------------------------- | ------ |
| Variable Expansion $VAR      | [ ]    |
| Command Sub $(...)           | [ ]    |
| Comprehensive Quote Handling | [ ]    |
| History and Job Control      | [ ]    |

- ***

## Phase 5: POSIX Compatibility & Testability

| Feature                          | Status |
| -------------------------------- | ------ |
| Incorporate POSIX compat testing | [ ]    |
| Ensure correctness of exit codes | [ ]    |
| Script-local scoping             | [ ]    |

- ***

## Stretch Goals

| Feature                      | Status |
| ---------------------------- | ------ |
| subshells                    | [ ]    |
| i/o redir w/ FD              | [ ]    |
| !, history, things like that | [ ]    |
| plugins?                     | [ ]    |

-
