#[macro_use]
extern crate lazy_static;

mod chemicals;
mod parser;
mod calculator;
mod compiler;

use chemicals::{Chemical, ChemToken};
use calculator::Action;

use std::{
    fs,
    io::{self, Write},
};

// (Buf) Uncomment these lines to have the output buffered, this can provide
// better performance but is not always intuitive behaviour.
// use std::io::BufWriter;

use structopt::StructOpt;

// Our CLI arguments. (help and version are automatically generated)
// Documentation on how to use:
// https://docs.rs/structopt/0.2.10/structopt/index.html#how-to-derivestructopt
#[derive(StructOpt, Debug)]
struct Cli {
    input:String
}


fn main() {
    let args = Cli::from_args();
    let chem =  parser::parse(args.input).unwrap();
    let tree = calculator::ChemTree::deconstruct(&chem);
    println!("{}\n\n", tree.initial_state.to_text());
    let actions = calculator::compute_actions(&tree);
    println!("{:?}\n", actions);
    let commands = compiler::compile(&actions);
    println!("{:?}\n", commands);
    let code = compiler::to_bytecode(&commands);
    println!("{}", code);
}



