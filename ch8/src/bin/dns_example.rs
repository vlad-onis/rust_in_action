use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

use clap::{App, Arg};
use rand;

use trust_dns::op::{Message, MessageType, OpCode, Query};
use trust_dns::rr::domain::Name;
use trust_dns::rr::record_type::RecordType;
use trust_dns::serialize::binary::*;

fn main () {

    let app = App::new("resolve")
        .about("A simple app to use DNS resolver")
        .arg(Arg::new("dns-server").short('s').default_value("1.1.1.1"))
        .arg(Arg::new("domain-name").required(true))
        .get_matches();

    let domain_name_raw = app.value_of("domain-name").unwrap();
    let domain_name = Name::from_ascii(&domain_name_raw).unwrap();

    let dns_server_raw = app.value_of("dns-server").unwrap();
    let dns_server: SocketAddr = format!("{}:53", dns_server_raw)
        .parse()
        .expect("Invalid address");

    let mut request_as_bytes:Vec<u8> = Vec::with_capacity(512); // Length 0 but reserved memory is 512
    let mut response_as_bytes: Vec<u8> = vec![0; 512]; // Response is required by the lib to have a lenght

    let mut msg = Message::new();
    msg
        .set_id(rand::random::<u16>())
        .set_message_type(MessageType::Query)
        .add_query(Query::query(domain_name, RecordType::A))
        .set_op_code(OpCode::Query)
        .set_recursion_desired(true);

    let mut encoder = 
        BinEncoder::new(&mut request_as_bytes);
    msg.emit(&mut encoder).unwrap();

    let localhost = UdpSocket::bind("0.0.0.0:0")
        .expect("Cannot bind to local socket"); // Listen to all addresses on a random port

    let timeout = Duration::from_secs(3);
    localhost.set_read_timeout(Some(timeout)).unwrap();
    localhost.set_nonblocking(false).unwrap();

    let _amt = localhost
        .send_to(&request_as_bytes, dns_server)
        .expect("socket misconfigured");

    let (_amt, remote) = localhost
        .recv_from(&mut response_as_bytes)
        .expect("timeout reached");

    let dns_message = Message::from_vec(&response_as_bytes)
        .expect("unable to parse response");

    for answer in dns_message.answers() {
        if answer.record_type() == RecordType::A {
            let resource = answer.rdata();
            let ip = resource
                .to_ip_addr()
                .expect("Invalid IP address received");
            print!("{}", ip.to_string());
        }
    }
}