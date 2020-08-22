// Copyright 2020-present Divy Srivastava and friends. All rights reserved. MIT license.

extern crate rs_lib;

// use deno_core and futures
use deno_core::plugin_api::Buf;
use deno_core::plugin_api::Interface;
use deno_core::plugin_api::Op;
use deno_core::plugin_api::ZeroCopyBuf;
use futures::future::FutureExt;
use std::path::Path;

mod structs;
mod utils;

use crate::structs::*;
use crate::utils::bind_tap;

// register all ops here
#[no_mangle]
pub fn deno_plugin_init(interface: &mut dyn Interface) {
    interface.register_op("type", op_type);
    interface.register_op("alert", op_alert);
    interface.register_op("screenSize", op_screen_size);
    interface.register_op("screenScale", op_screen_scale);
    interface.register_op("moveMouse", op_move_mouse);
    interface.register_op("quickMoveMouse", op_quick_move_mouse);
    interface.register_op("screenshot", op_screen_shot);
    interface.register_op("click", op_click);
    interface.register_op("tap", op_tap);
    interface.register_op("scroll", op_scroll);
    interface.register_op("mousePostition", op_mouse_pos);
    interface.register_op("pixelColor", op_mouse_pixel_color);
    interface.register_op("toggleKey", op_toggle_key);
    interface.register_op("pointVisible", op_point_visible);
    interface.register_op("getWindow", op_get_window);
    interface.register_op("getMonitors", op_monitor_list);
    interface.register_op("transformByIndex", op_transform_by_id);
    interface.register_op("notify", op_notify);
}

// incomplete fn to get the window name
fn op_get_window(_interface: &mut dyn Interface, zero_copy: &mut [ZeroCopyBuf]) -> Op {
    let data = &zero_copy[0][..];
    let data_str = std::str::from_utf8(&data[..]).unwrap().to_string();
    let index: usize = data_str.trim().parse().unwrap();
    let window = rs_lib::window::get_window(index);

    let response = WindowResponse { window: &window };
    let result_box: Buf = serde_json::to_vec(&response).unwrap().into_boxed_slice();

    Op::Sync(result_box)
}

// deno bindings to `type`
fn op_notify(_interface: &mut dyn Interface, zero_copy: &mut [ZeroCopyBuf]) -> Op {
    let data = &zero_copy[0][..];
    let params: NotifyParams = serde_json::from_slice(data).unwrap();
    rs_lib::notify::notify(&params.title, &params.body);
    let result = b"true";
    let result_box: Buf = Box::new(*result);
    Op::Sync(result_box)
}

// deno bindings for `type`
fn op_type(_interface: &mut dyn Interface, zero_copy: &mut [ZeroCopyBuf]) -> Op {
    let data = &zero_copy[0][..];
    // convert arg to string
    let data_str = std::str::from_utf8(&data[..]).unwrap().to_string();

    // in case, we need a optional functionality in future
    let fut = async move {
        let (tx, rx) = futures::channel::oneshot::channel::<Result<(), ()>>();
        std::thread::spawn(move || {
            // call type_string
            rs_lib::key::type_string(&data_str, &[], 200., 0.);
            std::thread::sleep(std::time::Duration::from_secs(1));
            tx.send(Ok(())).unwrap();
        });
        assert!(rx.await.is_ok());

        // return true
        let result = b"true";
        let result_box: Buf = Box::new(*result);
        result_box
    };

    Op::Async(fut.boxed())
}

fn op_screen_size(_interface: &mut dyn Interface, _zero_copy: &mut [ZeroCopyBuf]) -> Op {
    let mut response = Resp {
        width: 1000_f64,
        height: 1000_f64,
    };

    let result = rs_lib::screen::size();

    response.height = result.height;
    response.width = result.width;

    let result_box: Buf = serde_json::to_vec(&response).unwrap().into_boxed_slice();
    Op::Sync(result_box)
}

fn op_monitor_list(_interface: &mut dyn Interface, _zero_copy: &mut [ZeroCopyBuf]) -> Op {
    let no_of_monitors = rs_lib::window::get_active_monitors();

    let response = MonitorResponse {
        monitors: &no_of_monitors,
    };
    let result_box: Buf = serde_json::to_vec(&response).unwrap().into_boxed_slice();
    Op::Sync(result_box)
}

fn op_screen_scale(_interface: &mut dyn Interface, _zero_copy: &mut [ZeroCopyBuf]) -> Op {
    let mut response = ScaleResponse { scale: 1000_f64 };

    let result = rs_lib::screen::scale();

    response.scale = result;

    let result_box: Buf = serde_json::to_vec(&response).unwrap().into_boxed_slice();
    Op::Sync(result_box)
}

fn op_quick_move_mouse(_interface: &mut dyn Interface, zero_copy: &mut [ZeroCopyBuf]) -> Op {
    let data = &zero_copy[0][..];
    let params: QuickMousePostition = serde_json::from_slice(data).unwrap();

    rs_lib::mouse::move_to(rs_lib::geometry::Point::new(
        params.x as f64,
        params.y as f64,
    ))
    .expect("Unable to move mouse");

    let result = b"true";
    let result_box: Buf = Box::new(*result);
    Op::Sync(result_box)
}

fn op_move_mouse(_interface: &mut dyn Interface, zero_copy: &mut [ZeroCopyBuf]) -> Op {
    let data = &zero_copy[0][..];
    let params: MousePostition = serde_json::from_slice(data).unwrap();

    rs_lib::mouse::smooth_move(
        rs_lib::geometry::Point::new(params.x as f64, params.y as f64),
        params.d as f64,
    )
    .expect("Unable to move mouse");

    let result = b"true";
    let result_box: Buf = Box::new(*result);
    Op::Sync(result_box)
}

