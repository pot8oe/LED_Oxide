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
use led_oxide::led_strip_controller::protocol::ResponsePacketOption::{ Success, FailedRemote, FailedLocal };
use chrono::{DateTime, Utc};
use serde::Serialize;
use rocket::request::Form;
use rocket::Data;
use rocket::Request;
use rocket_contrib::json::Json;
use rocket_contrib::serve::StaticFiles;
use std::fs::File;
use std::io::Read;
use std::io::Write;

const MAX_FW_UPLOAD_SIZE: u64 = 524288;

const ERR_STR_FAIL_TO_FIND_HW: &str = "Failed to find LEDSC Hardware";

///
/// Simple command response data structure. Used as return value for basic commands:
/// set brightness, effect, color, etc...
///
#[derive(Serialize)]
struct SimpleCmdResponse {
    success: bool,
    status_str: String,
}

///
/// Used as the response when getting device status.
/// 
#[derive(Serialize)]
struct LedStatusResponse {
    success: bool,
    status_str: String,
    brightness_percent: f32,
    effect_id: u8,
    color: String,
    fire_pallet_id: u8,
    hw_debug: bool,
}

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
fn set_brightness(brightness_data: Form<FormDataBrightness>) -> Json<SimpleCmdResponse> {

    let status: String;

    match controller::auto_detect_ledsc() {
        Ok(port_info) => {
            let brightness: u8 = ((brightness_data.brightness_percent / 100.00) * 255.00) as u8;

            let protocol_instance = LedscTeensy001 {};
            let cmd = protocol_instance.create_cmd_string(Command::SetBrightness(brightness));

            match controller::send_command_wait_for_response(&port_info, cmd) {
                Ok(_rsp_pkt) => {
                    status = String::from("Set Brightness");
                    println!("{}", status);
                    return Json(SimpleCmdResponse { success: true, status_str: status});
                }
                Err(rsp_pkt) => {
                    status = String::from(format!("Failed to set brightness - {:?}", rsp_pkt));
                    println!("{}", status);
                    return Json(SimpleCmdResponse { success: false, status_str: status});
                }
            }
        }
        Err(_e) => {
            status = String::from(ERR_STR_FAIL_TO_FIND_HW);
            println!("{}", status);
            return Json(SimpleCmdResponse { success: false, status_str: status});
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
fn set_effect(effect_data: Form<FormDataEffect>) -> Json<SimpleCmdResponse> {

    let status: String;

    match controller::auto_detect_ledsc() {
        Ok(port_info) => {
            let protocol_instance = LedscTeensy001 {};
            let cmd = protocol_instance.create_cmd_string(Command::SetEffect(
                protocol_instance.get_effect_from_cmd_value(&effect_data.effect_id),
            ));

            match controller::send_command_wait_for_response(&port_info, cmd) {
                Ok(_rsp_pkt) => {
                    status = String::from("Set Effect");
                    println!("{}", status);
                    return Json(SimpleCmdResponse { success: true, status_str: status});
                }
                Err(rsp_pkt) => {
                    status = String::from(format!("Failed to set effect - {:?}", rsp_pkt));
                    println!("{}", status);
                    return Json(SimpleCmdResponse { success: false, status_str: status});
                }
            }
        }
        Err(_e) => {
            status = String::from(ERR_STR_FAIL_TO_FIND_HW);
            println!("{}", status);
            return Json(SimpleCmdResponse { success: false, status_str: status});
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
fn set_color(color_data: Form<FormDataColor>) -> Json<SimpleCmdResponse> {

    let status: String;
    
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
                            status = String::from("Set Color");
                            println!("{}", status);
                            return Json(SimpleCmdResponse { success: true, status_str: status});
                        }
                        Err(rsp_pkt) => {
                            status = String::from(format!("Failed to set color - {:?}", rsp_pkt));
                            println!("{}", status);
                            return Json(SimpleCmdResponse { success: false, status_str: status});
                        }
                    }
                }
                Err(e) => {
                    status = String::from(format!("Failed to parse color parameter: {} - {}", color_data.color, e));
                    println!("{}", status);
                    return Json(SimpleCmdResponse { success: false, status_str: status});
                }
            }
        }
        Err(_e) => {
            status = String::from(ERR_STR_FAIL_TO_FIND_HW);
            println!("{}", status);
            return Json(SimpleCmdResponse { success: false, status_str: status});
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
fn set_fire_color_pallet(fire_pallet_data: Form<FormDataFirePallet>) -> Json<SimpleCmdResponse> {

    let status: String;
    
    match controller::auto_detect_ledsc() {
        Ok(port_info) => {
            let protocol_instance = LedscTeensy001 {};
            let cmd = protocol_instance.create_cmd_string(Command::SetFireColorPallet(
                protocol_instance.get_fire_color_pallet_from_cmd_value(&fire_pallet_data.pallet_id),
            ));

            match controller::send_command_wait_for_response(&port_info, cmd) {
                Ok(_rsp_pkt) => {
                    status = String::from("Set Color Fire Pallet");
                    println!("{}", status);
                    return Json(SimpleCmdResponse { success: true, status_str: status});
                }
                Err(rsp_pkt) => {
                    status = String::from(format!("Failed to set color fire pallet - {:?}", rsp_pkt));
                    println!("{}", status);
                    return Json(SimpleCmdResponse { success: false, status_str: status});
                }
            }
        }
        Err(_e) => {
            status = String::from(ERR_STR_FAIL_TO_FIND_HW);
            println!("{}", status);
            return Json(SimpleCmdResponse { success: false, status_str: status});
        }
    };
}

///
/// Gets the device status & state
///
#[get("/status")]
fn get_device_status() -> Json<LedStatusResponse> {

    let status: String;

    match controller::auto_detect_ledsc() {
        Ok(port_info) => {
            //let brightness: u8 = ((brightness_data.brightness_percent / 100.00) * 255.00) as u8;

            let protocol_instance = LedscTeensy001 {};
            let cmd = protocol_instance.create_cmd_string(Command::GetStatus);

            match controller::send_command_wait_for_response(&port_info, cmd) {
                Ok(rsp_pkt) => {
                
                    match protocol_instance.parse_response_sting(rsp_pkt) {
                        Success(pkt) => {
                        
                        let status_packed: &String = &pkt.parameters[1];
                        let split = status_packed.split('|');
                        let mut led_status = LedStatusResponse {
                            success: true,
                            status_str: String::from(status_packed),
                            brightness_percent: 0.0,
                            effect_id: 0,
                            color: String::from("#000000"),
                            fire_pallet_id: 0,
                            hw_debug: false,
                        };
                        
                        let mut count = 0;
                        
                        for val in split {
                        
                            if count == 0 {
                                // Debug enabled
                                led_status.hw_debug = match u8::from_str_radix(val, 16) {
                                    Ok(dbg) => dbg != 0,
                                    Err(_) => false,
                                };
                            } else if count == 1 {
                                // Active Effect ID
                                led_status.effect_id = match u8::from_str_radix(val, 16) {
                                    Ok(id) => id,
                                    Err(_) => 0,
                                };
                            } else if count == 2 {
                                // Brightness percent
                                led_status.brightness_percent = match u8::from_str_radix(val, 16) {
                                    Ok(b) => b as f32 / 255.0,
                                    Err(_) => 0.0,
                                };
                            } else if count == 3 {
                                // Color RGB
                                led_status.color = String::from(val);
                            } else if count == 4 {
                                // Fire Color Pallet ID
                                led_status.fire_pallet_id = match u8::from_str_radix(val, 16) {
                                    Ok(id) => id,
                                    Err(_) => 0,
                                };
                            }
                            
                            count+=1;
                        }
                        
                        status = String::from("Status Read");
                            println!("{}", status);
                            return Json(led_status);
                        }
                        FailedRemote(pkt) => {
                        status = String::from(format!("Get Status hardware reported error - {:?}", pkt));
                            println!("{}", status);
                            return Json(LedStatusResponse {
                                success: false,
                                status_str: status,
                                brightness_percent: 0.0,
                                effect_id: 0,
                                color: String::from("#000000"),
                                fire_pallet_id: 0,
                                hw_debug: false,
                            });
                        }
                        FailedLocal(errcode) => {
                        status = String::from(format!("Get Status response failed local parsing - {}", errcode));
                            println!("{}", status);
                            return Json(LedStatusResponse {
                                success: false,
                                status_str: status,
                                brightness_percent: 0.0,
                                effect_id: 0,
                                color: String::from("#000000"),
                                fire_pallet_id: 0,
                                hw_debug: false,
                            });
                        }
                    }
                }
                Err(rsp_pkt) => {
                    status = String::from(format!("Failed to get status - {:?}", rsp_pkt));
                    println!("{}", status);
                    return Json(LedStatusResponse {
                        success: false,
                        status_str: status,
                        brightness_percent: 0.0,
                        effect_id: 0,
                        color: String::from("#000000"),
                        fire_pallet_id: 0,
                        hw_debug: false,
                    });
                }
            }
        }
        Err(_e) => {
            status = String::from(ERR_STR_FAIL_TO_FIND_HW);
            println!("{}", status);
            return Json(LedStatusResponse {
                success: false,
                status_str: status,
                brightness_percent: 0.0,
                effect_id: 0,
                color: String::from("#000000"),
                fire_pallet_id: 0,
                hw_debug: false,
            });
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
                get_device_status,
                upload_fw_update,
            ],
        )
        .mount(
            "/",
            StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static")),
        )
        .register(catchers![not_found])
        .launch();
}
