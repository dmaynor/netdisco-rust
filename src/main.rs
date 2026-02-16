//! Default binary - shows help text.

fn main() {
    println!("Netdisco {} - Network Management Tool (Rust Edition)", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Available commands:");
    println!("  netdisco-web      Start the web frontend server");
    println!("  netdisco-backend  Start the backend job control daemon");
    println!("  netdisco-do       Run ad-hoc operations from the CLI");
    println!("  netdisco-deploy   Deploy/upgrade the database schema");
    println!();
    println!("For more information, run any command with --help");
}
