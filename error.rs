fn main() {
    let indie_hacker = String::from("Caleb");
    
    let underdog_builder = indie_hacker; 
    
    println!("This wikl throw error? {}", indie_hacker);
    
    println!("New owner is: {}", underdog_builder);
}
