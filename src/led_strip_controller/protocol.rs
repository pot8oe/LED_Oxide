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

use crate::led_strip_controller::color;
use crc16::*;
use std::str::Chars;

/**
* Input Packet structure
* [CMD:param_1:param_2:param_3:param_4]\r
*
* Parameters are optional and command dependant
*
* */

/**
 * Response Packet structure
 * [CMD:param_1:param_2:param_3:param_4]\r
 *
 * Param 1 is the command status/error code
 * Parameters 2-4 are optional and command dependant
 *
 * */

// --------------------------------------
// - Protocol characters
// --------------------------------------

/// Start Transmission character
const PROTO_STX: char = '[';

/// End Transmission character
const PROTO_ETX: char = ']';

/// Parameter separator character
const PROTO_PSC: char = ':';

/// Carriage Return character
const PROTO_CR: char = '\r';

/// New line character
const PROTO_NL: char = '\n';

// --------------------------------------
// - Limits
// --------------------------------------

/// Max length of a packet
const MAX_PROTO_PACKET_LEN: i16 = 256;

/// Max length of a command
const MAX_PROTO_CMD: i16 = 10;

/// Max number of parameters for any one command packet
const MAX_PROTO_PARAM_COUNT: i16 = 4;

/// Max number of characters for any one parameter assuming there is only one param
const MAX_PROTO_PARAM_LEN: i16 = 50;

// --------------------------------------
// - Known Protocol Versions
// --------------------------------------

/// Unknown Firmware
const FWV_LEDSC_UNKNOWN: &str = "UNKNOWN";

/// LEDSC_Teensy_ prefix
const FWV_LEDSC_TEENSY: &str = "LEDSC_TEENSY_";

/// LEDSC_Teensy_001
const FWV_LEDSC_TEENSY_001: &str = "LEDSC_TEENSY_001";

// --------------------------------------
// - Commands
// --------------------------------------

/// Command print version
const CMD_PRINT_VERSION: &str = "CPV";

/// Command full firmware reset
const CMD_FULL_RESET: &str = "CFR";

/// Command enter bootloader
const CMD_ENTER_BOOTLOADER: &str = "CEB";

/// Command set debugging
const CMD_SET_DEBUGGING: &str = "CSD";

/// Command set LED effect
const CMD_SET_EFFECT: &str = "CSE";

/// Command set color
const CMD_SET_COLOR: &str = "CSC";

/// Command set brightness
const CMD_SET_BRIGHTNESS: &str = "CSB";

/// Command set fire pallet
const CMD_SET_FIRE_PALLET: &str = "CSFP";

/// Command get status
const CMD_GET_STATUS: &str = "CGS";

// --------------------------------------
// - Error Codes
// --------------------------------------

/// Code for success no error.
const ERR_PROTO_SUCCESS: i16 = 0;

/// Generic command processing error
const ERR_PROTO_CMD_PARSING: i16 = -100;

/// Missing expected STX character
const ERR_PROTO_CP_MISSING_STX: i16 = -101;

/// Missing expected ETX character
const ERR_PROTO_CP_MISSING_ETX: i16 = -102;

/// Missing expected PSC character
const ERR_PROTO_CP_MISSING_PSC: i16 = -103;

/// Missing expected framing character
const ERR_PROTO_CP_MISSING_EFC: i16 = -104;

/// Command buffer overflow
const ERR_PROTO_CP_CMD_OVERFLOW: i16 = -105;

/// Command not implemented
const ERR_PROTO_CP_CMD_NOT_IMP: i16 = -106;

/// Unknown command
const ERR_PROTO_CP_CMD_UNKNOWN: i16 = -107;

/// Missing parameters
const ERR_PROTO_CP_MISSING_PARAMS: i16 = -108;

/// Parameter out of range
const ERR_PROTO_CP_PARAM_OUT_RANGE: i16 = -109;

/// CRC16 mismatch
const ERR_PROTO_CP_CRC16_MISMATCH: i16 = -110;

/// CRC16 missing
const ERR_PROTO_CP_MISSING_CRC16: i16 = -111;

