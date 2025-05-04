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

    let mut buf = vec![0; 100];
    let mut len = 0;
    loop {
        let to_read = serial.bytes_to_read().unwrap();
        if to_read == 0 {
            std::thread::sleep(Duration::from_millis(1));
            continue;
        }

        len += serial.read(&mut buf[len..]).unwrap();
        if !(buf[0..len]).contains(&b'\n') {
            continue;
        }

        let str = String::from_utf8_lossy(&buf[0..len]);
        len = 0;

        let splits: Vec<f32> = str
            .trim()
            .split(", ")
            .flat_map(|s| s.parse().ok())
            .collect();

        if splits.len() != 3 {
            continue;
        }
        println!("{splits:?}");

        let x = splits[0];
        let y = splits[1];
        let z = splits[2];

        let bytes = encode_data(x, y, z);
        socket.send(&bytes).unwrap();
    }
}
