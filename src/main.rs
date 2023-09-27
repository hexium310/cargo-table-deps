mod cli;
mod manifest;
mod metadata;

fn main() {
    if let Err(error) = cli::execute() {
        eprintln!("Error: {error}");
    }
}