/// Response packet error
const ERR_PROTO_RSP_BUILDING: i16 = -200;

/// Too many params attempted in response packet
const ERR_PROTO_RB_TOO_MANY_PARAMS: i16 = -201;

/// Param buffer overflow
const ERR_PROTO_RB_PARAM_OVERFLOW: i16 = -202;

/// ADC Error
const ERR_ADC: i16 = -300;

/// Failed to read ADC
const ERR_ADC_READFAIL: i16 = -301;

/// ADC Register Depth error. Occurs when attemtping to R/W ADC register with incorrect size value.
const ERR_ADC_REGISTER_DEPTH: i16 = -302;

/// Set Movetohall config
const ERR_SMC: i16 = -400;

/// Polynomial index out of range
const ERR_SMC_POLY_INDEX_OOR: i16 = -401;

///
/// Represents possible LED Strip effects.
///
pub enum Effect {
    Off,
    SolidColor,
    RainbowCycle,
    Comet,
    CometRainbow,
    Fire,
    FireColor,
    SolidColorPulse,
    BouncingBall,
    Twinkle,
    MaxEffect,
}

pub enum FireColorPallet {
    Heat,
    Party,
    Rainbow,
    RainbowStripe,
    Forest,
    Ocean,
    Lava,
    Cloud,
}

///
/// Represents possible LED Strip Controller commands
///
pub enum Command {
    None,
    PrintVersion,
    FullReset,
    EnterBootloader,
    SetDebugging(bool),
    SetEffect(Effect),
    SetColor(color::Color24),
    SetBrightness(u8),
    SetFireColorPallet(FireColorPallet),
    GetStatus,
}

///
/// Known Firmware Protocol versions
///
pub enum KnownProtocolVersions {
    /// An unknown firmware version
    Unknown,
    /// Version LEDSC_Teensy_001
    LedscTeensy001,
    /// A Version of LEDSC_Teensy_* newer than this software knows
    LedscTeensyNewer,
}

impl KnownProtocolVersions {
    fn value(&self) -> &str {
        match self {
            KnownProtocolVersions::Unknown => FWV_LEDSC_UNKNOWN,
            KnownProtocolVersions::LedscTeensy001 => FWV_LEDSC_TEENSY_001,
            KnownProtocolVersions::LedscTeensyNewer => FWV_LEDSC_TEENSY,
            // _ => "UNKNOWN"
        }
    }
}

///
/// Gets the known protocol version for the given String.
///
pub fn get_known_protocol_version_from_str(proto_str: &str) -> Option<KnownProtocolVersions> {
    let proto_str_upper = proto_str.to_uppercase();

    return match proto_str_upper.as_str() {
        FWV_LEDSC_UNKNOWN => Some(KnownProtocolVersions::Unknown),
        FWV_LEDSC_TEENSY_001 => Some(KnownProtocolVersions::LedscTeensy001),
        _ => {
            if proto_str_upper.as_str().contains(FWV_LEDSC_TEENSY) {
                return Some(KnownProtocolVersions::LedscTeensyNewer);
            }

            Some(KnownProtocolVersions::Unknown)
        }
    };
}

///
/// Matches the given string to a known protocol version code and returns the proper object.
/// If the protocol is unknown the oldest version is returned.
/// If the protocol appears to be newer than a known version than the latest known version
/// is returned.
///
pub fn get_protocol_version_impl_from_str(protocol_str: &str) -> Box<dyn ProtocolVersion> {
    match get_known_protocol_version_from_str(&protocol_str) {
        Some(KnownProtocolVersions::Unknown) => Box::new(LedscTeensy001 {}),
        Some(KnownProtocolVersions::LedscTeensy001) => Box::new(LedscTeensy001 {}),
        Some(KnownProtocolVersions::LedscTeensyNewer) => Box::new(LedscTeensy001 {}),
        None => Box::new(LedscTeensy001 {}),
    }
}

