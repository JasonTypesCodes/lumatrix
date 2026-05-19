use serialport::{SerialPort, SerialPortType};
use std::io::{self, Write};

const FWK_MAGIC: [u8; 2] = [0x32, 0xAC];
const FWK_VID: u16 = 0x32AC;
const CMD_STAGE_COL: u8 = 0x07;
const CMD_FLUSH_COLS: u8 = 0x08;

const COLS: usize = 9;
const ROWS: usize = 34;

/// Send a full 9×34 grayscale frame to the LED matrix.
///
/// `pixels` must be 306 bytes in row-major order (row 0 all cols, row 1 all cols, …).
/// Internally this stages each column with StageCol (0x07) then commits with FlushCols (0x08).
pub fn send_frame(port: &mut Box<dyn SerialPort>, pixels: &[u8]) -> io::Result<()> {
    assert_eq!(pixels.len(), COLS * ROWS, "frame must be exactly 306 bytes");

    let mut packet = [0u8; 2 + 1 + 1 + ROWS]; // magic + cmd + col_idx + 34 row values
    packet[0] = FWK_MAGIC[0];
    packet[1] = FWK_MAGIC[1];
    packet[2] = CMD_STAGE_COL;

    for col in 0..COLS {
        packet[3] = col as u8;
        for row in 0..ROWS {
            packet[4 + row] = pixels[row * COLS + col];
        }
        port.write_all(&packet)?;
    }

    // Commit all staged columns; the firmware updates the display on this command.
    // No explicit port.flush() — tcdrain() blocks until hardware drains and times
    // out when frames are sent faster than 115200 baud can drain them.
    port.write_all(&[FWK_MAGIC[0], FWK_MAGIC[1], CMD_FLUSH_COLS])?;
    Ok(())
}

/// Clear the display by sending a blank frame.
pub fn clear_display(port: &mut Box<dyn SerialPort>) -> io::Result<()> {
    send_frame(port, &[0u8; COLS * ROWS])
}

pub struct DeviceInfo {
    pub port: String,
    pub description: String,
}

/// Return all serial ports that belong to Framework Computer (VID 0x32AC).
/// Falls back to all available serial ports if none match.
pub fn list_devices() -> Vec<DeviceInfo> {
    let Ok(ports) = serialport::available_ports() else {
        return Vec::new();
    };

    let framework: Vec<DeviceInfo> = ports
        .iter()
        .filter_map(|p| {
            if let SerialPortType::UsbPort(usb) = &p.port_type {
                if usb.vid == FWK_VID {
                    let desc = match (&usb.manufacturer, &usb.product) {
                        (Some(m), Some(p)) => format!("{} {} ({:04X}:{:04X})", m, p, usb.vid, usb.pid),
                        (_, Some(p)) => format!("{} ({:04X}:{:04X})", p, usb.vid, usb.pid),
                        _ => format!("{:04X}:{:04X}", usb.vid, usb.pid),
                    };
                    return Some(DeviceInfo { port: p.port_name.clone(), description: desc });
                }
            }
            None
        })
        .collect();

    if !framework.is_empty() {
        return framework;
    }

    // Fall back to all serial ports
    ports
        .into_iter()
        .map(|p| {
            let desc = match &p.port_type {
                SerialPortType::UsbPort(usb) => {
                    match (&usb.manufacturer, &usb.product) {
                        (Some(m), Some(pr)) => format!("{} {} ({:04X}:{:04X})", m, pr, usb.vid, usb.pid),
                        (_, Some(pr)) => format!("{} ({:04X}:{:04X})", pr, usb.vid, usb.pid),
                        _ => format!("USB {:04X}:{:04X}", usb.vid, usb.pid),
                    }
                }
                SerialPortType::BluetoothPort => "Bluetooth".to_owned(),
                SerialPortType::PciPort => "PCI".to_owned(),
                SerialPortType::Unknown => "Unknown".to_owned(),
            };
            DeviceInfo { port: p.port_name, description: desc }
        })
        .collect()
}

/// Return the port name of the first Framework device found, or None.
pub fn find_device() -> Option<String> {
    list_devices().into_iter().next().map(|d| d.port)
}
