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

#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use led_oxide::led_strip_controller::color::*;
use led_oxide::led_strip_controller::controller;
use led_oxide::led_strip_controller::protocol::*;
use chrono::{DateTime, Utc};
use rocket::request::Form;
use rocket::Data;
use rocket::Request;
use rocket_contrib::serve::StaticFiles;
use std::fs::File;
use std::io::Read;
use std::io::Write;

const MAX_FW_UPLOAD_SIZE: u64 = 524288;

///
/// Error 404 endpoint
///
#[catch(404)]
fn not_found(req: &Request) -> String {
    format!("Sorry, '{}' is not a valid path.", req.uri())
}

///
/// Welcome endpoint
///
#[get("/")]
fn index() -> &'static str {
    "Welcome to LED Oxide!"
}

///
/// Set brightness endpoint data
///
#[derive(FromForm)]
struct FormDataBrightness {
    brightness_percent: f32,
}

///
/// Set brightness endpoint
///
#[post("/brightness", data = "<brightness_data>")]
fn set_brightness(brightness_data: Form<FormDataBrightness>) -> rocket::http::Status {
    match controller::auto_detect_ledsc() {
        Ok(port_info) => {
            let brightness: u8 = ((brightness_data.brightness_percent / 100.00) * 255.00) as u8;

            let protocol_instance = LedscTeensy001 {};
            let cmd = protocol_instance.create_cmd_string(Command::SetBrightness(brightness));

            match controller::send_command_wait_for_response(&port_info, cmd) {
                Ok(_rsp_pkt) => {
                    println!("Set Brightness");
                    return rocket::http::Status::Ok;
                }
                Err(rsp_pkt) => {
                    println!("Failed to set brightness {:?}", rsp_pkt);
                    return rocket::http::Status::InternalServerError;
                }
            }
        }
        Err(_e) => {
            println!("Failed to find LEDSC");
            return rocket::http::Status::InternalServerError;
        }
    };
}

///
/// Set effect endpoint data
///
#[derive(FromForm)]
struct FormDataEffect {
    effect_id: u8,
}

///
/// Set effect endpoint
///
#[post("/effect", data = "<effect_data>")]
fn set_effect(effect_data: Form<FormDataEffect>) -> rocket::http::Status {
    match controller::auto_detect_ledsc() {
        Ok(port_info) => {
            let protocol_instance = LedscTeensy001 {};
            let cmd = protocol_instance.create_cmd_string(Command::SetEffect(
                protocol_instance.get_effect_from_cmd_value(&effect_data.effect_id),
            ));

            match controller::send_command_wait_for_response(&port_info, cmd) {
                Ok(_rsp_pkt) => {
                    println!("Set Effect - Solid color");
                    return rocket::http::Status::Ok;
                }
                Err(rsp_pkt) => {
                    println!("Failed to set effect {:?}", rsp_pkt);
                    return rocket::http::Status::InternalServerError;
                }
            }
        }
        Err(_e) => {
            println!("Failed to find LEDSC");
            return rocket::http::Status::InternalServerError;
        }
    };
}

///
/// Set color endpoint data
///
#[derive(FromForm)]
struct FormDataColor {
    color: String,
}

///
/// Set color endpoint
///
#[post("/color", data = "<color_data>")]
fn set_color(color_data: Form<FormDataColor>) -> String {
    match controller::auto_detect_ledsc() {
        Ok(port_info) => {
            let color_result = u32::from_str_radix(color_data.color.as_str().trim_matches('#'), 16);

            match color_result {
                Ok(color_int) => {
                    let protocol_instance = LedscTeensy001 {};
                    let cmd = protocol_instance
                        .create_cmd_string(Command::SetColor(Color24::from_u32(color_int)));

                    match controller::send_command_wait_for_response(&port_info, cmd) {
                        Ok(_rsp_pkt) => {
                            println!("Set Color");
                            return String::from(rocket::http::Status::Ok.reason);
                        }
                        Err(rsp_pkt) => {
                            println!("Failed to set color {:?}", rsp_pkt);
                            //return rocket::http::Status::InternalServerError;
                            return String::from("Failed to set color");
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to parse color parameter.");
                    //return rocket::http::Status::InternalServerError.reason;
                    return format!(
                        "Failed to parse color parameter: {} - {}",
                        color_data.color, e
                    );
                }
            }
        }
        Err(_e) => {
            println!("Failed to find LEDSC");
            //return rocket::http::Status::InternalServerError;
            return String::from("Failed to find LEDSC Hardware");
        }
    };
}

///
/// Set the Firepalle endpoint data
///
#[derive(FromForm)]
struct FormDataFirePallet {
    pallet_id: u8,
}

///
/// Set the Firepalle endpoint
///
#[post("/firepallet", data = "<fire_pallet_data>")]
fn set_fire_color_pallet(fire_pallet_data: Form<FormDataFirePallet>) -> rocket::http::Status {
    match controller::auto_detect_ledsc() {
        Ok(port_info) => {
            let protocol_instance = LedscTeensy001 {};
            let cmd = protocol_instance.create_cmd_string(Command::SetFireColorPallet(
                protocol_instance.get_fire_color_pallet_from_cmd_value(&fire_pallet_data.pallet_id),
            ));

            match controller::send_command_wait_for_response(&port_info, cmd) {
                Ok(rsp_pkt) => {
                    println!("Set Effect {:?}", rsp_pkt);
                    return rocket::http::Status::Ok;
                }
                Err(rsp_pkt) => {
                    println!("Failed to set effect {:?}", rsp_pkt);
                    return rocket::http::Status::InternalServerError;
                }
            }
        }
        Err(_e) => {
            println!("Failed to find LEDSC");
            return rocket::http::Status::InternalServerError;
        }
    };
}

///
/// Upload fw update endpoint
///
#[post("/upload_fw_update", format = "plain", data = "<data>")]
fn upload_fw_update(data: Data) -> Result<String, std::io::Error> {

    let mut stream = data.open().take(MAX_FW_UPLOAD_SIZE);
    let mut stream_buffer: Vec<u8> = vec![];
    match stream.read_to_end(&mut stream_buffer) {
            Ok(_) => {

                let now: DateTime<Utc> = Utc::now();
                let tmp_file_name = format!("/tmp/fw_teensy_{}.hex", now.format("%Y%m%d%H%M%S%f"));
                let mut file = File::create(tmp_file_name)?;

                match file.write_all(stream_buffer.as_slice()) {
                        Ok(_) => { Ok("Success".to_string()) },
                        Err(e) => Err(e)
                    }
            },
            Err(e) => Err(e)
        }
}

///
/// Main Application Entry
///
fn main() {
    rocket::ignite()
        .mount(
            "/",
            routes![
                //index,
                set_brightness,
                set_effect,
                set_color,
                set_fire_color_pallet,
                upload_fw_update
            ],
        )
        .mount(
            "/",
            StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static")),
        )
        .register(catchers![not_found])
        .launch();
}