///
/// Represents a response packet option returned by parsing a response packet.
///
pub enum ResponsePacketOption {
    /// Value when a reponse packet is parsed correctly and represents a success from firmware.
    Success(ResponsePacket),
    /// Value when a reponse pacakge is parsed correcly and represents a failure code from firmware.
    FailedRemote(ResponsePacket),
    /// Value when a response pacakge is failed to be parsed.
    FailedLocal(i16),
}

///
/// Represents a parsed response packet.
///
#[derive(Debug)]
pub struct ResponsePacket {
    pub command: String,
    pub parameters: Vec<String>,
    pub crc16_in: u16,
    pub crc16_calc: u16,
}

///
/// Trait describing necessary functions for a given protocol version. Each known protocol version
/// should implement this trait. The base implmentation of functions support TKJLED_Teensy_001.
///
pub trait ProtocolVersion {
    fn get_version_code(&self) -> &str;

    ///
    /// Return if the given command is supported by this version
    ///
    fn is_cmd_supported(&self, command: &Command) -> bool;

    ///
    /// Return if the given effect is supported by thhis version
    ///
    fn is_effect_supported(&self, effect: &Effect) -> bool;

    ///
    /// Returns the command's int value used when creating a command string to be sent to the firmware.
    ///
    fn get_effect_cmd_value(&self, effect: &Effect) -> u8;

    ///
    /// Returns the effect represented by the given command value.
    ///
    fn get_effect_from_cmd_value(&self, effect_id: &u8) -> Effect;

    ///
    /// Returns the color pallet int value used when creating a command string to be sent to the firmware.
    ///
    fn get_fire_color_pallet_value(&self, pallet: &FireColorPallet) -> u8;

    ///
    /// Returns the fire color pallet represented by the given pallet id.
    ///
    fn get_fire_color_pallet_from_cmd_value(&self, pallet_id: &u8) -> FireColorPallet;

    ///
    /// Returns the command string to be sent for the given command packet.
    ///
    fn create_cmd_string(&self, command: Command) -> String {
        // Start TX
        let mut cmd_str: String = String::from(PROTO_STX);

        // Command & Parameters
        match &command {
            Command::None => {}
            Command::PrintVersion => cmd_str.push_str(CMD_PRINT_VERSION),
            Command::FullReset => cmd_str.push_str(CMD_FULL_RESET),
            Command::EnterBootloader => cmd_str.push_str(CMD_ENTER_BOOTLOADER),
            Command::SetDebugging(state) => {
                cmd_str.push_str(CMD_SET_DEBUGGING);
                cmd_str.push(PROTO_PSC);
                if *state == true {
                    cmd_str.push_str("0x01");
                } else {
                    cmd_str.push_str("0x00");
                }
            }
            Command::SetEffect(effect) => {
                cmd_str.push_str(CMD_SET_EFFECT);
                cmd_str.push(PROTO_PSC);
                cmd_str.push_str(format!("{:X}", self.get_effect_cmd_value(&effect)).as_str());
            }
            Command::SetColor(color) => {
                cmd_str.push_str(CMD_SET_COLOR);
                cmd_str.push(PROTO_PSC);
                cmd_str.push_str(format!("{:X}", color.to_u32()).as_str());
            }
            Command::SetBrightness(brightness) => {
                cmd_str.push_str(CMD_SET_BRIGHTNESS);
                cmd_str.push(PROTO_PSC);
                cmd_str.push_str(format!("{:X}", brightness).as_str());
            }
            Command::SetFireColorPallet(fire_color) => {
                cmd_str.push_str(CMD_SET_FIRE_PALLET);
                cmd_str.push(PROTO_PSC);
                cmd_str.push_str(
                    format!("{:X}", self.get_fire_color_pallet_value(&fire_color)).as_str(),
                );
            }
            Command::GetStatus => cmd_str.push_str(CMD_GET_STATUS),
        };

        // End TX
        cmd_str.push(PROTO_ETX);

        // CRC16 - XMODEM
        cmd_str.push_str(format!("{:X}", State::<XMODEM>::calculate(cmd_str.as_bytes())).as_str());

        // carriage return line feed
        cmd_str.push(PROTO_CR);
        cmd_str.push(PROTO_NL);

        // Return immutable command string
        let cmd_str = cmd_str;

        cmd_str
    }

