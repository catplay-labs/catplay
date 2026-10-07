use super::{Frame, FrameCodec, LinkError, Writer};

fn encode(body: &[u8]) -> Result<Vec<u8>, LinkError> {
    let mut packet = Vec::new();
    FrameCodec::encode(body, &mut Writer::new(&mut packet))?;
    Ok(packet)
}

#[test]
fn request_identify_frame() {
    assert_eq!(encode(&[0, 0]).unwrap(), [0x55, 2, 0, 0]);
}

#[test]
fn frame_codec_leaves_checksum_validation_to_caller() {
    let packet = encode(&[0, 0]).unwrap();
    let (frame, consumed) = FrameCodec::decode(&packet).unwrap().unwrap();
    assert_eq!(frame.body, &[0, 0]);
    assert_eq!(consumed, packet.len());
}

#[test]
fn general_command_id_cannot_be_truncated() {
    assert_eq!(
        FrameCodec::command_body(0, 0x0100, None, &[]),
        Err(LinkError::CommandOutOfRange(0x0100))
    );
    assert_eq!(
        FrameCodec::command_body(2, 0x0100, None, &[]),
        Err(LinkError::CommandOutOfRange(0x0100))
    );
}

#[test]
fn only_extended_interface_uses_two_byte_commands() {
    let simple = FrameCodec::command_body(2, 0x12, Some(0x1234), &[0x56]).unwrap();
    assert_eq!(simple, [2, 0x12, 0x12, 0x34, 0x56]);
    assert_eq!(Frame { body: &simple }.command(), Some((0x12, &[0x12, 0x34, 0x56][..])));

    let extended = FrameCodec::command_body(4, 0x1234, Some(0x5678), &[0x9a]).unwrap();
    assert_eq!(extended, [4, 0x12, 0x34, 0x56, 0x78, 0x9a]);
    assert_eq!(Frame { body: &extended }.command(), Some((0x1234, &[0x56, 0x78, 0x9a][..])));
}

#[test]
fn extended_length_boundaries() {
    let mut body = vec![4, 0, 1];
    body.resize(255, 0);
    assert_eq!(encode(&body).unwrap()[1], 255);
    body.push(0);
    assert_eq!(&encode(&body).unwrap()[1..4], &[0, 1, 0]);

    body.resize(u16::MAX as usize, 0);
    assert_eq!(&encode(&body).unwrap()[1..4], &[0, 0xff, 0xff]);
    body.push(0);
    assert_eq!(encode(&body), Err(LinkError::BodyTooLong(65_536)));
}

#[test]
fn frame_encoder_appends_to_existing_writer() {
    let mut output = vec![0xaa];
    FrameCodec::encode(&[0, 0], &mut Writer::new(&mut output)).unwrap();
    assert_eq!(output, [0xaa, 0x55, 2, 0, 0]);
}
