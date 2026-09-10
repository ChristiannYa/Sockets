use bitp::{
    FIELD_LENGTHS, PacketKind,
    bits::{BitReader, BitWriter, FieldName, PacketSchema},
};
use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
};

type ClientStates = HashMap<u32, HashMap<FieldName, u32>>;

fn main() {
    let skt = UdpSocket::bind("0.0.0.0:34254").expect("Couldn't bind");
    let pkt_schema = PacketSchema::build(FIELD_LENGTHS).unwrap();
    let mut buf = [0; 1024];

    let mut skt_clis = HashMap::<SocketAddr, u32>::new();
    let mut next_sid: u32 = 0;

    let mut skt_seqs = HashMap::<SocketAddr, u8>::new();

    let mut cli_states = ClientStates::new();

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
        let is_new_cli = !skt_clis.contains_key(&skt_src);

        let sid = seed_packet(
            &skt_src,
            &mut skt_clis,
            &mut skt_seqs,
            &mut writer,
            &mut next_sid,
        );

        if is_new_cli {
            skt.send_to(&new_client_seed_buf(&pkt_schema, sid), skt_src)
                .ok();

            if !cli_states.is_empty() {
                skt.send_to(&sync_buf(&pkt_schema, &cli_states, sid), skt_src)
                    .ok();
            }
        }

        handle_writes(
            &pkt_schema,
            &mut cli_states,
            &mut reader,
            &mut writer,
            &skt_src,
            sid,
        );

        let mut buf = vec![PacketKind::Single as u8];
        buf.extend(writer.buf());
        broadcast(&skt, &skt_clis, &skt_src, &buf);
    }
}

/// Seeds packet wth session ID and then sequence number.
/// **Returns** the session id
fn seed_packet<'a>(
    skt_src: &SocketAddr,
    skt_clis: &mut HashMap<SocketAddr, u32>,
    skt_seqs: &mut HashMap<SocketAddr, u8>,
    writer: &mut BitWriter<'a>,
    next_sid: &mut u32,
) -> u32 {
    let sid: u32 = *skt_clis.entry(*skt_src).or_insert_with(|| {
        let id = *next_sid;
        *next_sid += 1;
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
    cli_states: &mut ClientStates,
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

fn save_client_state(field_name: &FieldName, states: &mut ClientStates, sid: u32, value: u32) {
    states
        .entry(sid)
        .or_default()
        .insert(field_name.clone(), value);
}

/// **Returns *seed* data the client needs before receiving main
/// information
fn new_client_seed_buf<'a>(pkt_schema: &PacketSchema<'a>, new_cli_sid: u32) -> Vec<u8> {
    let mut buf = vec![PacketKind::Single as u8];

    let mut writer = BitWriter::new(pkt_schema);
    writer.write(&FieldName::SessionId, &new_cli_sid);
    writer.write(&FieldName::IsNewClient, &1);

    buf.extend(writer.buf());
    buf
}

/// **Returns a byte buffer** containing every other connected client's
/// state to sync a newly-joined client.
/// Layout:
/// ```
/// let record_count = vec![2];
///
/// // 1=record_len, 2=mask_bytes, 3=value_bytes
/// // `record_len` is the byte length of the client's `BitWriter::buf()`'s
/// // output, including `SessionId` as a "seed" value to identify the record
/// let record1 = vec![1, 2, 3];
/// let record2 = vec![1, 2, 3];
///
/// // [2, 1, 2, 3, 1, 2, 3]
/// return [record_count, record1, record2].concat()
/// ```
fn sync_buf<'a>(
    pkt_schema: &PacketSchema<'a>,
    cli_states: &ClientStates,
    new_cli_sid: u32,
) -> Vec<u8> {
    let records: Vec<Vec<u8>> = cli_states
        .iter()
        .filter(|(sid, _)| **sid != new_cli_sid)
        .map(|(sid, cli_state)| {
            let mut writer = BitWriter::new(pkt_schema);
            writer.write(&FieldName::SessionId, sid);

            for (field_name, _) in pkt_schema.fields.iter() {
                if let Some(val) = cli_state.get(field_name) {
                    writer.write(field_name, val);
                }
            }

            writer.buf()
        })
        .collect();

    let mut buf_sync = vec![PacketKind::Batch as u8, records.len() as u8];
    for record in records {
        buf_sync.push(record.len() as u8);
        buf_sync.extend(record);
    }
    buf_sync
}

/// Broadcast to every client besides the current
fn broadcast(
    skt: &UdpSocket,
    skt_clis: &HashMap<SocketAddr, u32>,
    skt_src: &SocketAddr,
    buf: &[u8],
) {
    for skt_cli in skt_clis.keys() {
        if *skt_cli != *skt_src {
            skt.send_to(buf, skt_cli).ok();
        }
    }
}
