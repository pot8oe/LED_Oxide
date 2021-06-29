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

/// Represents a 24bit RGB Color
///
pub struct Color24 {
    r: u8,
    g: u8,
    b: u8,
}

impl Color24 {

    ///
    /// Convert Color24 to a u32
    ///
    pub fn to_u32(&self) -> u32 {
        let r: u32 = self.r.into();
        let g: u32 = self.g.into();
        let b: u32 = self.b.into();

        r << 16 | g << 8 | b
    }

    ///
    /// Creates a Color24 from a u32 value
    ///
    pub fn from_u32(rgb32: u32) -> Color24 {
        let red: u8 = (rgb32 >> 16) as u8;
        let green: u8 = (rgb32 >> 8) as u8;
        let blue: u8 = rgb32 as u8;

        Color24 {
            r: red,
            g: green,
            b: blue,
        }
    }
}


//
// Color Unit Tests
//
#[cfg(test)]
mod tests {

    use crate::led_strip_controller::color;

    #[test]
    fn color_to_u32_test() {
        assert_eq!(
            color::Color24 {
                r: 0x00,
                g: 0x00,
                b: 0x00
            }
            .to_u32(),
            0x00000000
        );

        assert_eq!(
            color::Color24 {
                r: 0x55,
                g: 0x55,
                b: 0x55
            }
            .to_u32(),
            0x00555555
        );

        assert_eq!(
            color::Color24 {
                r: 0xaa,
                g: 0xaa,
                b: 0xaa
            }
            .to_u32(),
            0x00aaaaaa
        );

        assert_eq!(
            color::Color24 {
                r: 0xff,
                g: 0xff,
                b: 0xff
            }
            .to_u32(),
            0x00ffffff
        );

        assert_eq!(
            color::Color24 {
                r: 0xc2,
                g: 0xb4,
                b: 0xf3
            }
            .to_u32(),
            0x00c2b4f3
        );
    }

    #[test]
    fn color_from_u32_test() {
        // 0x00000000
        let c1 = color::Color24::from_u32(0x00000000);
        let c2 = color::Color24 {
            r: 0x00,
            g: 0x00,
            b: 0x00,
        };

        assert_eq!(c1.r, c2.r);
        assert_eq!(c1.g, c2.g);
        assert_eq!(c1.b, c2.b);

        // 0x00555555
        let c1 = color::Color24::from_u32(0x00555555);
        let c2 = color::Color24 {
            r: 0x55,
            g: 0x55,
            b: 0x55,
        };

        assert_eq!(c1.r, c2.r);
        assert_eq!(c1.g, c2.g);
        assert_eq!(c1.b, c2.b);

        // 0x00aaaaaa
        let c1 = color::Color24::from_u32(0x00aaaaaa);
        let c2 = color::Color24 {
            r: 0xaa,
            g: 0xaa,
            b: 0xaa,
        };

        assert_eq!(c1.r, c2.r);
        assert_eq!(c1.g, c2.g);
        assert_eq!(c1.b, c2.b);

        // 0x00ffffff
        let c1 = color::Color24::from_u32(0x00ffffff);
        let c2 = color::Color24 {
            r: 0xff,
            g: 0xff,
            b: 0xff,
        };

        assert_eq!(c1.r, c2.r);
        assert_eq!(c1.g, c2.g);
        assert_eq!(c1.b, c2.b);

        // 0x00c2b4f3
        let c1 = color::Color24::from_u32(0x00c2b4f3);
        let c2 = color::Color24 {
            r: 0xc2,
            g: 0xb4,
            b: 0xf3,
        };

        assert_eq!(c1.r, c2.r);
        assert_eq!(c1.g, c2.g);
        assert_eq!(c1.b, c2.b);
    }
}
