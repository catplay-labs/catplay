use std::path::Path;

use bytes::BytesMut;
use image::{ColorType, ImageFormat};
use insta::assert_debug_snapshot;
use openh264::{decoder::Decoder, formats::YUVSource};

use super::*;
use crate::X264FrameBuffer;

const CARPLAY_SPS: [((i32, i32, i32), &[u8]); 6] = [
    (
        (800, 480, 30),
        &[
            0, 0, 0, 1, 0x27, 0x64, 0x00, 0x1f, 0xac, 0x13, 0x14, 0x50, 0x32, 0x0f, 0x69, 0xb8, 0x08, 0x68, 0x30, 0x36, 0x82, 0x21, 0x19,
            0x60,
        ],
    ),
    (
        (800, 480, 60),
        &[
            0, 0, 0, 1, 0x27, 0x64, 0x00, 0x1f, 0xac, 0x13, 0x14, 0x50, 0x32, 0x0f, 0x69, 0xb8, 0x08, 0x68, 0x30, 0x36, 0x82, 0x21, 0x19,
            0x60,
        ],
    ),
    (
        (960, 540, 30),
        &[
            0, 0, 0, 1, 0x27, 0x64, 0x00, 0x1f, 0xac, 0x13, 0x14, 0x50, 0x3c, 0x04, 0x5f, 0xb9, 0xb8, 0x08, 0x68, 0x30, 0x36, 0x82, 0x21,
            0x19, 0x60,
        ],
    ),
    (
        (960, 540, 60),
        &[
            0, 0, 0, 1, 0x27, 0x64, 0x00, 0x20, 0xac, 0x13, 0x14, 0x50, 0x3c, 0x04, 0x5f, 0xb9, 0xb8, 0x08, 0x68, 0x30, 0x36, 0x82, 0x21,
            0x19, 0x60,
        ],
    ),
    (
        (1920, 720, 30),
        &[
            0, 0, 0, 1, 0x27, 0x64, 0x00, 0x28, 0xac, 0x13, 0x14, 0x50, 0x1e, 0x01, 0x6e, 0x9b, 0x80, 0x86, 0x83, 0x03, 0x68, 0x22, 0x11,
            0x96,
        ],
    ),
    (
        (1920, 720, 60),
        &[
            0, 0, 0, 1, 0x27, 0x64, 0x00, 0x2a, 0xac, 0x13, 0x14, 0x50, 0x1e, 0x01, 0x6e, 0x9b, 0x80, 0x86, 0x83, 0x03, 0x68, 0x22, 0x11,
            0x96,
        ],
    ),
];
const APPLE_PPS: &[u8] = &[0, 0, 0, 1, 0x28, 0xee, 0x3c, 0xb0];

#[allow(dead_code)]
#[derive(Debug)]
struct NalSemanticsSnapshot {
    configuration: (i32, i32, i32),
    apple_sps: SpsSemantics,
    x264_sps: SpsSemantics,
    apple_pps: PpsSemantics,
    x264_pps: PpsSemantics,
    x264_idr: IdrSemantics,
}

fn annexb_nals(data: &[u8]) -> Vec<(&[u8], usize)> {
    let mut starts = Vec::new();
    let mut offset = 0;
    while offset + 3 <= data.len() {
        if data[offset..].starts_with(&[0, 0, 0, 1]) {
            starts.push((offset, 4));
            offset += 4;
        } else if data[offset..].starts_with(&[0, 0, 1]) {
            starts.push((offset, 3));
            offset += 3;
        } else {
            offset += 1;
        }
    }
    starts
        .iter()
        .enumerate()
        .map(|(index, &(start, prefix_len))| {
            let end = starts
                .get(index + 1)
                .map(|&(next, _)| next)
                .unwrap_or(data.len());
            (&data[start..end], prefix_len)
        })
        .collect()
}

fn find_nal(data: &[u8], nal_type: u8) -> &[u8] {
    annexb_nals(data)
        .into_iter()
        .find(|(nal, prefix_len)| {
            nal.get(*prefix_len)
                .is_some_and(|byte| byte & 0x1f == nal_type)
        })
        .map(|(nal, _)| nal)
        .unwrap_or_else(|| panic!("missing NAL type {nal_type}"))
}

