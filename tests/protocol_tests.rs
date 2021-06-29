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
 
//
// #[cfg(test)]
// mod test {
//
//     use led_oxide::led_strip_controller::protocol;
//
//     #[test]
//     fn get_effect_cmd_value_test() {
//
//         assert_eq!(protocol::get_effect_cmd_value(&protocol::Effect::Off), 0x00);
//         assert_eq!(protocol::get_effect_cmd_value(&protocol::Effect::SolidColor), 0x01);
//         assert_eq!(protocol::get_effect_cmd_value(&protocol::Effect::RainbowCycle), 0x02);
//         assert_eq!(protocol::get_effect_cmd_value(&protocol::Effect::Comet), 0x03);
//         assert_eq!(protocol::get_effect_cmd_value(&protocol::Effect::CometRainbow), 0x04);
//         assert_eq!(protocol::get_effect_cmd_value(&protocol::Effect::FireColor), 0x06);
//         assert_eq!(protocol::get_effect_cmd_value(&protocol::Effect::Fire), 0x05);
//         assert_eq!(protocol::get_effect_cmd_value(&protocol::Effect::SolidColorPulse), 0x07);
//         assert_eq!(protocol::get_effect_cmd_value(&protocol::Effect::BouncingBall), 0x08);
//         assert_eq!(protocol::get_effect_cmd_value(&protocol::Effect::Twinkle), 0x09);
//         assert_eq!(protocol::get_effect_cmd_value(&protocol::Effect::MaxEffect), 0x0a);
//     }
// }