    ///
    /// Parses the input string and returns a ResponsePacket
    ///
    fn parse_response_sting(&self, response_str: String) -> ResponsePacketOption {
        // init character iterator
        let mut response_chars: Chars<'_> = response_str.trim().chars();

        if response_chars.next() != Some(PROTO_STX) {
            // Stop: Missing STX
            return ResponsePacketOption::FailedLocal(ERR_PROTO_CP_MISSING_STX);
        }

        // Init Command string
        let mut cmd: String = String::from("");

        // Init CRC16
        let mut state_crc_16 = State::<XMODEM>::new();
        state_crc_16.update(PROTO_STX.to_string().as_bytes());

        // Read next char post PROTO_STX
        let mut current_char: Option<char> = response_chars.next();
        state_crc_16.update(current_char.unwrap().to_string().as_bytes());

        // Read Command
        while current_char != Some(PROTO_PSC)
            && current_char != Some(PROTO_ETX)
            && current_char != None
        {
            cmd.push(current_char.unwrap());

            current_char = response_chars.next();
            state_crc_16.update(current_char.unwrap().to_string().as_bytes());
        }

        // Read parameters if present
        let mut params_in: Vec<String> = vec![];

        while current_char == Some(PROTO_PSC) {
            current_char = response_chars.next();
            state_crc_16.update(current_char.unwrap().to_string().as_bytes());

            let mut param: String = String::from("");

            while current_char != Some(PROTO_PSC)
                && current_char != Some(PROTO_ETX)
                && current_char != None
            {
                param.push(current_char.unwrap());

                current_char = response_chars.next();
                state_crc_16.update(current_char.unwrap().to_string().as_bytes());
            }

            params_in.push(param);
        }

        if params_in.len() < 1 {
            // Should always get at minimum 1 parameter, status code.
            return ResponsePacketOption::FailedLocal(ERR_PROTO_CP_MISSING_PARAMS);
        }

        if current_char != Some(PROTO_ETX) {
            // Stop: Missing ETX
            return ResponsePacketOption::FailedLocal(ERR_PROTO_CP_MISSING_ETX);
        }

        // Read CRC16
        current_char = response_chars.next();
        let mut crc16_in_str: String = String::from("");
        while current_char != Some(PROTO_CR) && current_char != None {
            crc16_in_str.push(current_char.unwrap());
            current_char = response_chars.next();
        }

        // Create response packet object
        let response_packet = ResponsePacket {
            command: cmd,
            parameters: params_in,
            crc16_in: match u16::from_str_radix(crc16_in_str.as_str(), 16) {
                Result::Ok(value) => value,
                Result::Err(..) => 0x00,
            },
            crc16_calc: state_crc_16.get(),
        };

        let success_str: String = format!("{}", ERR_PROTO_SUCCESS);

        // Mark ResponsePacketOption::FailedRemote() if param 1 is not OK
        if Some(&success_str) == response_packet.parameters.get(0) {
            // Return success
            return ResponsePacketOption::Success(response_packet);
        }

        return ResponsePacketOption::FailedRemote(response_packet);
    }
}

///
/// Type object for TKJRLED_TEENSY_001 firmware version
///
pub struct LedscTeensy001 {}

///
/// ProtocolVersion implementation for TKJRLED_TEENSY_001 firmware version.
///
impl ProtocolVersion for LedscTeensy001 {
    fn get_version_code(&self) -> &str {
        FWV_LEDSC_TEENSY_001
    }

    ///
    /// Return if the given command is supported by this version
    ///
    fn is_cmd_supported(&self, command: &Command) -> bool {
        match &command {
            Command::None => false,
            Command::PrintVersion => true,
            Command::FullReset => false,
            Command::EnterBootloader => false,
            Command::SetDebugging(..) => true,
            Command::SetEffect(effect) => self.is_effect_supported(effect),
            Command::SetColor(..) => true,
            Command::SetBrightness(..) => true,
            Command::SetFireColorPallet(..) => true,
            Command::GetStatus => true,
            // Will need this if future commands are implemented newer firmware
            // _ => false
        }
    }

