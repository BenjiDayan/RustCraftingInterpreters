use std::io;
use std::io::Write;
use std::env;


mod token_type;
use token_type::TokenType;
use token_type::Token;
use token_type::Literal;

mod lox;
use lox::ast::{Expr, Binary, parser};
use lox::ast::interpreter::{RuntimeError, Interp};
mod scanner;

static mut HAD_ERROR: bool = false;
static mut HAD_RUNTIME_ERROR: bool = false;

static mut my_interpreter: std::sync::LazyLock<Interp> = std::sync::LazyLock::new(|| Interp::new());


fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(&args);
    println!("args length: {}", args.len());

    
    let word = "Prog";
    println!("{}", word.chars().nth(3).unwrap());

    println!("token type: {:?}", TokenType::LEFT_PAREN);
    println!("{}", 1.0/0.0);


    // let boo: Binary = Binary {
    //     left: Box::new(Expr),
    //     right: Box::new(Expr),
    //     operator: Token()
    // };


    let token = Token::new(
        TokenType::STRING,
        "example".to_string(),
        // Some(Literal::String("hello".to_string())),
        Literal::String("hello".to_string()),
        1,
    );

    println!("token: {:?}", token.to_string());


    if args.len() > 2 {
        println!("Usage: jlox [script]");
        std::process::exit(1);
    } else if args.len() == 2 {
        run_file(args[1].clone());
    } else {
        run_prompt();
    }



    // println!("Guess the number!");

    // let secret_number = rand::thread_rng().gen_range(1..=100);

    // // println!("The secret number is {secret_number}");


    // loop {
    //     println!("Please input your guess.");

    //     let mut guess: String = String::new();

    //     io::stdin()
    //         .read_line(&mut guess)
    //         .expect("Failed to read line");

    //     // let guess: u32 = guess.trim().parse().expect("Please type a number!");
    //     let guess: u32 = match guess.trim().parse() {
    //         Ok(num) => num,
    //         Err(_) => continue,
    //     };


    //     println!("You guessed: {}", guess);

    //     match guess.cmp(&secret_number) {
    //         Ordering::Less => println!("Too small!"),
    //         Ordering::Greater => println!("Too big!"),
    //         Ordering::Equal => {
    //             println!("You win!");
    //             break;
    //         },
    //     }
    // }

}

fn run_file(path: String) {
    let contents = std::fs::read_to_string(&path).expect("Should have read the file!");
    // println!("runFile read {path} and got:\n{contents}!");

    run(&contents);
    unsafe{
    if HAD_ERROR {
        std::process::exit(65);
    }
    if HAD_RUNTIME_ERROR {
        std::process::exit(70);
    }
    }
}

fn run_prompt() {
    loop {
        print!("> ");
        io::stdout().flush().expect("aaaah");
        let mut repl_input: String = String::new();

        match io::stdin()
            .read_line(&mut repl_input)
            {
                Ok(0) => {
                    break;
                }
                Ok(_) => {
                    run(&repl_input);
                }
                Err(error) => {
                    eprintln!("IDK??");
                }
            }
        // println!("You entered: {repl_input}");
    }
}

fn run(source: &String) {
    // println!("{line}");
    let mut S = scanner::Scanner::new(source.clone());

    let tokens: Vec<Token> = S.scan_tokens();
    for token in &tokens {
        println!("{:?}", token);
    }

    let mut my_parser = parser::Parser::new(tokens);
    // let expr = my_parser.parse().unwrap();

    // unsafe {
    //     my_interpreter.interpret(&expr);
    // }

    let stmts = my_parser.parse();
    unsafe {
        my_interpreter.get_mut().interpret_stmts(&stmts);
    }
}

pub fn error(line: usize, message: &str) {
    report(line, "", message);
}

fn report(line: usize, location: &str, message: &str) {
    eprintln!("[line {}] Error{}: {}", line, location, message);
}

fn runtime_error(err: RuntimeError) {
    println!("{err:?}\n[line {}]", err.token.line);
    unsafe { HAD_RUNTIME_ERROR = true };
}