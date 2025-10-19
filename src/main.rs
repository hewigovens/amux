use amux::run;

fn main() {
    if let Err(err) = run() {
        eprintln!("amux: {}", err);
        std::process::exit(1);
    }
}
