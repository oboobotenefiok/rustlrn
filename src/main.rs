//! This is the entry point of the program and will mostly act as a 'pointer' :-)

//  //! means comment on this file(module)
//  /// means documentation comment
// Well, just an inline
/* */ 
// <--And That One Above Is Multi-line comment


// You must declare the type of constants, and also best practice to keep it capital.
// My major use case for constants is for source of truth otherwise you'll almost NEVER see me use it.
// Quick one is, you cannot use the to_string() or  string::from("") on CONST cause of compile time needs. We'll use &str.

// JUST KNOW THAT HEAP ALLOCATION CAN'T HAPPEN AT COMPILE TIME.
const NAME:&str = "rustlrn";
/* Spectra, Obot here... feel we can make this a CLI tool for learning Rust so I'll add actual stuff to main and we can iterate from there. Think it will be cool that way? I'm not versed in the contribution culture but for now this comment should be fine :-) We can learn a lot from implementing this and fuse it to an app or web later when we understand better...*/


// That reminds me, this binary will handroll its own interface. and many other features.
use clap::Parser;

/// In the last one I used 
/// `use colored::*` 
// But that led to errors. I had to change it to Colorize. Note that it begins with capital C
use colored::Colorize;
/// The main function will basically be kept as empty as possible and we'll route several things to other modules.
/// This will serve as one of the evidence of understanding handrolling.
// At first, all we have to do is make the program move from one step to another, like PREVIOUS, NEXT!
// Then we make it feel as instant and re-filling as possible.
// That will be enough proof-of-work!!!!
// I would have used the terminal ANSI colors but since we are here for Rust, we'll......PLEASE NOTE THAT THE COMMENTS IN HERE WILL SWITCH BETWEEN WE, I, OUR, ME, MY , US, INTERCHANGEABLY...
// Like I was saying, we are are going to use the colored crate instead of ANSI.

// We bring it in because it is not in the PRELUDE exposed by the compiler.

// A PRELUDE is the ATMOSPHERE of the compiler you are experiencing right now. If you still don't understand that, come back to this comment in future.
use std::io::{self, Write};
// This derive gave me headache

/// Trait for clap needs derive to be implemented for certain stuff. Be sure to keep an eye at Cargo.toml.
// I had to run `cargo add clap --features derive` to get it.
// Also there are a lot of ambiguities in crate versions. It works for now so no problem.
#[derive(Parser)]
#[command(author, version, about = "Rust Tutor - Learn Rust interactively", long_about = None)]
struct Cli {
    /// Starting lesson number (1-5)
    #[arg(short, long, default_value_t = 1, value_parser = clap::value_parser!(u8).range(1..=5))]
    lesson: u8,
}

fn main() {
    let cli = Cli::parse();

    let lessons = [
        "Lesson 1: Hello World - print!(\"Hello, world!\")",
        "Lesson 2: Variables - let x = 5;",
        "Lesson 3: Functions - fn add(a: i32, b: i32) -> i32 { a + b }",
        "Lesson 4: If/Else - if x > 0 { println!(\"Positive\"); }",
        "Lesson 5: Loops - for i in 0..5 { println!(\"{}\", i); }",
    ];

    let mut current = (cli.lesson - 1) as usize;
    let mut warn_count = 0;

// I don't know how long we'll keep this loop but this will be like the main place we keep users while in the app... The app will be a loop basically.
    loop {
        clear_screen(); // We clear screen each time the loop begins. So far, it's not any noticeable if the screen flickered and seeing that we will be working with just text, I don't even see that coming.
        // I can perceive a very crazy state machine being built inside this loop in future but let's see.
        println!("{}", NAME.cyan().bold());
        println!("{}\n", lessons[current]);
        println!("{}", "Press 'n' for next, 'p' for previous, 'q' to quit".bold().green());

        let warn = warn_count > 0;
        if warn {
            eprintln!("{}", "Please type a valid input".red().bold());
            warn_count = 0;
        }

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let next = current < lessons.len() - 1;
        let previous = current > 0;

        match input.trim() {
            "n" => {
                if next {
                    current += 1;
                } else {
                    warn_count += 1;
                }
            }
            "p" => {
                if previous {
                    current -= 1;
                } else {
                    warn_count += 1;
                }
            }
            "q" => break,
            _ => {
                warn_count += 1;
                continue;
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn clear_screen() {
    std::process::Command::new("cmd")
        .args(&["/c", "cls"])
        .status()
        .unwrap();
}

#[cfg(not(target_os = "windows"))]
fn clear_screen() {
    std::process::Command::new("clear").status().unwrap();
}
