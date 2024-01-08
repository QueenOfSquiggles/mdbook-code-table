use std::{io, process};

use clap::Command;
use mdbook::{
    errors::Error,
    preprocess::{CmdPreprocessor, Preprocessor},
};

mod table;

fn make_app() -> Command {
    Command::new("code-table")
        .about("A mdbook preprocessor that allows fenced code blocks in your markdown tables")
}

fn main() {
    make_app();
    let prep = table::CodeTables;
    if let Err(e) = handle_processing(&prep) {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn handle_processing(pre: &dyn Preprocessor) -> Result<(), Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;
    let processed = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed)?;
    Ok(())
}
