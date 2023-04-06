use clap::Parser;
use std::io;
use std::io::BufRead;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::net::TcpStream;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Options {
    #[arg(short, long)]
    server: String,
    #[arg(short, long, default_value_t = 9090)]
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let options = Options::parse();

    let address = format!("{}:{}", options.server, options.port);
    let stream = TcpStream::connect(address).await?;
    let mut reader = BufReader::new(stream);

    loop {
        let mut buf: Vec<u8> = Vec::new();
        io::stdin().lock().read_until(b'\n', &mut buf)?;

        reader.write_all(&buf).await?;

        let mut reply = String::new();
        reader.read_line(&mut reply).await?;
        print!("{reply}");
    }
}
