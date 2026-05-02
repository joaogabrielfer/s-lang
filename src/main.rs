mod error;
mod lexer;
mod value;
mod parser;

use std::{collections::HashMap, error::Error, fs, io::{self, Write}, process::exit, vec};

use lexer::*;
use value::*;
use parser::*;

fn main() -> Result<(), Box<dyn Error>>{
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        let mut input = String::new();
        let mut stack: Vec<RuntimeValue> = vec![];
        let mut variables: HashMap<String, i32> = HashMap::new();
        let mut functions: HashMap<String, Vec<Token>> = HashMap::new();
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

            if let Err(e) = parse(tokens.clone(), &mut stack, &mut variables, &mut functions){
                eprintln!("ERROR: {e}")
            } else {
                if tokens[tokens.len() - 1] == Token::Pop{
                    println!()
                }
                println!("stack: {:?}", stack);
            }

            input.clear();
        }
    } else {
        let content = fs::read_to_string(args[1].clone())?;
        let mut stack : Vec<RuntimeValue> = vec![];
        let mut variables: HashMap<String, i32> = HashMap::new();
        let mut functions: HashMap<String, Vec<Token>> = HashMap::new();
        let tokens = tokenize(content.clone());
        if cfg!(feature = "token-logging"){
            #[cfg(feature = "token-logging")]
            log_tokens(tokens.clone());
            exit(0);
        }

        if let Err(e) = parse(tokens, &mut stack, &mut variables, &mut functions){
            eprintln!("ERROR: {e}");
            exit(0);
        }


        if !stack.is_empty(){
            println!("warning: trailing number still in the stack: {:?}", stack);
        }
    }

    Ok(())
}


#[cfg(feature = "token-logging")]
fn log_tokens(tokens: Vec<Token>){
    tokens
        .iter()
        .for_each(|tk| println!("{:?}", tk));
}
