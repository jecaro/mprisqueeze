use anyhow::Result;
use log::{info, warn};
use nom::{
    IResult, Parser,
    bytes::{self, complete::tag},
    combinator::{flat_map, map, map_res},
    number,
    sequence::preceded,
};
use std::{net::IpAddr, time::Duration};
use tokio::{net::UdpSocket, time::timeout};

#[derive(Debug)]
pub struct Reply {
    pub hostname: String,
    pub port: u16,
    #[allow(dead_code)]
    pub uuid: String,
    #[allow(dead_code)]
    pub version: String,
}

// The LMS server can be discovered by sending a broadcast UDP packet to port 3483.
//
// Example of answer from LMS
// "ENAME\u{10}myhostnameJSON\u{4}9000UUID$e9b557b8-92e2-45cd-8a95-8730ffd604a5VERS\u{5}8.3.1"
// '$' = 36 in the ASCII table
//
// Each value starts with a tag, followed by the length of the value in one byte, then the value
// itself in the next length bytes.
//
// See the code of LMS for details:
// https://github.com/LMS-Community/slimserver/blob/65aa473e029f2dec35b70c14d637d27867cddd11/Slim/Networking/Discovery.pm#L113-L127

/// Discover the LMS server on the local network
pub async fn discover(reply_timeout: Duration) -> Result<(IpAddr, Reply)> {
    info!("Discovering LMS server on the local network");

    let sock = UdpSocket::bind("0.0.0.0:0").await?;
    sock.set_broadcast(true)?;

    let (ip, buffer) = loop {
        let response = timeout(reply_timeout, broasdcast_and_recv(&sock)).await;
        match response {
            Ok(Ok(ip_and_buffer)) => {
                break ip_and_buffer;
            }
            Ok(Err(e)) => return Err(e.into()),
            Err(_) => warn!("Timeout waiting for LMS reply, retrying..."),
        }
    };

    parse_reply(&buffer)
        .map(|(_, reply)| {
            info!(
                "Discovered LMS '{}' at {}:{}",
                reply.hostname, ip, reply.port
            );

            (ip, reply)
        })
        .map_err(|error| error.to_owned().into())
}

async fn broasdcast_and_recv(sock: &UdpSocket) -> Result<(std::net::IpAddr, [u8; 1024])> {
    let mut buffer = [0; 1024];
    let message = "eNAME\0JSON\0UUID\0VERS\0".as_bytes();
    let _ = sock.send_to(&message, "255.255.255.255:3483").await?;
    let (_, addr) = sock.recv_from(&mut buffer).await?;
    Ok((addr.ip(), buffer))
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

fn parse_reply(input: &[u8]) -> IResult<&[u8], Reply> {
    map(
        (parse_hostname, parse_port, parse_uuid, parse_version),
        |(hostname, port, uuid, version)| Reply {
            hostname,
            port,
            uuid,
            version,
        },
    )
    .parse(input)
}
