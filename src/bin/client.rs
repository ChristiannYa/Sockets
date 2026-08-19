use std::{
    io::{self, BufRead, Write},
    net::UdpSocket,
};

use sockets::{
    FIELD_LENGTHS,
    bits::{BitReader, BitWriter, FieldName, PacketSchema},
};

fn main() {
    let skt = UdpSocket::bind("0.0.0.0:0").expect("Couldn't bind");
    let pkt_schema = PacketSchema::build(FIELD_LENGTHS).unwrap();
    let stdin = io::stdin();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        stdin.lock().read_line(&mut line).unwrap();
        let line = line.trim();

        let mut writer = BitWriter::new(&pkt_schema);
        match line {
            "jump" => writer.write(FieldName::IsJumping, 1),
            "hit" => writer.write(FieldName::IsHit, 1),
            "quit" => break,
            _ => {
                println!("Unknown command");
                continue;
            }
        }

        skt.send_to(&writer.buf(), "127.0.0.1:34254")
            .expect("Send failed");

        let mut buf = [0; 1024];
        let (bytes_len, _) = skt
            .recv_from(&mut buf)
            .expect("Receive failed");
        let buf = &buf[..bytes_len];

        let mut reader = BitReader::new(&pkt_schema, buf);
        for (field_name, _) in pkt_schema.fields.iter() {
            if reader.isset(field_name) {
                println!("Server replied {field_name:?} = {}", reader.read(field_name));
            }
        }
    }
}