    ///
    /// Return if the given effect is supported by thhis version
    ///
    fn is_effect_supported(&self, effect: &Effect) -> bool {
        match &effect {
            Effect::Off => true,
            Effect::SolidColor => true,
            Effect::RainbowCycle => true,
            Effect::Comet => true,
            Effect::CometRainbow => true,
            Effect::Fire => true,
            Effect::FireColor => true,
            Effect::SolidColorPulse => true,
            Effect::BouncingBall => true,
            Effect::Twinkle => true,
            Effect::MaxEffect => true,
            // Will need this if future effects are implemented in newer firmware
            // _ => false
        }
    }

    ///
    /// Returns the command's int value used when creating a command string to be sent to the firmware.
    ///
    fn get_effect_cmd_value(&self, effect: &Effect) -> u8 {
        match &effect {
            Effect::Off => 0x00,
            Effect::SolidColor => 0x01,
            Effect::RainbowCycle => 0x02,
            Effect::Comet => 0x03,
            Effect::CometRainbow => 0x04,
            Effect::Fire => 0x05,
            Effect::FireColor => 0x06,
            Effect::SolidColorPulse => 0x07,
            Effect::BouncingBall => 0x08,
            Effect::Twinkle => 0x09,
            Effect::MaxEffect => 0x0a,
        }
    }

    ///
    /// Returns the effect represented by the given command value.
    /// Returns off for any unknown effect id's
    ///
    fn get_effect_from_cmd_value(&self, effect_id: &u8) -> Effect {
        match &effect_id {
            0x00 => Effect::Off,
            0x01 => Effect::SolidColor,
            0x02 => Effect::RainbowCycle,
            0x03 => Effect::Comet,
            0x04 => Effect::CometRainbow,
            0x05 => Effect::Fire,
            0x06 => Effect::FireColor,
            0x07 => Effect::SolidColorPulse,
            0x08 => Effect::BouncingBall,
            0x09 => Effect::Twinkle,
            0x0a => Effect::MaxEffect,
            _ => Effect::Off,
        }
    }

    ///
    /// Returns the color pallet int value used when creating a command string to be sent to the firmware.
    ///
    fn get_fire_color_pallet_value(&self, pallet: &FireColorPallet) -> u8 {
        match &pallet {
            FireColorPallet::Heat => 0x00,
            FireColorPallet::Party => 0x01,
            FireColorPallet::Rainbow => 0x02,
            FireColorPallet::RainbowStripe => 0x03,
            FireColorPallet::Forest => 0x04,
            FireColorPallet::Ocean => 0x05,
            FireColorPallet::Lava => 0x06,
            FireColorPallet::Cloud => 0x07,
        }
    }

    ///
    /// Returns the fire color pallet represented by the given pallet id.
    ///
    fn get_fire_color_pallet_from_cmd_value(&self, pallet_id: &u8) -> FireColorPallet {
        match &pallet_id {
            0x00 => FireColorPallet::Heat,
            0x01 => FireColorPallet::Party,
            0x02 => FireColorPallet::Rainbow,
            0x03 => FireColorPallet::RainbowStripe,
            0x04 => FireColorPallet::Forest,
            0x05 => FireColorPallet::Ocean,
            0x06 => FireColorPallet::Lava,
            0x07 => FireColorPallet::Cloud,
            _ => FireColorPallet::Heat,
        }
    }
}

/// -----------------
/// Unit Tests
/// -----------------
#[cfg(test)]
mod test {
    use crate::led_strip_controller::color::Color24;
    use crate::led_strip_controller::protocol;
    use crate::led_strip_controller::protocol::ProtocolVersion;
    use crate::led_strip_controller::protocol::{
        Effect, CMD_ENTER_BOOTLOADER, CMD_FULL_RESET, CMD_PRINT_VERSION, CMD_SET_BRIGHTNESS,
        CMD_SET_COLOR, CMD_SET_EFFECT, PROTO_CR, PROTO_ETX, PROTO_NL, PROTO_PSC, PROTO_STX,
    };

