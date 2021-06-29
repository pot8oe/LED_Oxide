/*
   led_oxide is an http API interface to the LedStripController Firmware.

   Copyright (C) 2021  Thomas G. Kenny Jr

   This program is free software: you can redistribute it and/or modify
   it under the terms of the GNU General Public License as published by
   the Free Software Foundation, either version 3 of the License, or
   (at your option) any later version.

   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU General Public License for more details.

   You should have received a copy of the GNU General Public License
   along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use crate::led_strip_controller::protocol::*;
use serialport::*;
use std::{thread, time};

/// Default baud rate. 115200 - 8 1 none
const LEDSC_BAUD: u32 = 115200;

/// Time in milliseconds a response will be waited after sending a command.
const RECEIVE_TIMEOUT_MS: u64 = 500;

/// No devices found user message
const ERROR_NO_DEVICES_FOUND: &str = "No Devices Found";

/// No available ports found user message
const ERROR_NO_AVAILABLE_PORTS: &str = "No Available Ports";

/// Failed to open port user message
const ERROR_FAILED_TO_OPEN_PORT: &str = "Failed to Open Port";

/// Failed to write to port user message
const ERROR_FAILED_TO_WRITE_TO_PORT: &str = "Failed to Write to Serial Port";

/// No Response received user message
const ERROR_NO_RESPONSE: &str = "Knock Knock - No Response";

/// Firmware reported error user message
const ERROR_FAILED_PROTOCOL_PROCESSING_REMOTE: &str = "Firmware reported error";

/// Local response processing failed user message
const ERROR_FAILED_PROTOCOL_PROCESSING_LOCAL: &str = "Failed to parse response";

/// Failed to read serial port bytes user message
const ERROR_FAILED_TO_READ_SERIAL_PORT_BYTES: &str = "Failed to read serial port bytes";

/// Timed out reading serial port user message
const ERROR_TIMEDOUT_READING_SERIAL_PORT: &str = "Timed out reading serial port";

/// Serial port error
const ERROR_SERIAL_PORT_ERROR: &str = "Serial Port Error";

///
/// Probes available ports for a LEDSC based device. Returns the SerialPortInfo for the first
/// device found.
///
pub fn auto_detect_ledsc() -> std::result::Result<SerialPortInfo, &'static str> {
    let ports = available_ports();

    match ports {
        Ok(port_vect) => {
            for p in port_vect {
                let result = auto_detect_ledsc_on_port(p);

                if result.is_ok() {
                    return Ok(result.unwrap());
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to get available serial ports: {:?}", e);
            return Err(ERROR_NO_AVAILABLE_PORTS);
        }
    }

    return Err(ERROR_NO_DEVICES_FOUND);
}

///
/// Attempts to open the given port and probe for a LEDSC based device.
///
fn auto_detect_ledsc_on_port(
    port_info: SerialPortInfo,
) -> std::result::Result<SerialPortInfo, &'static str> {
    match serialport::new(&port_info.port_name, LEDSC_BAUD).open() {
        Ok(mut serial_port) => {
            let protocol_instance = LedscTeensy001 {};

            // Create print version command
            let cmd: String = protocol_instance.create_cmd_string(Command::PrintVersion);

            // Write printer version command
            let write_result = serial_port.write_all(cmd.as_bytes());
            if write_result.is_ok() {
                let response_option = wait_for_response(&mut serial_port, RECEIVE_TIMEOUT_MS);

                if response_option.is_ok() {
                    match protocol_instance.parse_response_sting(response_option.unwrap()) {
                        ResponsePacketOption::Success(..) => return Ok(port_info),

                        ResponsePacketOption::FailedRemote(pkt) => {
                            eprintln!("Failed Remote: {:?}", pkt);
                            return Err(ERROR_FAILED_PROTOCOL_PROCESSING_REMOTE);
                        }

                        ResponsePacketOption::FailedLocal(pkt) => {
                            eprintln!("Failed Local: {:?}", pkt);
                            return Err(ERROR_FAILED_PROTOCOL_PROCESSING_LOCAL);
                        }
                    }
                } else {
                    eprintln!(
                        "Failed waiting for auto detect response: {:?}",
                        response_option
                    );
                    return Err(ERROR_NO_RESPONSE);
                }
            } else {
                eprintln!("Failed writing to serial port: {:?}", write_result);
                return Err(ERROR_FAILED_TO_WRITE_TO_PORT);
            }
        }
        Err(e) => {
            eprintln!("Auto detect failed to open serial port: {:?}", e);
            return Err(ERROR_FAILED_TO_OPEN_PORT);
        }
    }
}

///
/// Reads incoming data from serial port. Waits for data up-to timeout.This function will always
/// take at minimum timeout_ms to return. It waits cummulatively during periods of zero bytes.
/// Ex: timeout_ms = 500ms.
/// - waits 100ms for the first bytes to be awaiting read.
/// - reads available bytes in read buffer
/// - waits 50ms for more bytes to be available
/// - reads available bytes in read buffer
/// - waits 350ms, no new bytes received, exits.
///
fn wait_for_response(
    serial_port: &mut Box<dyn serialport::SerialPort>,
    timeout_ms: u64,
) -> std::result::Result<String, &'static str> {
    let sleep_ms: u64 = 10;
    let mut timeout_count_down = timeout_ms / sleep_ms;
    let mut receive_buffer = [0; 10];
    let mut received_bytes: Vec<u8> = vec![];

    // Count down until timeout
    while timeout_count_down > 0 {
        // Wait for bytes to be available
        loop {
            let bytes_to_read = serial_port.bytes_to_read();

            if bytes_to_read.is_err() {
                eprintln!("{:?}", bytes_to_read);
                return Err(ERROR_SERIAL_PORT_ERROR);
            }

            if bytes_to_read.unwrap() > 0 || timeout_count_down <= 0 {
                break;
            }

            thread::sleep(time::Duration::from_millis(sleep_ms));
            timeout_count_down -= 1;
        }

        // Read bytes available
        loop {
            let bytes_to_read = serial_port.bytes_to_read();

            if bytes_to_read.is_err() {
                eprintln!("{:?}", bytes_to_read);
                return Err(ERROR_SERIAL_PORT_ERROR);
            }

            let bytes_to_read = bytes_to_read.unwrap();

            if bytes_to_read <= 0 || timeout_count_down <= 0 {
                break;
            }

            let read_bytes_result = serial_port.read(&mut receive_buffer[..]);

            if read_bytes_result.is_err() {
                eprintln!("{:?}", read_bytes_result);
                return Err(ERROR_FAILED_TO_READ_SERIAL_PORT_BYTES);
            } else {
                received_bytes.append(&mut receive_buffer.to_vec());
            }
        }
    }

    if received_bytes.is_empty() {
        eprintln!("{:?}", received_bytes);
        return Err(ERROR_TIMEDOUT_READING_SERIAL_PORT);
    } else {
        let received_string = String::from_utf8(received_bytes).unwrap();
        return Ok(received_string);
    }
}

///
/// Sends a command and waits for the response
///
pub fn send_command_wait_for_response(
    port_info: &SerialPortInfo,
    cmd: String,
) -> std::result::Result<String, &'static str> {
    match serialport::new(&port_info.port_name, LEDSC_BAUD).open() {
        Ok(mut serial_port) => {
            let write_result = serial_port.write_all(cmd.as_bytes());

            if write_result.is_ok() {
                return wait_for_response(&mut serial_port, RECEIVE_TIMEOUT_MS);
            }

            eprintln!(
                "Senc command and wait failed to write to port: {:?}",
                write_result
            );
            Err(ERROR_FAILED_TO_WRITE_TO_PORT)
        }
        Err(e) => {
            eprintln!("Send command and wait failed to open serial port: {:?}", e);
            return Err(ERROR_FAILED_TO_OPEN_PORT);
        }
    }
}

/// -----------------
/// Unit Tests
/// -----------------
#[cfg(test)]
mod test {

    use crate::led_strip_controller::color::*;
    use crate::led_strip_controller::controller;
    use crate::led_strip_controller::protocol::*;
    use std::{thread, time};

    /// Tests will only pass if hardware is connected and available
    const HW_AVAILABLE: bool = true;

    #[test]
    fn send_command_wait_for_response_test() {
        match controller::auto_detect_ledsc() {
            Ok(port_info) => {
                assert!(HW_AVAILABLE, "Found LEDSC");

                let protocol_instance = LedscTeensy001 {};
                let cmd =
                    protocol_instance.create_cmd_string(Command::SetEffect(Effect::SolidColor));

                match controller::send_command_wait_for_response(&port_info, cmd) {
                    Ok(_rsp_pkt) => assert!(HW_AVAILABLE, "Set Effect - Solid color"),
                    Err(rsp_pkt) => {
                        assert!(false, "Failed to set effect {:?}", rsp_pkt);
                    }
                }

                let cmd = protocol_instance
                    .create_cmd_string(Command::SetColor(Color24::from_u32(0xff0000)));

                match controller::send_command_wait_for_response(&port_info, cmd) {
                    Ok(_rsp_pkt) => assert!(HW_AVAILABLE, "Set Color - Red"),
                    Err(rsp_pkt) => {
                        assert!(false, "Failed to set color {:?}", rsp_pkt);
                    }
                }

                thread::sleep(time::Duration::from_millis(500));

                let cmd = protocol_instance
                    .create_cmd_string(Command::SetColor(Color24::from_u32(0x00ff00)));

                match controller::send_command_wait_for_response(&port_info, cmd) {
                    Ok(_rsp_pkt) => assert!(HW_AVAILABLE, "Set Color - Green"),
                    Err(rsp_pkt) => {
                        assert!(false, "Failed to set color {:?}", rsp_pkt);
                    }
                }

                thread::sleep(time::Duration::from_millis(500));

                let cmd = protocol_instance
                    .create_cmd_string(Command::SetColor(Color24::from_u32(0x0000ff)));

                match controller::send_command_wait_for_response(&port_info, cmd) {
                    Ok(_rsp_pkt) => assert!(HW_AVAILABLE, "Set Color - Blue"),
                    Err(rsp_pkt) => {
                        assert!(false, "Failed to set color {:?}", rsp_pkt);
                    }
                }

                thread::sleep(time::Duration::from_millis(500));

                let cmd = protocol_instance.create_cmd_string(Command::SetBrightness(0xff));

                match controller::send_command_wait_for_response(&port_info, cmd) {
                    Ok(_rsp_pkt) => assert!(HW_AVAILABLE, "Set Brightness - 100%"),
                    Err(rsp_pkt) => {
                        assert!(false, "Failed to set brightness {:?}", rsp_pkt);
                    }
                }

                thread::sleep(time::Duration::from_millis(500));

                let cmd = protocol_instance.create_cmd_string(Command::SetBrightness(0x88));

                match controller::send_command_wait_for_response(&port_info, cmd) {
                    Ok(_rsp_pkt) => assert!(HW_AVAILABLE, "Set Brightness - 50%"),
                    Err(rsp_pkt) => {
                        assert!(false, "Failed to set brightness {:?}", rsp_pkt);
                    }
                }

                thread::sleep(time::Duration::from_millis(500));

                let cmd = protocol_instance.create_cmd_string(Command::SetBrightness(0x22));

                match controller::send_command_wait_for_response(&port_info, cmd) {
                    Ok(_rsp_pkt) => assert!(HW_AVAILABLE, "Set Brightness - 13%"),
                    Err(rsp_pkt) => {
                        assert!(false, "Failed to set brightness {:?}", rsp_pkt);
                    }
                }

                thread::sleep(time::Duration::from_millis(500));

                let cmd =
                    protocol_instance.create_cmd_string(Command::SetEffect(Effect::CometRainbow));

                match controller::send_command_wait_for_response(&port_info, cmd) {
                    Ok(_rsp_pkt) => assert!(HW_AVAILABLE, "Set Effect CometRainbow"),
                    Err(rsp_pkt) => {
                        assert!(false, "Failed to set effect CometRainbow {:?}", rsp_pkt);
                    }
                }

                thread::sleep(time::Duration::from_millis(5000));

                let cmd =
                    protocol_instance.create_cmd_string(Command::SetEffect(Effect::RainbowCycle));

                match controller::send_command_wait_for_response(&port_info, cmd) {
                    Ok(_rsp_pkt) => assert!(HW_AVAILABLE, "Set Effect RainbowCycle"),
                    Err(rsp_pkt) => {
                        assert!(false, "Failed to set effect rainbowcycle {:?}", rsp_pkt);
                    }
                }
            }
            Err(_e) => assert!(false, "Failed to find LEDSC"),
        }
    }
}
