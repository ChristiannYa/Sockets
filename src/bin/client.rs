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

    let (tx_ev, rx_ev) = mpsc::channel::<Event>();
    let tx_ev_1 = tx_ev.clone();

    // Dedicated thread that listens for incoming server replies
    let skt_recv = skt.try_clone().expect("Socket clone failed");
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

                let msg = format!("@Server: {field_name:?}={}", reader.read(field_name));
                tx_ev.send(Event::Server(msg)).unwrap();
            }
        }
    });

    // Dedicated thread that handles reading stdin
    thread::spawn(move || {
        let stdin = io::stdin();
        loop {
            prompt();

            let mut line = String::new();
            stdin.lock().read_line(&mut line).unwrap();
            let line = line.trim().to_string();

            tx_ev_1.send(Event::Input(line)).unwrap();
        }
    });

    // Main thread handles packing and sending datagrams, and providing
    // server information
    for ev in rx_ev {
        match ev {
            Event::Server(msg) => {
                println!("{msg}");
                prompt();
            }

            Event::Input(inp) => {
                let mut writer = BitWriter::new(&pkt_schema);

                match inp.as_str() {
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
        }
    }
}

enum Event {
    Input(String),
    Server(String),
}

fn prompt() {
    print!("> ");
    io::stdout().flush().unwrap();
}