    #[test]
    fn get_effect_cmd_value_test() {
        let protocol_version = protocol::LedscTeensy001 {};

        assert_eq!(
            protocol_version.get_effect_cmd_value(&protocol::Effect::Off),
            0x00
        );
        assert_eq!(
            protocol_version.get_effect_cmd_value(&protocol::Effect::SolidColor),
            0x01
        );
        assert_eq!(
            protocol_version.get_effect_cmd_value(&protocol::Effect::RainbowCycle),
            0x02
        );
        assert_eq!(
            protocol_version.get_effect_cmd_value(&protocol::Effect::Comet),
            0x03
        );
        assert_eq!(
            protocol_version.get_effect_cmd_value(&protocol::Effect::CometRainbow),
            0x04
        );
        assert_eq!(
            protocol_version.get_effect_cmd_value(&protocol::Effect::FireColor),
            0x06
        );
        assert_eq!(
            protocol_version.get_effect_cmd_value(&protocol::Effect::Fire),
            0x05
        );
        assert_eq!(
            protocol_version.get_effect_cmd_value(&protocol::Effect::SolidColorPulse),
            0x07
        );
        assert_eq!(
            protocol_version.get_effect_cmd_value(&protocol::Effect::BouncingBall),
            0x08
        );
        assert_eq!(
            protocol_version.get_effect_cmd_value(&protocol::Effect::Twinkle),
            0x09
        );
        assert_eq!(
            protocol_version.get_effect_cmd_value(&protocol::Effect::MaxEffect),
            0x0a
        );
    }

    #[test]
    fn get_fire_color_pallet_value_test() {
        let protocol_version = protocol::LedscTeensy001 {};

        assert_eq!(
            protocol_version.get_fire_color_pallet_value(&protocol::FireColorPallet::Heat),
            0x00
        );
        assert_eq!(
            protocol_version.get_fire_color_pallet_value(&protocol::FireColorPallet::Party),
            0x01
        );
        assert_eq!(
            protocol_version.get_fire_color_pallet_value(&protocol::FireColorPallet::Rainbow),
            0x02
        );
        assert_eq!(
            protocol_version.get_fire_color_pallet_value(&protocol::FireColorPallet::RainbowStripe),
            0x03
        );
        assert_eq!(
            protocol_version.get_fire_color_pallet_value(&protocol::FireColorPallet::Forest),
            0x04
        );
        assert_eq!(
            protocol_version.get_fire_color_pallet_value(&protocol::FireColorPallet::Ocean),
            0x05
        );
        assert_eq!(
            protocol_version.get_fire_color_pallet_value(&protocol::FireColorPallet::Lava),
            0x06
        );
        assert_eq!(
            protocol_version.get_fire_color_pallet_value(&protocol::FireColorPallet::Cloud),
            0x07
        );
    }

    #[test]
    fn packet_command_get_cmd_string_test() {
        let protocol_version = protocol::LedscTeensy001 {};

        let test_str: String = format!(
            "{}{}{}{}{}{}",
            PROTO_STX, CMD_PRINT_VERSION, PROTO_ETX, "7D02", PROTO_CR, PROTO_NL
        );

        assert_eq!(
            protocol_version.create_cmd_string(protocol::Command::PrintVersion),
            test_str
        );

        let test_str: String = format!(
            "{}{}{}{}{}{}",
            PROTO_STX, CMD_ENTER_BOOTLOADER, PROTO_ETX, "1A26", PROTO_CR, PROTO_NL
        );

        assert_eq!(
            protocol_version.create_cmd_string(protocol::Command::EnterBootloader),
            test_str
        );

        let test_str: String = format!(
            "{}{}{}{}{}{}",
            PROTO_STX, CMD_FULL_RESET, PROTO_ETX, "4005", PROTO_CR, PROTO_NL
        );

        assert_eq!(
            protocol_version.create_cmd_string(protocol::Command::FullReset),
            test_str
        );

        let test_str: String = format!(
            "{}{}{}{}{}{}{}{}",
            PROTO_STX, CMD_SET_EFFECT, PROTO_PSC, "4", PROTO_ETX, "6C1C", PROTO_CR, PROTO_NL
        );

        assert_eq!(
            protocol_version.create_cmd_string(protocol::Command::SetEffect(Effect::CometRainbow)),
            test_str
        );

        let test_str: String = format!(
            "{}{}{}{}{}{}{}{}",
            PROTO_STX, CMD_SET_BRIGHTNESS, PROTO_PSC, "5C", PROTO_ETX, "4AEA", PROTO_CR, PROTO_NL
        );

        assert_eq!(
            protocol_version.create_cmd_string(protocol::Command::SetBrightness(0x5c)),
            test_str
        );

        let test_str: String = format!(
            "{}{}{}{}{}{}{}{}",
            PROTO_STX, CMD_SET_COLOR, PROTO_PSC, "4F2D86", PROTO_ETX, "E1A3", PROTO_CR, PROTO_NL
        );

        assert_eq!(
            protocol_version
                .create_cmd_string(protocol::Command::SetColor(Color24::from_u32(0x004F2D86))),
            test_str
        );
    }