fn op_transform_by_id(_interface: &mut dyn Interface, zero_copy: &mut [ZeroCopyBuf]) -> Op {
    let data = &zero_copy[0][..];
    let params: TransformParams = serde_json::from_slice(data).unwrap();
    rs_lib::window::transform_by_index(params.index, params.height, params.width);
    let result = b"true";
    let result_box: Buf = Box::new(*result);
    Op::Sync(result_box)
}

fn op_screen_shot(_interface: &mut dyn Interface, zero_copy: &mut [ZeroCopyBuf]) -> Op {
    let data = &zero_copy[0][..];
    let data_str = std::str::from_utf8(&data[..]).unwrap().to_string();
    let bmp = rs_lib::bitmap::capture_screen().expect("Unable to capture screen");

    let bmp_path = Path::new(file!())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(&data_str);
    let _ = bmp
        .image
        .save(&bmp_path)
        .expect("Unable to save screenshot");
    let result = b"true";
    let result_box: Buf = Box::new(*result);
    Op::Sync(result_box)
}

// deno bindings for `alert`
fn op_alert(_interface: &mut dyn Interface, zero_copy: &mut [ZeroCopyBuf]) -> Op {
    let data = &zero_copy[0][..];
    let params: AlertOptions = serde_json::from_slice(data).unwrap();

    let _ = rs_lib::alert::alert(&params.msg, &params.title, None, None);

    let result = b"true";
    let result_box: Buf = Box::new(*result);
    Op::Sync(result_box)
}

fn op_click(_interface: &mut dyn Interface, zero_copy: &mut [ZeroCopyBuf]) -> Op {
    let data = &zero_copy[0][..];
    // convert arg to string
    let data_str = std::str::from_utf8(&data[..]).unwrap().to_string();

    if data_str == "left" {
        rs_lib::mouse::click(rs_lib::mouse::Button::Left, 10 as u64);
    }
    if data_str == "right" {
        rs_lib::mouse::click(rs_lib::mouse::Button::Right, 10 as u64);
    }
    if data_str == "middle" {
        rs_lib::mouse::click(rs_lib::mouse::Button::Middle, 10 as u64);
    }

    let result = b"true";
    let result_box: Buf = Box::new(*result);
    Op::Sync(result_box)
}

// TODO: incomplete
fn op_scroll(_interface: &mut dyn Interface, zero_copy: &mut [ZeroCopyBuf]) -> Op {
    let data = &zero_copy[0][..];
    // convert arg to string
    let _data_str = std::str::from_utf8(&data[..]).unwrap().to_string();

    rs_lib::mouse::scroll(rs_lib::mouse::ScrollDirection::Up, 5 as u32);

    let result = b"true";
    let result_box: Buf = Box::new(*result);
    Op::Sync(result_box)
}

// get mouse pixel color
fn op_mouse_pixel_color(_interface: &mut dyn Interface, _zero_copy: &mut [ZeroCopyBuf]) -> Op {
    let mut response = PixelRsp {
        r: 0x82u8,
        g: 0x82u8,
        b: 0x82u8,
        a: 0x82u8,
    };

    let result = rs_lib::screen::get_color(rs_lib::mouse::location());
    let r = result.ok().unwrap();

    response.r = r[0];
    response.g = r[1];
    response.b = r[2];
    response.a = r[3];

    let result_box: Buf = serde_json::to_vec(&response).unwrap().into_boxed_slice();
    Op::Sync(result_box)
}

// point is visible or not
fn op_point_visible(_interface: &mut dyn Interface, zero_copy: &mut [ZeroCopyBuf]) -> Op {
    let data = &zero_copy[0][..];
    let params: PointPosition = serde_json::from_slice(data).unwrap();

    let r = rs_lib::screen::is_point_visible(rs_lib::geometry::Point::new(
        params.x as f64,
        params.y as f64,
    ));
    let mut result = b"0";
    if r == true {
        result = b"1"
    };
    let result_box: Buf = Box::new(*result);
    Op::Sync(result_box)
}

// get mouse position
fn op_mouse_pos(_interface: &mut dyn Interface, _zero_copy: &mut [ZeroCopyBuf]) -> Op {
    let mut response = MouseResp {
        x: 100_f64,
        y: 100_f64,
    };

    let result = rs_lib::mouse::location();

    response.x = result.x;
    response.y = result.y;

    let result_box: Buf = serde_json::to_vec(&response).unwrap().into_boxed_slice();
    Op::Sync(result_box)
}

// toggle a key
fn op_toggle_key(_interface: &mut dyn Interface, zero_copy: &mut [ZeroCopyBuf]) -> Op {
    let data = &zero_copy[0][..];
    let params: ToggleOptions = serde_json::from_slice(data).unwrap();

    rs_lib::key::toggle(
        &rs_lib::key::Code(bind_tap(&params.key)),
        params.down != 0,
        &[],
        0. as u64,
    );

    let result = b"true";
    let result_box: Buf = Box::new(*result);
    Op::Sync(result_box)
}

// tap a key
fn op_tap(_interface: &mut dyn Interface, zero_copy: &mut [ZeroCopyBuf]) -> Op {
    let data = &zero_copy[0][..];
    // convert arg to string
    let data_str = std::str::from_utf8(&data[..]).unwrap().to_string();

    rs_lib::key::tap(
        &rs_lib::key::Code(bind_tap(&data_str)),
        &[],
        0. as u64,
        0. as u64,
    );

    let result = b"true";
    let result_box: Buf = Box::new(*result);
    Op::Sync(result_box)
}
