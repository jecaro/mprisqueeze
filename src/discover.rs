use anyhow::Result;
use log::{info, warn};
use nom::{
    bytes::{self, complete::tag},
    combinator::{flat_map, map, map_res},
    number,
    sequence::preceded,
    IResult, Parser,
};
use std::time::Duration;
use tokio::{net::UdpSocket, time::timeout};

#[derive(Debug)]
pub struct Reply {
    pub hostname: String,
    pub ip: std::net::IpAddr,
    pub port: u16,
    #[allow(dead_code)]
    pub uuid: String,
    pub version: String,
}

// The LMS server can be discovered by sending a broadcast UDP packet to port 3483.
// Example of answer from LMS
// "ENAME\u{10}myhostnameJSON\u{4}9000UUID$e9b557b8-92e2-45cd-8a95-8730ffd604a5VERS\u{5}8.3.1"
// '$' = 36 in the ASCII table
// Each value starts with a tag, followed by the length of the value in one byte, then the value
// itself in the next length bytes.

/// Discover the LMS server on the local network
pub async fn discover(reply_timeout: Duration) -> Result<Reply> {
    info!("Discovering LMS server on the local network");

    let sock = UdpSocket::bind("0.0.0.0:0").await?;
    sock.set_broadcast(true)?;

    let mut buf = [0; 1024];

    let ip;

    loop {
        let response = timeout(reply_timeout, broasdcast_and_recv(&mut buf, &sock)).await;
        match response {
            Ok(Ok(found_ip)) => {
                ip = found_ip;
                break;
            }
            Ok(Err(e)) => return Err(e.into()),
            Err(_) => warn!("Timeout waiting for LMS reply, retrying..."),
        }
    }

    parse_reply(&buf, ip)
        .map(|(_, reply)| {
            info!(
                "Found LMS server: {}:{} ({})",
                reply.hostname, reply.port, reply.version
            );
            reply
        })
        .map_err(|error| error.to_owned().into())
}

async fn broasdcast_and_recv(buf: &mut [u8], sock: &UdpSocket) -> Result<std::net::IpAddr> {
    let message = "eNAME\0JSON\0UUID\0VERS\0".as_bytes();
    let _ = sock.send_to(&message, "255.255.255.255:3483").await?;
    let (_, src) = sock.recv_from(buf).await?;
    Ok(src.ip())
}

fn parse_tag<'a>(input: &'a [u8], start_tag: &str) -> IResult<&'a [u8], String> {
    map_res(
        preceded(
            tag(start_tag),
            flat_map(number::complete::be_u8, bytes::complete::take),
        ),
        |bytes: &[u8]| String::from_utf8(bytes.to_vec()),
    )
    .parse(input)
}

fn parse_hostname(input: &[u8]) -> IResult<&[u8], String> {
    parse_tag(input, "ENAME")
}

fn parse_port(input: &[u8]) -> IResult<&[u8], u16> {
    map_res(|input| parse_tag(input, "JSON"), |s| s.parse::<u16>()).parse(input)
}

fn parse_uuid(input: &[u8]) -> IResult<&[u8], String> {
    parse_tag(input, "UUID")
}

fn parse_version(input: &[u8]) -> IResult<&[u8], String> {
    parse_tag(input, "VERS")
}

fn parse_reply(input: &[u8], ip: std::net::IpAddr) -> IResult<&[u8], Reply> {
    map(
        (parse_hostname, parse_port, parse_uuid, parse_version),
        |(hostname, port, uuid, version)| Reply {
            hostname,
            port,
            uuid,
            version,
            ip,
        },
    )
    .parse(input)
}