#[test]
fn carplay_parameter_set_and_idr_semantics() {
    let snapshots = CARPLAY_SPS
        .iter()
        .map(|&(configuration @ (width, height, fps), apple_sps)| {
            let mut encoder = X264FrameBuffer::new(width, height, fps).unwrap();
            let mut headers = BytesMut::new();
            encoder.get_headers(&mut headers).unwrap();

            let rgba = vec![0; width as usize * height as usize * 4];
            let mut frame = BytesMut::new();
            encoder.update_rgba(&rgba, &mut frame).unwrap();

            NalSemanticsSnapshot {
                configuration,
                apple_sps: SpsSemantics::parse(apple_sps).unwrap(),
                x264_sps: SpsSemantics::parse(find_nal(&headers, 7)).unwrap(),
                apple_pps: PpsSemantics::parse(APPLE_PPS).unwrap(),
                x264_pps: PpsSemantics::parse(find_nal(&headers, 8)).unwrap(),
                x264_idr: IdrSemantics::parse(find_nal(&frame, 5)).unwrap(),
            }
        })
        .collect::<Vec<_>>();

    assert_debug_snapshot!(snapshots);
}

#[test]
fn x264_carplay_stream_decodes_to_tiled_png() {
    const COLORS: [[u8; 4]; 12] = [
        [230, 25, 75, 255],
        [60, 180, 75, 255],
        [255, 225, 25, 255],
        [0, 130, 200, 255],
        [245, 130, 48, 255],
        [145, 30, 180, 255],
        [70, 240, 240, 255],
        [240, 50, 230, 255],
        [210, 245, 60, 255],
        [250, 190, 212, 255],
        [0, 128, 128, 255],
        [220, 190, 255, 255],
    ];

    for &((width, height, fps), apple_sps) in &CARPLAY_SPS {
        let (width_usize, height_usize) = (width as usize, height as usize);
        let (tile_width, tile_height) = (width_usize / 8, height_usize / 6);
        let mut source_rgba = vec![0; width_usize * height_usize * 4];
        for y in 0..height_usize {
            for x in 0..width_usize {
                let tile_x = x / tile_width;
                let tile_y = y / tile_height;
                let color = if x % tile_width < 4 || y % tile_height < 4 {
                    [20, 20, 20, 255]
                } else {
                    COLORS[(tile_y * 8 + tile_x) % COLORS.len()]
                };
                source_rgba[(y * width_usize + x) * 4..][..4].copy_from_slice(&color);
            }
        }

        let mut encoder = X264FrameBuffer::new(width, height, fps).unwrap();
        let mut x264_headers = BytesMut::new();
        encoder.get_headers(&mut x264_headers).unwrap();
        let mut idr = BytesMut::new();
        encoder.update_rgba(&source_rgba, &mut idr).unwrap();

        let decode = |parameter_sets: &[u8]| {
            let mut bitstream = Vec::with_capacity(parameter_sets.len() + idr.len());
            bitstream.extend_from_slice(parameter_sets);
            bitstream.extend_from_slice(&idr);
            let mut decoder = Decoder::new().unwrap();
            let decoded = decoder
                .decode(&bitstream)
                .unwrap_or_else(|error| panic!("OpenH264 rejected {width}x{height}@{fps}: {error}"))
                .unwrap_or_else(|| panic!("OpenH264 did not output {width}x{height}@{fps}"));
            assert_eq!(decoded.dimensions(), (width_usize, height_usize));
            let mut rgba = vec![0; decoded.rgba8_len()];
            decoded.write_rgba8(&mut rgba);
            rgba
        };

        let decoded_with_x264_parameter_sets = decode(&x264_headers);
        let mut apple_parameter_sets = Vec::with_capacity(apple_sps.len() + APPLE_PPS.len());
        apple_parameter_sets.extend_from_slice(apple_sps);
        apple_parameter_sets.extend_from_slice(APPLE_PPS);
        let decoded_rgba = decode(&apple_parameter_sets);
        assert_eq!(
            decoded_rgba, decoded_with_x264_parameter_sets,
            "decoded pixels differ for {width}x{height}@{fps}"
        );

        let snapshot_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/carplay_helpers/snapshots/png")
            .join(format!("x264_carplay_tiled_decode_{width}x{height}_{fps}fps.png"));
        image::save_buffer_with_format(
            snapshot_path,
            &decoded_rgba,
            width as u32,
            height as u32,
            ColorType::Rgba8,
            ImageFormat::Png,
        )
        .expect("failed to write decoded PNG snapshot");
    }
}
