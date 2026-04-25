fn main() {
    if let Err(error) = electrification::terminal::run() {
        eprintln!("terminal error: {error}");
        std::process::exit(1);
    }
}
