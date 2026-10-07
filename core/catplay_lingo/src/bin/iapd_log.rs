//! Decode iapd's text packet capture, preserving capture order and failures.
use catplay_lingo::lingos::LingoRegistry;
use catplay_lingo::{DecodeContext, Frame, FrameCodec, Packet, ProtocolRegistry, TransactionIdPolicy};
use std::io::{self, BufRead, Write};

#[derive(Debug, PartialEq, Eq)]
struct Header {
    timestamp: String,
    elapsed: String,
    transport_ptr: String,
    transport: String,
    lingo: u8,
    command_id: u16,
    transaction_id: u32,
    payload_length: usize,
}

#[derive(Debug, PartialEq, Eq)]
enum Direction {
    AccessoryToIPod,
    IPodToAccessory,
}

#[derive(Debug)]
struct LogFrame {
    header: Header,
    direction: Direction,
    payload: Vec<u8>,
}

fn hex(field: &str, prefix: &str) -> Result<u32, String> {
    let value = field
        .strip_prefix(prefix)
        .ok_or_else(|| format!("expected {prefix}, got {field:?}"))?;
    u32::from_str_radix(value, 16).map_err(|e| format!("invalid {prefix}: {e}"))
}

fn parse_line(line: &str) -> Result<LogFrame, String> {
    let fields: Vec<_> = line.split(';').map(str::trim).collect();
    let [
        timestamp,
        elapsed,
        transport_ptr,
        transport,
        source,
        lingo,
        command,
        transaction,
        payload,
    ] = fields.as_slice()
    else {
        return Err(format!("expected 9 semicolon-separated fields, got {}", fields.len()));
    };
    let timestamp = timestamp.strip_suffix(" LOG").ok_or("missing LOG marker")?;
    let transport_ptr = transport_ptr
        .strip_prefix("transport_ptr ")
        .ok_or("missing transport_ptr")?;
    let direction = match *source {
        "Acc" => Direction::AccessoryToIPod,
        "iPod" => Direction::IPodToAccessory,
        _ => return Err(format!("unknown packet source {source:?}")),
    };
    let (length, bytes) = payload
        .strip_prefix("payload(")
        .ok_or("missing payload")?
        .split_once(")=<")
        .ok_or("invalid payload delimiter")?;
    let payload_length: usize = length
        .parse()
        .map_err(|e| format!("invalid payload length: {e}"))?;
    let bytes = bytes
        .strip_suffix('>')
        .ok_or("missing payload closing bracket")?;
    let payload: Vec<u8> = bytes
        .split_whitespace()
        .map(|byte| {
            if byte.len() != 2 {
                return Err(format!("invalid payload byte {byte:?}"));
            }
            u8::from_str_radix(byte, 16).map_err(|e| format!("invalid payload byte {byte:?}: {e}"))
        })
        .collect::<Result<_, _>>()?;
    if payload.len() != payload_length {
        return Err(format!(
            "payload length mismatch: declared {payload_length}, actual {}",
            payload.len()
        ));
    }
    Ok(LogFrame {
        header: Header {
            timestamp: timestamp.to_owned(),
            elapsed: (*elapsed).to_owned(),
            transport_ptr: transport_ptr.to_owned(),
            transport: (*transport).to_owned(),
            lingo: u8::try_from(hex(lingo, "lingo=0x")?).map_err(|_| "lingo exceeds u8")?,
            command_id: u16::try_from(hex(command, "cmdID=0x")?).map_err(|_| "command exceeds u16")?,
            transaction_id: hex(transaction, "transID=0x")?,
            payload_length,
        },
        direction,
        payload,
    })
}

fn decode(frame: &LogFrame) -> Result<Packet, String> {
    // The capture supplies the transaction separately: do not guess whether
    // the first two payload bytes are an ID. Required IDs may also be zero.
    // Original wire length/checksum are absent from the capture.
    let metadata = LingoRegistry::metadata(frame.header.lingo, frame.header.command_id).map_err(|e| format!("{e:?}: {e}"))?;
    let transaction_id = match frame.header.transaction_id {
        0 if !matches!(metadata.map(|m| m.transaction_id), Some(TransactionIdPolicy::Required)) => None,
        id => Some(u16::try_from(id).map_err(|_| "transaction ID exceeds iAP1 u16")?),
    };
    let body =
        FrameCodec::command_body(frame.header.lingo, frame.header.command_id, transaction_id, &frame.payload).map_err(|e| e.to_string())?;
    let wire_frame = Frame { body: &body };
    let (command, data) = wire_frame
        .command()
        .ok_or("missing reconstructed command")?;
    let payload = if transaction_id.is_some() { &data[2..] } else { data };
    let message = LingoRegistry::decode(
        wire_frame.lingo().ok_or("missing reconstructed lingo")?,
        command,
        &DecodeContext {
            adjusted_payload_length: payload.len(),
            has_transaction_id: transaction_id.is_some(),
        },
        payload,
    )
    .map_err(|e| format!("{e:?}: {e}"))?;
    Ok(Packet { message, transaction_id })
}

