fn main() {
    let shlum_peace = String::from("שלום");
    let olam_world = String::from("עולם");
    let string = format!("Peace: {shlum_peace}, world: {olam_world}.");
    println!("Whole: {string}.");

    for char in string.chars(){
        println!("{}",char);
    }
}

