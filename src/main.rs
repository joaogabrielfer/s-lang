mod error;
mod lexer;
mod value;
mod parser;

use std::{error::Error, fs, io::{self, Write}, process::exit};

use lexer::*;
use value::*;
use parser::*;

fn main() -> Result<(), Box<dyn Error>>{
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        let mut input = String::new();
        let mut pvm = PVM::new();
        loop{
            print!("> ");
            io::stdout().flush()?;
            io::stdin().read_line(&mut input)?;

            let tokens= tokenize(input.clone());
            if cfg!(feature = "token-logging"){
                    #[cfg(feature = "token-logging")]
                    log_tokens(tokens.clone());
                    exit(0);
            }

            match pvm.parse(tokens.clone()){
                Ok(Flow::Return) => return Ok(()),
                Err(e) => {
                    eprintln!("ERROR: {e}");
                    print_stack(&pvm.stack);
                }
                _ => {
                    if tokens[tokens.len() - 1] == Token::Pop{
                        println!()
                    }
                    print_stack(&pvm.stack);
                }
            }
            input.clear();
        }
    } else {
        let content = fs::read_to_string(args[1].clone())?;
        let mut pvm = PVM::new();
        let tokens = tokenize(content.clone());
        if cfg!(feature = "token-logging"){
            #[cfg(feature = "token-logging")]
            log_tokens(tokens.clone());
            exit(0);
        }

        match pvm.parse(tokens){

            Ok(Flow::Return) => return Ok(()),
            Err(e) => {
                eprintln!("ERROR: {e}");
                exit(0);
            }
            _ => { }
        }


        if !pvm.stack.is_empty(){
            println!("warning: trailing number still in the stack: {:?}", pvm.stack);
        }
    }

    Ok(())
}

fn print_stack(stack: &[RuntimeValue]){
    print!("stack: [");
    stack
        .iter()
        .enumerate()
        .for_each(|(i, x)| {
            match x{
                RuntimeValue::String(s) => print!("\"{s}\""),
                x => print!("{x}") // TODO: não coloca virgulas caso o mesmo numero
            }
            if i != stack.len() - 1{
                print!(", ");
            }
        });
    println!("]");
}

#[cfg(feature = "token-logging")]
fn log_tokens(tokens: Vec<Token>){
    tokens
        .iter()
        .for_each(|tk| println!("{:?}", tk));
}
