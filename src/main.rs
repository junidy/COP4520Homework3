mod presents;
mod temperature;

use std::io;

use presents::sort_presents;
use temperature::run_temperature_module;

fn main() {
    // Problem 1
    sort_presents(500_000, 4);

    println!("\nPress enter to continue...\n");

    io::Write::flush(&mut io::stdout()).unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    // Problem 2
    run_temperature_module(36000, 8);
}