    #[test]
    fn parse_response_sting_test() {
        let protocol_version = protocol::LedscTeensy001 {};

        //
        // Set Effect OK response
        //
        let test_string: String = String::from("[CSE:0]A0D8");

        let response: protocol::ResponsePacketOption =
            protocol_version.parse_response_sting(test_string);

        match response {
            protocol::ResponsePacketOption::Success(pkt) => {
                assert_eq!(pkt.command, "CSE");
                assert_eq!(pkt.parameters[0], "0");
                assert_eq!(pkt.crc16_in, 0xA0D8);
                assert_eq!(pkt.crc16_calc, 0xA0D8);
            }
            protocol::ResponsePacketOption::FailedRemote(..) => assert!(
                false,
                "Parsing '[CSE:0]A0D8' should not return failed remote."
            ),
            protocol::ResponsePacketOption::FailedLocal(..) => assert!(
                false,
                "Parsing '[CSE:0]A0D8' should not return failed local."
            ),
        }

        //
        // Set Brightness OK response
        //
        let test_string: String = String::from("[CSB:0]F1F5");

        let response: protocol::ResponsePacketOption =
            protocol_version.parse_response_sting(test_string);

        match response {
            protocol::ResponsePacketOption::Success(pkt) => {
                assert_eq!(pkt.command, "CSB");
                assert_eq!(pkt.parameters[0], "0");
                assert_eq!(pkt.crc16_in, 0xF1F5);
                assert_eq!(pkt.crc16_calc, 0xF1F5);
            }
            protocol::ResponsePacketOption::FailedRemote(..) => assert!(
                false,
                "Parsing '[CSB:0]F1F5' should not return failed remote."
            ),
            protocol::ResponsePacketOption::FailedLocal(..) => assert!(
                false,
                "Parsing '[CSB:0]F1F5' should not return failed lcoal."
            ),
        }

        //
        // Missing framing character response
        //
        let test_string: String = String::from("[CS:-104]599D");

        let response: protocol::ResponsePacketOption =
            protocol_version.parse_response_sting(test_string);

        match response {
            protocol::ResponsePacketOption::Success(_pkt) => {
                assert!(false, "Parsing '[CS:-104]599D' should not return success.")
            }
            protocol::ResponsePacketOption::FailedRemote(pkt) => {
                assert_eq!(pkt.command, "CS");
                assert_eq!(pkt.parameters[0], "-104");
                assert_eq!(pkt.crc16_in, 0x599D);
                assert_eq!(pkt.crc16_calc, 0x599D);
            }
            protocol::ResponsePacketOption::FailedLocal(..) => assert!(
                false,
                "Parsing '[CS:-104]599D' should not return failed local."
            ),
        }
    }

