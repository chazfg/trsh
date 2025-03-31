use std::{os::unix::process::ExitStatusExt, path::Path, process::ExitStatus};

use crate::TrshResult;

pub static BINARY_TESTS: phf::Map<&'static str, BinaryTest> = phf::phf_map! {
    "=" => BinaryTest::Eq,
    "!=" => BinaryTest::Neq,
    "-eq" => BinaryTest::Eq,
    "-ne" => BinaryTest::Neq,
    "-gt" => BinaryTest::Gt,
    "-lt" => BinaryTest::Lt,
    "-ge" => BinaryTest::GtEq,
    "-le" => BinaryTest::LtEq,
};

pub enum BinaryTest {
    Eq,
    Neq,
    Gt,
    GtEq,
    Lt,
    LtEq,
}

impl BinaryTest {
    pub fn compare(&self, left: &str, right: &str) -> TrshResult<ExitStatus> {
        let left_num = match left.parse::<i64>() {
            Ok(i) => i,
            Err(_) => return Ok(exit_num(2)),
        };
        let right_num = match right.parse::<i64>() {
            Ok(i) => i,
            Err(_) => return Ok(exit_num(2)),
        };
        if match self {
            BinaryTest::Eq => left_num == right_num,
            BinaryTest::Neq => left_num != right_num,
            BinaryTest::Gt => left_num > right_num,
            BinaryTest::GtEq => left_num >= right_num,
            BinaryTest::Lt => left_num < right_num,
            BinaryTest::LtEq => left_num <= right_num,
        } {
            Ok(exit_zero())
        } else {
            Ok(exit_num(1))
        }
    }
}

pub static UNARY_TESTS: phf::Map<&'static str, fn(&str) -> bool> = phf::phf_map! {
    "-f" => |path| Path::new(path).is_file(),
    "-d" => |path| Path::new(path).is_dir(),
    "-e" => |path| Path::new(path).exists(),
    "-n" => |s| !s.is_empty(),
    "-z" => |s| s.is_empty(),
};

pub fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|meta| meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

pub fn exit_zero() -> ExitStatus {
    ExitStatus::from_raw(0)
}

pub fn exit_num(i: i32) -> ExitStatus {
    ExitStatus::from_raw(i)
}
