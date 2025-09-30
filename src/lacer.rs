//! Handler for lacing and delacing operations on frame data.

use crate::Error;

/// Handler for lacing and delacing operations on frame data.
pub enum Lacer {
    /// Xiph lacing (variable-size frames with size prefixes)
    Xiph,
    /// Fixed-size lacing (all frames have the same size)
    FixedSize,
    /// EBML lacing (variable-size frames with EBML-encoded sizes)
    Ebml,
}

impl Lacer {
    /// Encode multiple frames into a single laced block
    pub fn lace(&self, frames: &[&[u8]]) -> Vec<u8> {
        match self {
            Lacer::Xiph => {
                if frames.is_empty() {
                    return vec![];
                }
                let num_frames = frames.len();
                let mut output = vec![];
                output.push((num_frames - 1) as u8); // Number of frames - 1
                for frame in &frames[..num_frames - 1] {
                    let mut size = frame.len();
                    while size >= 0xFF {
                        output.push(0xFF);
                        size -= 0xFF;
                    }
                    output.push(size as u8);
                }
                for frame in frames {
                    output.extend_from_slice(frame);
                }
                output
            }
            Lacer::FixedSize => todo!("implement fixed-size lacing"),
            Lacer::Ebml => todo!("implement EBML lacing"),
        }
    }

    /// Decode a laced block into individual frames
    pub fn delace<'a>(&self, data: &'a [u8]) -> crate::Result<Vec<&'a [u8]>> {
        // TODO(perf): avoid heap allocations ideally
        // we should be able to return a `impl Iterator<Item = crate::Result<&'a [u8]>>` here
        // can make it work using nightly features like `generators`.
        // but not sure how to do that with the current stable Rust.

        match self {
            Lacer::Xiph => {
                if data.is_empty() {
                    return Ok(vec![]);
                }

                let num_frames = data[0] as usize + 1;
                if num_frames == 1 {
                    return Ok(vec![&data[1..]]);
                }
                let mut out = Vec::with_capacity(num_frames);

                let data_start_pos = data
                    .iter()
                    .enumerate()
                    .skip(1)
                    .filter(|(_, b)| **b != 0xFF)
                    .nth(num_frames - 2)
                    .map(|(i, _)| i)
                    .ok_or(Error::MalformedLacingData)?
                    + 1;

                let laced_data = data
                    .get(data_start_pos..)
                    .ok_or(Error::MalformedLacingData)?;

                let mut start = 0;
                for size in data[1..data_start_pos]
                    .split_inclusive(|b| *b != 0xFF)
                    .map(|chunk| chunk.iter().map(|b| *b as usize).sum::<usize>())
                {
                    out.push(
                        laced_data
                            .get(start..start + size)
                            .ok_or(Error::MalformedLacingData)?,
                    );
                    start += size;
                }
                out.push(laced_data.get(start..).ok_or(Error::MalformedLacingData)?);
                Ok(out)
            }
            Lacer::FixedSize => todo!("implement fixed-size delacing"),
            Lacer::Ebml => todo!("implement EBML delacing"),
        }
    }
}

// The Xiph lacing uses the same coding of size as found in the Ogg container [@?RFC3533]. The bits 5-6 of the Block Header flags are set to 01.
// The Block data with laced frames is stored as follows:
//     Lacing Head on 1 Octet: Number of frames in the lace minus 1.
//     Lacing size of each frame except the last one.
//     Binary data of each frame consecutively.
// The lacing size is split into 255 values, stored as unsigned octets – for example, 500 is coded 255;245 or [0xFF 0xF5]. A frame with a size multiple of 255 is coded with a 0 at the end of the size – for example, 765 is coded 255;255;255;0 or [0xFF 0xFF 0xFF 0x00].
// The size of the last frame is deduced from the size remaining in the Block after the other frames.
#[cfg(test)]
mod lacer_tests {
    use super::*;
    #[test]
    fn test_xiph_lacing() {
        // 0 frames
        let laced = Lacer::Xiph.lace(&[]);
        assert_eq!(laced, vec![]);
        let frames: Vec<_> = Lacer::Xiph.delace(&[]).unwrap();
        assert_eq!(frames.len(), 0);

        // 4 frames, sizes: 255, 256, 1, remaining
        let len = vec![0x03, 0xFF, 0x00, 0xFF, 0x1, 0x1];
        let frame0 = vec![2u8; 255];
        let frame1 = vec![42u8; 256];
        let frame2 = vec![38u8; 1];
        let frame3 = vec![100u8; 1];

        let laced = Lacer::Xiph.lace(&[&frame0, &frame1, &frame2, &frame3]);
        let data = [len, frame0, frame1, frame2, frame3].concat();
        assert_eq!(laced, data);

        let frames: Vec<_> = Lacer::Xiph.delace(&data).unwrap();
        assert_eq!(frames.len(), 4);
        assert_eq!(frames[0], &[2u8; 255]);
        assert_eq!(frames[1], &[42u8; 256]);
        assert_eq!(frames[2], &[38u8; 1]);
        assert_eq!(frames[3], &[100u8; 1]);

        // 1 frame, size: remaining
        let len = vec![0x00];
        let frame0 = vec![2u8; 255];

        let laced = Lacer::Xiph.lace(&[&frame0]);
        let data = [len, frame0].concat();
        assert_eq!(laced, data);

        let frames: Vec<_> = Lacer::Xiph.delace(&data).unwrap();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0], &[2u8; 255]);

        // 2 frames, sizes: 32, remaining
        let len = vec![0x01, 0x20];
        let frame0 = vec![2u8; 32];
        let frame1 = vec![42u8; 256];

        let laced = Lacer::Xiph.lace(&[&frame0, &frame1]);
        let data = [len, frame0, frame1].concat();
        assert_eq!(laced, data);

        let frames: Vec<_> = Lacer::Xiph.delace(&data).unwrap();
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0], &[2u8; 32]);
        assert_eq!(frames[1], &[42u8; 256]);

        // 4 frames, sizes: 600, 3, 520, remaining
        let len = vec![0x03, 0xFF, 0xFF, 0x5A, 0x3, 0xFF, 0xFF, 0xA];
        assert_eq!(0xff + 0xff + 0x5A, 600);
        assert_eq!(0xff + 0xff + 0xA, 520);
        let frame0 = vec![2u8; 600];
        let frame1 = vec![42u8; 3];
        let frame2 = vec![38u8; 520];
        let frame3 = vec![100u8; 1];

        let laced = Lacer::Xiph.lace(&[&frame0, &frame1, &frame2, &frame3]);
        let data = [len, frame0, frame1, frame2, frame3].concat();
        assert_eq!(laced, data);

        let frames: Vec<_> = Lacer::Xiph.delace(&data).unwrap();
        assert_eq!(frames.len(), 4);
        assert_eq!(frames[0], &[2u8; 600]);
        assert_eq!(frames[1], &[42u8; 3]);
        assert_eq!(frames[2], &[38u8; 520]);
        assert_eq!(frames[3], &[100u8; 1]);
    }
}
