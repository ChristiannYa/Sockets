use tap::Pipe;

use crate::{
    bits::{
        FieldName, PacketSchema,
        shared::{BufferProgress, field_meta_or_panic},
    },
    util::mask,
};

pub struct BitWriter<'a> {
    packet_schema: &'a PacketSchema<'a>,
    buf: Vec<u8>,
    buf_mask: Vec<u8>,
    bits_acc: usize,
}

impl<'a> BitWriter<'a> {
    pub fn new(packet_schema: &'a PacketSchema) -> Self {
        let buf_mask: Vec<u8> = packet_schema
            .fields
            .len()
            .div_ceil(8)
            .pipe(|len| vec![0u8; len]);

        BitWriter {
            packet_schema,
            buf: Vec::new(),
            bits_acc: 0,
            buf_mask,
        }
    }

    pub fn write(&mut self, field_name: &FieldName, field_val: &u32) {
        let (field_ind, field_len) = field_meta_or_panic(self.packet_schema.fields, field_name);

        // Handle buffer mask
        let mask_ind = field_ind / 8;
        let mask_ofs = field_ind % 8;
        self.buf_mask[mask_ind] |= 1 << mask_ofs;

        let prog = BufferProgress::calc(self.bits_acc, &field_len);

        // New byte allocation needed for the write
        if prog.ind >= self.buf.len() {
            self.buf.push(0);
        }

        // Overflow
        if field_len > prog.rem_len {
            self.write_ovf(*field_val, field_len, &prog);
        } else {
            self.buf[prog.ind] |= (*field_val as u8) << prog.ofs;
            self.bits_acc += field_len;
        }
    }

    fn write_ovf(&mut self, field_val: u32, field_len: usize, prog: &BufferProgress) {
        let field_rem = (field_val & mask(&prog.rem_len)) as u8;
        let field_ovf = (field_val >> prog.rem_len) & mask(&prog.ovf_len);

        self.buf[prog.ind] |= field_rem << prog.ofs;

        // Super overflow
        if prog.ovf_len > 8 {
            self.bits_acc += prog.rem_len;
            self.buf.push(0);

            let prog_ovf_len = prog.ovf_len;
            let prog = BufferProgress::calc(self.bits_acc, &prog_ovf_len);
            self.write_ovf(field_ovf, prog_ovf_len, &prog);
        } else {
            self.bits_acc += field_len;
            self.buf.push(field_ovf as u8);
        }
    }

    pub fn buf(&self) -> Vec<u8> {
        [self.buf_mask.as_slice(), self.buf.as_slice()].concat()
    }
}

#[cfg(test)]
mod tests {
    use tap::{Pipe, Tap};

    use crate::{
        bits::{BitReader, BitWriter, FieldName, PacketSchema, shared::field_meta_or_panic},
        util::split_into_bytes,
    };

    fn t_write_fields(writer: &mut BitWriter, fields: &[(FieldName, u32)]) -> String {
        let mut bits_acc_str: String = String::from("");

        for (field_name, field_val) in fields.iter() {
            let (_, field_len) = field_meta_or_panic(writer.packet_schema.fields, field_name);

            let field_as_bits_str = format!("{field_val:0f_meta$b}", f_meta = field_len);

            // log::info!(
            //     "field as bits: {field_as_bits_str}, bits acc (cur): {}",
            //     split_into_bytes(&bits_acc_str)
            // );

            bits_acc_str.insert_str(0, &field_as_bits_str);
            writer.write(field_name, field_val);

            // log::info!("bits acc (new): {}\n", split_into_bytes(&bits_acc_str));
        }

        split_into_bytes(&bits_acc_str)
    }

