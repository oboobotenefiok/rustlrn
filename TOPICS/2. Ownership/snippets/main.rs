 fn main() {
    println!("Welcome to the Rust Learning Journey!");
   println!("Check the 'topics' folder for explanations and 'examples' for code. \n Explanatios are named README while examples are in SNIPPETS");
   start();
  }
  
  fn start() {
    status();
}

fn status() {
    let indie_hacker = String::from("Caleb");
    
    let underdog_builder = indie_hacker; 
  
    println!("This is the name: {}", underdog_builder);
    
}
