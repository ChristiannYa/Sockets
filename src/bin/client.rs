use std::{
    io::{self, BufRead, Write},
    net::UdpSocket,
    sync::mpsc,
    thread,
};

use sockets::{
    FIELD_LENGTHS,
    bits::{BitReader, BitWriter, FieldName, PacketSchema},
};

fn main() {
    let skt = UdpSocket::bind("0.0.0.0:0").expect("Couldn't bind");
    let pkt_schema = PacketSchema::build(FIELD_LENGTHS).unwrap();

    // Dedicated thread that listens for incoming server replies
    let skt_recv = skt.try_clone().expect("Socket clone failed");
    let (tx_skt_recv, rx_skt_recv) = mpsc::channel::<String>();
    let pkt_schema_recv = pkt_schema.clone();
    thread::spawn(move || {
        let mut buf = [0; 1024];
        loop {
            let (skt_src_buflen, _) = skt_recv
                .recv_from(&mut buf)
                .expect("Didn't receive data");

            let buf = &buf[..skt_src_buflen];
            let mut reader = BitReader::new(&pkt_schema_recv, buf);
            for (field_name, _) in pkt_schema_recv.fields.iter() {
                if !reader.isset(field_name) {
                    continue;
                }

                tx_skt_recv
                    .send(format!("@Server: {field_name:?}={}", reader.read(field_name)))
                    .unwrap();
            }
        }
    });

    // Dedicated thread that handles reading stdin
    let (tx_stdin, rx_stdin) = mpsc::channel::<String>();
    thread::spawn(move || {
        let stdin = io::stdin();
        loop {
            prompt();

            let mut line = String::new();
            stdin.lock().read_line(&mut line).unwrap();
            let line = line.trim().to_string();

            tx_stdin.send(line).unwrap();
        }
    });

    // Main thread handles packing and sending datagrams, and providing
    // server information
    let mut writer = BitWriter::new(&pkt_schema);
    loop {
        let input_recv = rx_stdin.try_recv().ok();
        let is_input_recv_none = input_recv.is_none();
        if let Some(input) = input_recv {
            match input.as_str() {
                "is_jumping" => writer.write(&FieldName::IsJumping, &1),
                "is_crouching" => writer.write(&FieldName::IsCrouching, &1),
                "quit" => break,
                _ => {
                    println!("Unkown command");
                    continue;
                }
            }

            skt.send_to(&writer.buf(), "127.0.0.1:34254")
                .expect("Send failed");
        }

        let skt_recv = rx_skt_recv.try_recv().ok();
        let is_skt_recv_none = skt_recv.is_none();
        if let Some(skt_resp) = skt_recv {
            println!("{skt_resp}");
            prompt();
        }

        if is_input_recv_none || is_skt_recv_none {
            thread::park();
        }
    }
}

fn prompt() {
    print!("> ");
    io::stdout().flush().unwrap();
}
