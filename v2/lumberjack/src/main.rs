use image::{DynamicImage, ImageBuffer, GenericImageView, Pixel, RgbaImage};
use xcap::Monitor;
use std::result::Result;
use std::process::Command;
use serde_json::Value;
use std::error::Error;
use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};
use opencv::prelude::*;
use opencv::imgcodecs;
use opencv::imgproc;
use opencv::core::{self, Mat, Point, Mat_AUTO_STEP, Vec3b};
use rdev::{simulate, Button, Event, EventType, Key, SimulateError};
use std::{thread, time};

fn dynamic_image_to_mat(img: &DynamicImage) -> Result<Mat, opencv::Error> {
    // Convert the DynamicImage to RGB8
    let rgb_image = img.to_rgb8();
    let (width, height) = rgb_image.dimensions();
    let mut mat = Mat::new_rows_cols_with_default(height as i32, width as i32, opencv::core::CV_8UC3, opencv::core::Scalar::default())?;
    
    // Iterate over the pixels and copy them to the Mat
    for y in 0..height {
        for x in 0..width {
            let pixel = rgb_image.get_pixel(x, y);
            let data = pixel.0;
            *mat.at_2d_mut::<Vec3b>(y as i32, x as i32)? = Vec3b::from(data);
        }
    }
    Ok(mat)
}

fn take_screenshot(x: u32, y: u32, width: u32, height: u32) -> Result<Mat, Box<dyn Error>> {
    let resolution_adjust = 2 as u32;
    let monitors = Monitor::all()?;
    let primary_monitor = &monitors[0];  // Assuming the first monitor is the primary one
    let frame = primary_monitor.capture_image()?;
    let frame_width = frame.width();
    let frame_height = frame.height();
    let image = RgbaImage::from_raw(frame_width, frame_height, frame.into_raw())
        .ok_or("Failed to create image buffer")?;
    let cropped_image = DynamicImage::ImageRgba8(image).crop_imm(x * resolution_adjust, y * resolution_adjust, width * resolution_adjust, height * resolution_adjust);
    // cropped_image.save("screenshot.png")?;
    Ok(dynamic_image_to_mat(&cropped_image)?)
}

fn get_telegram_window_bounds() -> Option<(u32, u32, u32, u32)> {
    // List all running applications
    let output = Command::new("osascript")
        .arg("-e")
        .arg("tell application \"System Events\" to get the name of every process whose background only is false")
        .output()
        .expect("Failed to execute AppleScript");

    // debug output
    let app_list = String::from_utf8_lossy(&output.stdout);

    if app_list.contains("Telegram") {
        println!("Telegram is running");
        let script = r#"
        tell application "System Events"
        set appName to "Telegram"
        try
            -- Get screen dimensions
            tell application "Finder"
                set screenResolution to bounds of window of desktop
            end tell
            
            tell application process appName
                set theWindows to windows
                if (count of theWindows) is greater than 0 then
                    set frontWindow to item 1 of theWindows
                    set windowPosition to position of frontWindow
                    set windowSize to size of frontWindow
                    set windowBounds to {windowPosition, windowSize}
                    set resultJSON to "{\"appName\":\"" & appName & "\", \"windowPosition\":[" & (item 1 of windowPosition) & "," & (item 2 of windowPosition) & "], \"windowSize\":[" & (item 1 of windowSize) & "," & (item 2 of windowSize) & "], \"screenResolution\":[" & (item 3 of screenResolution) & "," & (item 4 of screenResolution) & "]}"
                    do shell script "echo " & quoted form of resultJSON
                else
                    set resultJSON to "{\"error\":\"No windows found for " & appName & "\"}"
                    do shell script "echo " & quoted form of resultJSON
                end if
            end tell
        on error errMsg number errNum
            set resultJSON to "{\"error\":\"" & errMsg & "\", \"errorNumber\":" & errNum & "}"
            do shell script "echo " & quoted form of resultJSON
        end try
    end tell"#;
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .expect("Failed to execute AppleScript");
        let output_json = String::from_utf8_lossy(&output.stdout);
        let json: Value = serde_json::from_str(&output_json).expect("Failed to parse JSON");
        let window_position = json["windowPosition"].as_array().unwrap();
        let window_size = json["windowSize"].as_array().unwrap();
        let screen_resolution = json["screenResolution"].as_array().unwrap();
        println!("Window Position: {:?}", window_position);
        println!("Window Size: {:?}", window_size);
        println!("Screen Resolution: {:?}", screen_resolution);

        let x = window_position[0].as_u64().unwrap() as u32;
        let y = window_position[1].as_u64().unwrap() as u32;
        let width = window_size[0].as_u64().unwrap() as u32;
        let height = window_size[1].as_u64().unwrap() as u32;

        println!("Bounds: x={}, y={}, width={}, height={}", x, y, width, height);

        return Some((x, y, width, height));
    } else {
        println!("Telegram is not running");
        return None;
    }
}


