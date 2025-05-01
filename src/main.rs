use std::{net::UdpSocket, time::Duration};

use rosc::{OscMessage, OscPacket, OscType};
use serialport::SerialPortType;

fn encode_data(x: f32, y: f32, z: f32) -> Vec<u8> {
    let msg = OscMessage {
        addr: String::from("/flappies"),
        args: vec![OscType::Float(x), OscType::Float(y), OscType::Float(z)],
    };
    let packet = OscPacket::Message(msg);

    rosc::encoder::encode(&packet).unwrap()
}

fn main() {
    let ports = serialport::available_ports().unwrap();

    let Some(p) = ports.iter().find(|p| {
        if let SerialPortType::UsbPort(pi) = &p.port_type {
            if pi
                .product
                .as_ref()
                .is_some_and(|prod| prod.contains("micro:bit"))
            {
                return true;
            }
        }
        false
    }) else {
        println!("No valid serial ports found");
        return;
    };

    println!("Connecting to {}", p.port_name);
    let mut serial = serialport::new(&p.port_name, 115200).open().unwrap();

    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    socket.set_broadcast(true).unwrap();
    socket.connect("255.255.255.255:8081").unwrap();

    let mut buf = Vec::with_capacity(100);
    loop {
        let to_read = serial.bytes_to_read().unwrap();
        if to_read == 0 {
            std::thread::sleep(Duration::from_millis(5));
            continue;
        }

        buf.resize(to_read as usize, 0);
        serial.read_exact(&mut buf).unwrap();

        let str = String::from_utf8_lossy(&buf);
        print!("{str}");

        let splits: Vec<f32> = str
            .trim()
            .split(", ")
            .flat_map(|s| s.parse().ok())
            .collect();

        if splits.len() != 3 {
            continue;
        }

        let x = splits[0];
        let y = splits[1];
        let z = splits[2];

        let bytes = encode_data(x, y, z);
        socket.send(&bytes).unwrap();
    }

    // loop {
    //     let bytes = encode_data(1.5, 2.5, 3.5);
    //     socket.send(&bytes).unwrap();
    //     println!("Data sent");

    //     std::thread::sleep(Duration::from_secs(1));
    // }
}
