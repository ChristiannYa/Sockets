use bitp::{
    FIELD_LENGTHS,
    bits::{BitReader, BitWriter, FieldName, PacketSchema},
};
use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
};

type CliStates = HashMap<u32, HashMap<FieldName, u32>>;

fn main() {
    let skt = UdpSocket::bind("0.0.0.0:34254").expect("Couldn't bind");
    let pkt_schema = PacketSchema::build(FIELD_LENGTHS).unwrap();
    let mut buf = [0; 1024];

    let mut skt_clients = HashMap::<SocketAddr, u32>::new();
    let mut next_sess_id: u32 = 0;

    let mut skt_seqs = HashMap::<SocketAddr, u8>::new();

    let mut cli_states = CliStates::new();

    loop {
        let (skt_src_buflen, skt_src) = skt.recv_from(&mut buf).expect("Didn't receive data");

        // Bail early on packets too short to even hold the mask bytes the
        // schema expects.
        // Protects `BitReader::new()`'s `split_at()` from panicking and
        // taking the whole server down over one client's bad packet.
        if skt_src_buflen < pkt_schema.fields.len().div_ceil(8) {
            continue;
        }

        let buf = &buf[..skt_src_buflen];
        let mut reader = BitReader::new(&pkt_schema, buf);
        let mut writer = BitWriter::new(&pkt_schema);

        // Capture client newness before registering new potential client
        let is_new_cli = !skt_clients.contains_key(&skt_src);

        let sid = seed_packet(
            &skt_src,
            &mut skt_clients,
            &mut skt_seqs,
            &mut writer,
            &mut next_sess_id,
        );

        if is_new_cli {
            sync_client(&pkt_schema, &mut cli_states, sid, &skt, &skt_src);
        }

        handle_writes(
            &pkt_schema,
            &mut cli_states,
            &mut reader,
            &mut writer,
            &skt_src,
            sid,
        );

        broadcast(&skt, &skt_clients, &skt_src, &writer.buf());
    }
}

/// Seeds packet wth session ID and then sequence number.
/// **Returns** the session id
fn seed_packet<'a>(
    skt_src: &SocketAddr,
    skt_clients: &mut HashMap<SocketAddr, u32>,
    skt_seqs: &mut HashMap<SocketAddr, u8>,
    writer: &mut BitWriter<'a>,
    next_sess_id: &mut u32,
) -> u32 {
    let sid: u32 = *skt_clients.entry(*skt_src).or_insert_with(|| {
        let id = *next_sess_id;
        *next_sess_id += 1;
        id
    });
    writer.write(&FieldName::SessionId, &sid);

    let seq: &mut u8 = skt_seqs.entry(*skt_src).or_insert(0);
    *seq = seq.wrapping_add(1);
    writer.write(&FieldName::Sequence, &(*seq as u32));

    sid
}

fn handle_writes<'a>(
    pkt_schema: &PacketSchema<'a>,
    cli_states: &mut CliStates,
    reader: &mut BitReader<'a>,
    writer: &mut BitWriter<'a>,
    skt_src: &SocketAddr,
    sid: u32,
) {
    for (field_name, _) in pkt_schema.fields.iter() {
        if !reader.isset(field_name) {
            continue;
        }

        let value = reader.read(field_name);
        writer.write(field_name, &value);

        save_client_state(field_name, cli_states, sid, value);

        println!("@{skt_src} #{sid}: {field_name:?}={value}");
    }
}

/// Broadcast to every client besides the current
fn broadcast(
    skt: &UdpSocket,
    skt_clients: &HashMap<SocketAddr, u32>,
    skt_src: &SocketAddr,
    buf: &[u8],
) {
    for skt_client in skt_clients.keys() {
        if *skt_client != *skt_src {
            skt.send_to(buf, skt_client).ok();
        }
    }
}

fn save_client_state(field_name: &FieldName, states: &mut CliStates, sid: u32, value: u32) {
    states
        .entry(sid)
        .or_default()
        .insert(field_name.clone(), value);
}

fn sync_client<'a>(
    pkt_schema: &PacketSchema<'a>,
    cli_states: &mut CliStates,
    cur_sid: u32,
    skt: &UdpSocket,
    skt_src: &SocketAddr,
) {
    for (cli_sid, states) in cli_states.iter() {
        if *cli_sid == cur_sid {
            continue;
        }

        let mut writer = BitWriter::new(pkt_schema);
        writer.write(&FieldName::SessionId, cli_sid);

        for (field_name, _) in pkt_schema.fields.iter() {
            if let Some(value) = states.get(field_name) {
                writer.write(field_name, value);
            }
        }

        skt.send_to(&writer.buf(), skt_src).ok();
    }
}
