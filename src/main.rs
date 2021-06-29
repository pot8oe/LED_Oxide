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

#[macro_use] extern crate rocket;

use serde::Deserialize;
use rocket::http::Status;
use rocket::Request;
use rocket::request::Form;
use rocket::request::LenientForm;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::json::Json;
use led_oxide::led_strip_controller::controller;
use led_oxide::led_strip_controller::protocol::*;
use led_oxide::led_strip_controller::color::*;
use led_oxide::led_strip_controller::controller::*;




#[catch(404)]
fn not_found(req: &Request) -> String {
    format!("Sorry, '{}' is not a valid path.", req.uri())
}

#[get("/")]
fn index() -> &'static str {
    "Welcome to LED Oxide!"
}

#[derive(FromForm)]
struct FormDataBrightness {
    brightness_percent: f32
}

#[post("/brightness", data = "<brightness_data>")]
fn set_brightness(brightness_data: Form<FormDataBrightness>) -> rocket::http::Status {

    match controller::auto_detect_ledsc() {
        Ok(port_info) => {

            let brightness: u8 = ((brightness_data.brightness_percent / 100.00) * 255.00) as u8;

            let protocol_instance = LedscTeensy001 {};
            let cmd = protocol_instance.create_cmd_string(
                Command::SetBrightness(brightness)
            );

            match controller::send_command_wait_for_response(&port_info, cmd) {
                Ok(_rsp_pkt) => {
                    println!("Set Brightness");
                    return rocket::http::Status::Ok;
                },
                Err(rsp_pkt) => {
                    println!("Failed to set brightness {:?}", rsp_pkt);
                    return rocket::http::Status::InternalServerError;
                }
            }

        },
        Err(_e) => {
            println!("Failed to find LEDSC");
            return rocket::http::Status::InternalServerError;
        },
    };

    rocket::http::Status::ImATeapot
}

#[derive(FromForm)]
struct FormDataEffect {
    effect_id: u8
}

#[post("/effect", data = "<effect_data>")]
fn set_effect(effect_data: Form<FormDataEffect>) -> rocket::http::Status {

    match controller::auto_detect_ledsc() {
        Ok(port_info) => {

            let protocol_instance = LedscTeensy001 {};
            let cmd = protocol_instance.create_cmd_string(
                Command::SetEffect(protocol_instance.get_effect_from_cmd_value(&effect_data.effect_id))
            );

            match controller::send_command_wait_for_response(&port_info, cmd) {
                Ok(_rsp_pkt) => {
                    println!("Set Effect - Solid color");
                    return rocket::http::Status::Ok;
                },
                Err(rsp_pkt) => {
                    println!("Failed to set effect {:?}", rsp_pkt);
                    return rocket::http::Status::InternalServerError;
                }
            }

        },
        Err(_e) => {
            println!("Failed to find LEDSC");
            return rocket::http::Status::InternalServerError;
        },
    };

    rocket::http::Status::ImATeapot
}

#[derive(FromForm)]
struct FormDataColor {
    color: String
}

#[post("/color", data = "<color_data>")]
fn set_color(color_data: Form<FormDataColor>) -> String {

    match controller::auto_detect_ledsc() {
        Ok(port_info) => {

            let color_result = u32::from_str_radix(color_data.color.as_str().trim_matches('#'), 16);

            match color_result {
                Ok(color_int) => {
                    let protocol_instance = LedscTeensy001 {};
                    let cmd = protocol_instance.create_cmd_string(
                        Command::SetColor(Color24::from_u32(color_int))
                    );

                    match controller::send_command_wait_for_response(&port_info, cmd) {
                        Ok(_rsp_pkt) => {
                            println!("Set Color");
                            return String::from(rocket::http::Status::Ok.reason);
                        },
                        Err(rsp_pkt) => {
                            println!("Failed to set color {:?}", rsp_pkt);
                            //return rocket::http::Status::InternalServerError;
                            return String::from("Failed to set color");
                        }
                    }
                },
                Err(e) => {
                    println!("Failed to parse color parameter.");
                    //return rocket::http::Status::InternalServerError.reason;
                    return format!("Failed to parse color parameter: {} - {}", color_data.color, e);
                }
            }
        },
        Err(_e) => {
            println!("Failed to find LEDSC");
            //return rocket::http::Status::InternalServerError;
            return String::from("Failed to find LEDSC Hardware");
        },
    };

    String::from(rocket::http::Status::ImATeapot.reason)
}

fn main() {
    rocket::ignite()
        .mount("/",
            routes![
                //index,
                set_brightness,
                set_effect,
                set_color
                ]
        )
        .mount("/", StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static")))
        .register(catchers![not_found])
        .launch();
}
