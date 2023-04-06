use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Options {
    #[arg(short, long)]
    server: String,
    #[arg(short, long, default_value_t = 9000)]
    port: u16,
}

fn main() {
    let options = Options::parse();
    println!("server: {}, port: {}", options.server, options.port);
}