    #[test]
    fn get_known_protocol_version_from_str_test() {
        // Checking standard 001 all caps
        match protocol::get_known_protocol_version_from_str("LEDSC_TEENSY_001") {
            Some(protocol::KnownProtocolVersions::LedscTeensy001) => {
                assert!(true, "This is correct")
            }
            _ => assert!(
                false,
                "Failed Checking LEDSC_TEENSY_001 to LedscTeensy001 value"
            ),
        }

        // Checking lower case semi-incorrect formatting but should pass
        match protocol::get_known_protocol_version_from_str("Ledsc_teensy_001") {
            Some(protocol::KnownProtocolVersions::LedscTeensy001) => {
                assert!(true, "This is correct")
            }
            _ => assert!(
                false,
                "Failed Checking Ledsc_teensy_001 to LedscTeensy001 value"
            ),
        }

        // Checking some newer version than we know about
        match protocol::get_known_protocol_version_from_str("LEDSC_TEENSY_256") {
            Some(protocol::KnownProtocolVersions::LedscTeensyNewer) => {
                assert!(true, "This is correct")
            }
            _ => assert!(
                false,
                "Failed Checking LEDSC_TEENSY_256 to LedscTeensyNewer value"
            ),
        }

        // Testing garbage input
        match protocol::get_known_protocol_version_from_str("Something") {
            Some(protocol::KnownProtocolVersions::Unknown) => assert!(true, "This is correct"),
            _ => assert!(false, "Failed Checking garbage input to Unknown value"),
        }
    }

    //
    // R&D Test to determine the correct algorithm
    // use crc16::*;
    //
    // #[test]
    // fn crc16_algo_check() {
    //
    //     let cmd_str: String = String::from("[CPV:-111]");
    //
    //     println!("7A37 ?? {:X}",State::<ARC>::calculate(cmd_str.as_bytes()));
    //     println!("7A37 ?? {:X}",State::<AUG_CCITT>::calculate(cmd_str.as_bytes()));
    //     println!("7A37 ?? {:X}",State::<BUYPASS>::calculate(cmd_str.as_bytes()));
    //     println!("7A37 ?? {:X}",State::<CCITT_FALSE>::calculate(cmd_str.as_bytes()));
    //     println!("7A37 ?? {:X}",State::<CDMA2000>::calculate(cmd_str.as_bytes()));
    //     println!("7A37 ?? {:X}",State::<CRC_A>::calculate(cmd_str.as_bytes()));
    //     println!("7A37 ?? {:X}",State::<DDS_110>::calculate(cmd_str.as_bytes()));
    //     println!("7A37 ?? {:X}",State::<DECT_R>::calculate(cmd_str.as_bytes()));
    //     println!("7A37 ?? {:X}",State::<DECT_X>::calculate(cmd_str.as_bytes()));
    //     println!("7A37 ?? {:X}",State::<DNP>::calculate(cmd_str.as_bytes()));
    //     println!("7A37 ?? {:X}",State::<EN_13757>::calculate(cmd_str.as_bytes()));
    //     println!("7A37 ?? {:X}",State::<GENIBUS>::calculate(cmd_str.as_bytes()));
    //     println!("7A37 ?? {:X}",State::<KERMIT>::calculate(cmd_str.as_bytes()));
    //     println!("7A37 ?? {:X}",State::<MAXIM>::calculate(cmd_str.as_bytes()));
    //     println!("7A37 ?? {:X}",State::<MCRF4XX>::calculate(cmd_str.as_bytes()));
    //     println!("7A37 ?? {:X}",State::<MODBUS>::calculate(cmd_str.as_bytes()));
    //     println!("7A37 ?? {:X}",State::<RIELLO>::calculate(cmd_str.as_bytes()));
    //     println!("7A37 ?? {:X}",State::<T10_DIF>::calculate(cmd_str.as_bytes()));
    //     println!("7A37 ?? {:X}",State::<TELEDISK>::calculate(cmd_str.as_bytes()));
    //     println!("7A37 ?? {:X}",State::<TMS37157>::calculate(cmd_str.as_bytes()));
    //     println!("7A37 ?? {:X}",State::<USB>::calculate(cmd_str.as_bytes()));
    //     println!("7A37 ?? {:X}",State::<XMODEM>::calculate(cmd_str.as_bytes()));
    //     println!("7A37 ?? {:X}",State::<X_25>::calculate(cmd_str.as_bytes()));
    //
    // }
}