    #[test]
    fn write_v1() -> Result<(), String> {
        let schema = PacketSchema::build(&[
            (FieldName::Health, 3),
            (FieldName::PlayerCount, 4),
            (FieldName::IsJumping, 1),
        ])?;

        let mut writer = BitWriter::new(&schema);

        let field_writes_resl = {
            let values = &[
                (FieldName::Health, 5),
                (FieldName::PlayerCount, 9),
                (FieldName::IsJumping, 1),
            ];
            t_write_fields(&mut writer, values)
        };

        let expect = split_into_bytes(&format!("{:08b}", writer.buf[0]));

        if *field_writes_resl == expect {
            Ok(())
        } else {
            Err(format!("'{field_writes_resl}' does not match '{expect}'"))
        }
    }

    #[test]
    fn write_v2() -> Result<(), String> {
        let schema = PacketSchema::build(&[
            (FieldName::Health, 3),
            (FieldName::PlayerCount, 4),
            (FieldName::IsJumping, 1),
        ])?;

        let mut writer = BitWriter::new(&schema);

        let byte_write_resl = {
            let values = &[
                (FieldName::Health, 5),
                (FieldName::PlayerCount, 9),
                (FieldName::IsJumping, 1),
            ];
            t_write_fields(&mut writer, values)
        };

        let expect = split_into_bytes(&format!("{:08b}", writer.buf[0]));

        if *byte_write_resl == expect {
            Ok(())
        } else {
            Err(format!("'{byte_write_resl}' does not match '{expect}'"))
        }
    }

    #[test]
    fn write_buffer() -> Result<(), String> {
        let schema = PacketSchema::build(&[
            (FieldName::IsJumping, 1),
            (FieldName::Health, 3),
            (FieldName::PlayerCount, 4),
            (FieldName::Level, 4),
            (FieldName::Mana, 4),
            (FieldName::RocketsCount, 3),
            (FieldName::IsCrouching, 1),
            (FieldName::IsFriendly, 1),
            (FieldName::KeysCount, 3),
        ])?;

        let mut writer = BitWriter::new(&schema);

        let field_writes_resl = {
            let values = &[
                (FieldName::IsJumping, 1),
                (FieldName::Health, 5),
                (FieldName::PlayerCount, 9),
                (FieldName::Level, 2),
                (FieldName::Mana, 11),
                (FieldName::RocketsCount, 2),
                (FieldName::IsCrouching, 0),
                (FieldName::IsFriendly, 0),
                (FieldName::KeysCount, 4),
            ];
            t_write_fields(&mut writer, values)
        };

        let byte_1 = format!("{:08b}", writer.buf[0]);
        let byte_2 = format!("{:08b}", writer.buf[1]);
        let byte_3 = format!("{:08b}", writer.buf[2]);

        let expect = split_into_bytes(&format!("{byte_3}{byte_2}{byte_1}"));

        if field_writes_resl == expect {
            Ok(())
        } else {
            println!("Test:   {:?}", field_writes_resl);
            println!("Buffer: {:?}", expect);
            Err(String::from("Buffers do not match"))
        }
    }

    #[test]
    fn write_overflow() -> Result<(), String> {
        let schema = PacketSchema::build(&[
            (FieldName::Health, 3),
            (FieldName::RocketsCount, 3),
            (FieldName::PlayerCount, 4),
            (FieldName::KeysCount, 3),
            (FieldName::Time24, 5),
            (FieldName::IsFriendly, 1),
        ])?;

        let mut writer = BitWriter::new(&schema);

        let field_writes_resl = {
            let values = &[
                (FieldName::Health, 2),
                (FieldName::RocketsCount, 4),
                (FieldName::PlayerCount, 12),
                (FieldName::KeysCount, 1),
                (FieldName::Time24, 16),
                (FieldName::IsFriendly, 0),
            ];
            t_write_fields(&mut writer, values)
        };

        let expect = {
            let byte_1 = format!("{:08b}", writer.buf[0]);
            let byte_2 = format!("{:08b}", writer.buf[1]);
            let byte_3 = format!("{:03b}", writer.buf[2]);
            split_into_bytes(&format!("{byte_3}{byte_2}{byte_1}"))
        };

        if field_writes_resl == expect {
            Ok(())
        } else {
            println!("Test  : {:?}", field_writes_resl);
            println!("Buffer: {:?}", expect);
            Err(String::from("Buffers do not match"))
        }
    }

