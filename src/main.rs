use std::{collections::VecDeque, fmt::Debug, time::Duration};

use serialport::{SerialPortType, UsbPortInfo};

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

    let mut buf = Vec::with_capacity(100);
    loop {
        let to_read = serial.bytes_to_read().unwrap();
        if to_read == 0 {
            std::thread::sleep(Duration::from_millis(5));
            continue;
        }

        buf.resize(to_read as usize, 0);
        serial.read_exact(&mut buf);

        let str = String::from_utf8_lossy(&buf);
        print!("{str}");
    }
}
