#[macro_use]
extern crate lazy_static;

mod chemicals;
mod parser;
mod calculator;
mod compiler;

use chemicals::{Chemical, ChemToken, NumberToken, NumberOperator};
use calculator::{Action, ChemState};

// (Buf) Uncomment these lines to have the output buffered, this can provide
// better performance but is not always intuitive behaviour.
// use std::io::BufWriter;

use structopt::StructOpt;

/// Example inputs. Mostly not real recipes. Dollar signs are essentially substituted for the quantity of the parent: 
/// 
/// 50:($/3:nitrogen;$/1:hydrogen;)
/// 
/// 50:*METH;
/// 
/// 25:($/2:*OIL;$/3:*METH;)@374;
#[derive(StructOpt, Debug)]
struct Cli {
    #[structopt(flatten)]
    command:Command
}

#[derive(StructOpt, Debug)]
enum Command {
    /// Calculate the compiled chemfuck code based on the input
    Calc{
        input:String
    },
    /// List known premade chem formulas that are available to substitute.
    List {}
}


fn main() {
    let args = Cli::from_args();
    match args.command {
        Command::Calc {input} => {
            let (chem, total_quantity) =  parser::parse(input);
            let chem = chem.unwrap();
            let tree = calculator::ChemTree::deconstruct(&chem);
            let mut tree = tree.clone();
            tree.initial_state.multiply(total_quantity);
            let (actions, sizes) = calculator::compute_actions(&tree, total_quantity);
            print_required_state(&sizes, &tree.initial_state);
            println!("{:?}\n", actions);
            let commands = compiler::compile(&actions);
            println!("{:?}\n", commands);
            let code = compiler::to_bytecode(&commands);
            println!("{}", code);   
        },
        Command::List {} => {
            for chemical_name in parser::SUB_MAP.keys() {
                println!("{}", chemical_name);
            }
        }
    }
}

fn print_required_state(sizes:&Vec<u32>, state:&ChemState) {
    for i in 0..sizes.len() {
        let state = state.get(i);
        let name:String = if state.contents.is_some() {state.contents.as_ref().unwrap().chemical.name.as_ref().unwrap_or(&"None".to_string()).to_string()} else {"None".to_string()};
        let amount = if state.contents.is_some() {state.contents.as_ref().unwrap().concrete_quantity.unwrap()} else {0};
        let size = sizes[i];
        println!("r{}: ({}/{}) {}", i+1,amount,size, name);
    }
}

