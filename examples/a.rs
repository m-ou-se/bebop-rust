use bebop::{bebop, Bebop};

bebop!("examples/a.bop");

fn main() {
    let data = MediaMessage {
        codec: Some(VideoCodec::H264),
        data: Some(VideoData {
            time: 1.0,
            width: 100,
            height: 300,
            fragment: vec![1, 2, 3],
        }),
    };

    let bytes = data.encode();
    assert_eq!(
        bytes,
        b"\x1e\0\0\0\x01\0\0\0\0\x02\0\0\0\0\0\0\xf0\x3f\
        \x64\0\0\0\x2c\x01\0\0\x03\0\0\0\x01\x02\x03\0"
    );

    let data2 = MediaMessage::decode(&bytes).unwrap();
    assert_eq!(data, data2);
}