    #[test]
    fn write_overflow_super() -> Result<(), String> {
        let schema = PacketSchema::build(&[
            (FieldName::Health, 3),
            (FieldName::RocketsCount, 3),
            (FieldName::PlayerCount, 4),
            (FieldName::KeysCount, 3),
            (FieldName::Time24, 5),
            (FieldName::IsFriendly, 1),
            (FieldName::Exp, 24),
        ])?;

        let mut writer = BitWriter::new(&schema);

        let field_writes_resl = {
            let values = &[
                (FieldName::Health, 2),
                (FieldName::RocketsCount, 4),
                (FieldName::PlayerCount, 12),
                (FieldName::KeysCount, 1),
                (FieldName::Time24, 16),
                (FieldName::IsFriendly, 1),
                (FieldName::Exp, 9842952),
            ];
            t_write_fields(&mut writer, values)
        };

        let expect = {
            let byte_1 = format!("{:08b}", writer.buf[0]);
            let byte_2 = format!("{:08b}", writer.buf[1]);
            let byte_3 = format!("{:08b}", writer.buf[2]);
            let byte_4 = format!("{:08b}", writer.buf[3]);
            let byte_5 = format!("{:08b}", writer.buf[4]);
            let byte_6 = format!("{:03b}", writer.buf[5]);
            split_into_bytes(&format!("{byte_6}{byte_5}{byte_4}{byte_3}{byte_2}{byte_1}"))
        };

        if field_writes_resl == expect {
            Ok(())
        } else {
            println!("Test  : {:?}", field_writes_resl);
            println!("Buffer: {:?}", expect);
            Err(String::from("Buffers do not match"))
        }
    }

    #[test]
    fn write_buffer_mask() -> Result<(), String> {
        let schema = PacketSchema::build(&[
            (FieldName::Health, 3),
            (FieldName::RocketsCount, 3),
            (FieldName::PlayerCount, 4),
            (FieldName::KeysCount, 3),
            (FieldName::Time24, 5),
            (FieldName::IsFriendly, 1),
            (FieldName::Exp, 24),
            (FieldName::IsJumping, 1),
            (FieldName::Level, 4),
        ])
        .unwrap();

        let mut writer = BitWriter::new(&schema);

        [
            (FieldName::RocketsCount, 6),
            (FieldName::KeysCount, 4),
            (FieldName::IsFriendly, 1),
            (FieldName::Exp, 120000),
            (FieldName::Level, 1),
        ]
        .tap(|values| {
            t_write_fields(&mut writer, values);
        });

        let bytes_written = writer.buf();

        let reader = BitReader::new(&schema, &bytes_written);

        let writer_mask = {
            let mut acc = String::from("");

            for (ind, mask) in writer.buf_mask.iter().enumerate() {
                // println!("(T) #{ind}: {mask:08b}");
                let push = if ind + 1 == writer.buf_mask.len() {
                    (schema.fields.len() % 8)
                        .pipe(|val| if val != 0 { val } else { 8 })
                        .pipe(|len| format!("{mask:0len$b}"))
                } else {
                    format!("{mask:08b}")
                }
                .pipe(|val| val.chars().rev().collect::<String>());

                acc.push_str(&push);
            }

            acc
        };

        println!();

        let field_schema_mask = schema
            .fields
            .iter()
            .map(|(f_name, _)| -> String {
                let is_set = reader.isset(f_name);
                // println!("(T) {:?} set: {is_set}", f_name);
                if is_set {
                    "1".to_string()
                } else {
                    "0".to_string()
                }
            })
            .collect::<Vec<String>>()
            .join("");

        if writer_mask == field_schema_mask {
            Ok(())
        } else {
            println!("Writer mask: {writer_mask}");
            println!("Schema mask: {field_schema_mask}");
            Err("Mask not expected".to_string())
        }
    }
}
