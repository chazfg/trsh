use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "bash.pest"]
pub struct TrshPrsr;
