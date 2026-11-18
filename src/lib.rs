mod break_immunity;
mod efficacy;

#[skyline::main(name = "oligoludia_plugin")]
pub fn main() {
    break_immunity::install();
    efficacy::install();
    println!("[oligoludia_plugin] loaded; Break and efficacy immunity exceptions installed");
}