fn load_image(path: &str) -> Result<Mat, opencv::Error> {
    let img = imgcodecs::imread(path, imgcodecs::IMREAD_COLOR)?;
    Ok(img)
}

fn find_subimage_location(main_image: &Mat, sub_image: &Mat, threshold: f64) -> Result<Option<Point>, opencv::Error> {
    let mut result = Mat::default();
    let match_method = imgproc::TM_CCOEFF_NORMED;

    imgproc::match_template(main_image, sub_image, &mut result, match_method, &core::no_array())?;
    let mut min_val = 0.0;
    let mut max_val = 0.0;
    let mut min_loc = core::Point::new(0, 0);
    let mut max_loc = core::Point::new(0, 0);

    core::min_max_loc(
        &result,
        Some(&mut min_val),
        Some(&mut max_val),
        Some(&mut min_loc),
        Some(&mut max_loc),
        &core::no_array(),
    )?;

    if max_val >= threshold {
        Ok(Some(max_loc))
    } else {
        Ok(None)
    }
}

fn send(event_type: &EventType) {
    let delay = time::Duration::from_millis(25);
    match simulate(event_type) {
        Ok(()) => (),
        Err(SimulateError) => {
            println!("We could not send {:?}", event_type);
        }
    }
    // Let ths OS catchup (at least MacOS)
    thread::sleep(delay);
}

fn move_and_click(screen: &Mat, btn_path: &str, screen_x: u32, screen_y: u32) -> Result<(), Box<dyn Error>> {
    // Load images
    let btn = load_image(btn_path)?;
    let btn_width = btn.cols() as u32 / 2;
    let btn_height = btn.rows() as u32 / 2;
    
    // Set matching threshold
    let threshold = 0.6;

    // Find button location
    let location = match find_subimage_location(&screen, &btn, threshold)? {
        Some(location) => location,
        None => {
            // throw error
            return Err("Button not found".into());
        }
    };

    // Calculate screen coordinates
    let x = ((screen_x + (location.x as f64 / 2.0) as u32) + btn_width / 2 ) as f64;
    let y = ((screen_y + (location.y as f64 / 2.0) as u32) + btn_height / 2 ) as f64;

    println!("Button found at: x={}, y={}", x, y);

    // Move mouse to the button location
    simulate(&EventType::MouseMove { x, y })?;

    // Simulate a mouse click
    let delay: Duration = time::Duration::from_millis(100);
    send(&EventType::ButtonPress(rdev::Button::Left));
    send(&EventType::ButtonRelease(rdev::Button::Left));
    thread::sleep(delay);
    send(&EventType::ButtonPress(rdev::Button::Left));
    send(&EventType::ButtonRelease(rdev::Button::Left));
    thread::sleep(delay);
    send(&EventType::ButtonPress(rdev::Button::Left));
    send(&EventType::ButtonRelease(rdev::Button::Left));

    Ok(())
}

fn double_press(event_type_1: Key, event_type_2: Key) {
    send(&EventType::KeyPress((event_type_1)));
    send(&EventType::KeyRelease(event_type_1));
    send(&EventType::KeyPress((event_type_1)));
    send(&EventType::KeyRelease(event_type_1));
    send(&EventType::KeyPress(event_type_2));
    send(&EventType::KeyRelease(event_type_2));
    send(&EventType::KeyPress(event_type_2));
    send(&EventType::KeyRelease(event_type_2));
}

