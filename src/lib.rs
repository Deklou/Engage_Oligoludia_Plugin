mod break_immunity;

#[skyline::main(name = "oligoludia_plugin")]
pub fn main() {
    break_immunity::install();
    println!("[oligoludia_plugin] loaded; Break immunity exception installed");
}