fn run() -> Result<bool, Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = args
        .next()
        .ok_or("usage: iapd-log <capture.log> [output.txt]")?;
    let output = args.next();
    if args.next().is_some() {
        return Err("usage: iapd-log <capture.log> [output.txt]".into());
    }
    let reader = io::BufReader::new(std::fs::File::open(path)?);
    let mut writer: Box<dyn Write> = match output {
        Some(path) => Box::new(io::BufWriter::new(std::fs::File::create(path)?)),
        None => Box::new(io::BufWriter::new(io::stdout().lock())),
    };
    let (mut total, mut decoded, mut errors, mut events) = (0, 0, 0, 0);
    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if line.split(';').nth(3).map(str::trim) == Some("Event") {
            events += 1;
            continue;
        }
        total += 1;
        match parse_line(&line) {
            Ok(frame) => {
                let arrow = match frame.direction {
                    Direction::AccessoryToIPod => "->",
                    Direction::IPodToAccessory => "<-",
                };
                write!(writer, "{arrow} {} 0x{:04x} ", frame.header.elapsed, frame.header.transaction_id)?;
                match decode(&frame) {
                    Ok(packet) => {
                        writeln!(writer, "{:#?}\n", packet.message)?;
                        decoded += 1;
                    }
                    Err(error) => {
                        writeln!(
                            writer,
                            "Decode error (line {}, lingo=0x{:02x}, command=0x{:04x}): {error}\nPayload: {:02x?}\n",
                            index + 1,
                            frame.header.lingo,
                            frame.header.command_id,
                            frame.payload
                        )?;
                        errors += 1;
                    }
                }
            }
            Err(error) => {
                writeln!(writer, "Log parse error (line {}): {error}\nRaw: {line}\n", index + 1)?;
                errors += 1;
            }
        }
    }
    writer.flush()?;
    eprintln!("packets={total}, decoded={decoded}, errors={errors}, events={events}");
    Ok(errors == 0)
}

fn main() {
    match run() {
        Ok(true) => {}
        Ok(false) => std::process::exit(1),
        Err(error) => {
            eprintln!("iapd-log: {error}");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(source: &str, lingo: &str, command: &str, id: &str, payload: &str) -> String {
        format!(
            "2026-10-01 15:52:09.118000 +0200 LOG; 0.000000; transport_ptr 20; USB; {source}; lingo=0x{lingo}; cmdID=0x{command}; transID=0x{id}; {payload}"
        )
    }

    #[test]
    fn decodes_capture_with_and_without_transaction_and_wide_command() {
        for (source, lingo, command, id, payload, expected) in [
            ("Acc", "00", "0038", "00000000", "payload(0)=<>", "StartIDPS"),
            ("iPod", "00", "0002", "00000002", "payload(2)=<00 38 >", "IPodAck"),
            ("iPod", "00", "0002", "00000000", "payload(2)=<00 38 >", "IPodAck"),
            ("Acc", "04", "001c", "0000000f", "payload(0)=<>", "GetPlayStatus"),
        ] {
            let frame = parse_line(&line(source, lingo, command, id, payload)).unwrap();
            let packet = decode(&frame).unwrap();
            assert!(format!("{:?}", packet.message).contains(expected));
            assert_eq!(
                packet.transaction_id,
                if command == "0002" && id == "00000000" {
                    None
                } else {
                    Some(u16::from_str_radix(id, 16).unwrap())
                }
            );
            assert_eq!(
                frame.direction,
                if source == "Acc" {
                    Direction::AccessoryToIPod
                } else {
                    Direction::IPodToAccessory
                }
            );
        }
    }

    #[test]
    fn rejects_corrupt_capture_fields() {
        for (source, lingo, payload) in [
            ("Acc", "00", "payload(2)=<00 >"),
            ("Acc", "00", "payload(1)=<zz >"),
            ("Acc", "100", "payload(0)=<>"),
            ("unknown", "00", "payload(0)=<>"),
        ] {
            assert!(parse_line(&line(source, lingo, "0038", "00000000", payload)).is_err());
        }
        assert!(parse_line("invalid").is_err());
    }

    #[test]
    fn preserves_unknown_commands_and_rejects_truncating_transaction_ids() {
        let frame = parse_line(&line("Acc", "00", "00ff", "00000000", "payload(0)=<>")).unwrap();
        assert!(decode(&frame).unwrap_err().contains("UnknownCommand"));
        let frame = parse_line(&line("Acc", "00", "0038", "00010000", "payload(0)=<>")).unwrap();
        assert!(decode(&frame).unwrap_err().contains("exceeds"));
    }
}