fn detect_tree_and_tap(
    branches: &Vec<Mat>,
    left_branch: &Mat,
    right_branch: &Mat,
    screen: &Mat,
) -> Result<(), Box<dyn Error>> {
    let threshold = 0.65;
    let branch_1 = &branches[0];
    match find_subimage_location(&screen, &branch_1, threshold)? {
        Some(location) => {
            println!("[LEFT LEFT]");
            double_press(Key::LeftArrow, Key::LeftArrow);
            return Ok(());
        }, 
        None => {}
    };
    let branch_2 = &branches[1];
    match find_subimage_location(&screen, &branch_2, threshold)? {
        Some(location) => {
            println!("[LEFT RIGHT]");
            double_press(Key::LeftArrow, Key::RightArrow);
            return Ok(());
        }, 
        None => {}
    };
    let branch_3 = &branches[2];
    match find_subimage_location(&screen, &branch_3, threshold)? {
        Some(location) => {
            println!("[RIGHT RIGHT]");
            double_press(Key::RightArrow, Key::RightArrow);
            return Ok(());
        }, 
        None => {}
    };
    let branch_4 = &branches[3];
    match find_subimage_location(&screen, &branch_4, threshold)? {
        Some(location) => {
            println!("[RIGHT LEFT]");
            double_press(Key::RightArrow, Key::LeftArrow);
            return Ok(());
        }, 
        None => {}
    };

    let left_branch_location = match find_subimage_location(&screen, &left_branch, threshold)? {
        Some(location) => Some(location),
        None => None
    };
    let right_branch_location = match find_subimage_location(&screen, &right_branch, threshold)? {
        Some(location) => Some(location),
        None => None
    };

    if let (Some(left_loc), Some(right_loc)) = (left_branch_location, right_branch_location) {
        if left_loc.y > right_loc.y {
            println!("[RIGHT 1]");
            send(&EventType::KeyPress(Key::RightArrow));
            send(&EventType::KeyRelease(Key::RightArrow));
            send(&EventType::KeyPress(Key::RightArrow));
            send(&EventType::KeyRelease(Key::RightArrow));
        } else {
            println!("[LEFT 1]");
            send(&EventType::KeyPress(Key::LeftArrow));
            send(&EventType::KeyRelease(Key::LeftArrow));
            send(&EventType::KeyPress(Key::LeftArrow));
            send(&EventType::KeyRelease(Key::LeftArrow));
        }
    } else if let Some(_) = left_branch_location {
        println!("[RIGHT 2]");
        send(&EventType::KeyPress(Key::RightArrow));
        send(&EventType::KeyRelease(Key::RightArrow));
        send(&EventType::KeyPress(Key::RightArrow));
        send(&EventType::KeyRelease(Key::RightArrow));
    } else if let Some(_) = right_branch_location {
        println!("[LEFT 2]");
        send(&EventType::KeyPress(Key::LeftArrow));
        send(&EventType::KeyRelease(Key::LeftArrow));
        send(&EventType::KeyPress(Key::LeftArrow));
        send(&EventType::KeyRelease(Key::LeftArrow));
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {

    let (x, y, width, height) = match get_telegram_window_bounds() {
        Some(bounds) => bounds,
        None => panic!("Telegram is not running!"),
    };
    let left_branch= load_image("lumber/left_branch.png")?;
    let right_branch = load_image("lumber/right_branch.png")?;
    
    let branch_1 = load_image("lumber/branch_1.png")?;
    let branch_2 = load_image("lumber/branch_2.png")?;
    let branch_3 = load_image("lumber/branch_3.png")?;
    let branch_4 = load_image("lumber/branch_4.png")?;

    let branches = vec![branch_1, branch_2, branch_3, branch_4];

    let delay = time::Duration::from_millis(1000);
    let mut ss = take_screenshot(x, y, width, height)?;
    match move_and_click(&ss, "lumber/restart_button.png", x, y) {
        Ok(_) => (),
        Err(_) => {
            println!("Restart button not found");
            move_and_click(&ss, "lumber/play_button.png", x, y)?;
            thread::sleep(delay);
            ss = take_screenshot(x, y, width, height)?;
            move_and_click(&ss, "lumber/start_button.png", x, y)?;
        }
    }

    thread::sleep(time::Duration::from_millis(100));
    send(&EventType::KeyPress(Key::LeftArrow));
    send(&EventType::KeyPress(Key::LeftArrow));

    // Detect Image and Execute
    // while true
    loop {
        let start = Instant::now();
        ss = take_screenshot(x, y, width, height)?;
        detect_tree_and_tap(&branches, &left_branch, &right_branch, &ss)?;
        let end = Instant::now();
        let duration = end.duration_since(start);
        println!("Time taken: {:?}", duration);
    }

    // End
    Ok(())
}