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

async fn open_connection(server: &String, port: u16) -> anyhow::Result<BufReader<TcpStream>> {
    let address = format!("{}:{}", server, port);
    let stream = TcpStream::connect(address).await?;
    Ok(BufReader::new(stream))
}

fn read_line_as_bytes(buf: &mut dyn BufRead) -> anyhow::Result<Vec<u8>> {
    let mut line: Vec<u8> = Vec::new();
    buf.read_until(b'\n', &mut line)?;
    Ok(line)
}

async fn get_reply(reader: &mut BufReader<TcpStream>) -> anyhow::Result<String> {
    let mut reply = String::new();
    reader.read_line(&mut reply).await?;
    Ok(reply)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let options = Options::parse();

    let mut reader = open_connection(&options.server, options.port).await?;

    loop {
        let line = read_line_as_bytes(&mut io::stdin().lock())?;

        reader.write_all(&line).await?;

        let reply = get_reply(&mut reader).await?;
        print!("{reply}");
    }
}
