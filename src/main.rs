extern crate rand;
extern crate sdl2;
extern crate clap;

use sdl2::event::Event;
use sdl2::render::TextureAccess;
use sdl2::keyboard::Keycode;
use std::thread;
use std::time::{Duration, Instant};
use clap::{Arg, App};


// FPS and frame capping constants
const TARGET_FRAME_RATE: u64 = 60;
const BILLION: u64 = 1_000_000_000;
const FRAME_TIME_NS: u64 = BILLION / TARGET_FRAME_RATE;

fn main() {
    // Parse command line options for width, height and fullscreen toggle.
    let matches = App::new("A.L.I.C.E.")
                          .version("1.0")
                          .author("Michael Smith <m@michaelsmith.be>")
                          .arg(Arg::with_name("WIDTH")
                               .help("Specify the horizontal resolution.")
                               .index(1))
                          .arg(Arg::with_name("HEIGHT")
                               .help("Specify the vertical resolution.")
                               .index(2))
                          .arg(Arg::with_name("f")
                               .short("f")
                               .help("Run in fullscreen."))
                          .get_matches();

    let window_fullscreen = matches.is_present("f");

    let window_width = if window_fullscreen {
        match matches.value_of("WIDTH")
                     .unwrap_or("1920")
                     .parse::<u32>() {
                        Ok(n) => n,
                        Err(_e) => 1920,
                     }
    } else {
        match matches.value_of("WIDTH")
                     .unwrap_or("960")
                     .parse::<u32>() {
                        Ok(n) => n,
                        Err(_e) => 960,
                     }

    };

    let window_height = if window_fullscreen {
        match matches.value_of("HEIGHT")
                     .unwrap_or("1080")
                     .parse::<u32>() {
                        Ok(n) => n,
                        Err(_e) => 1080,
                     }
    } else {
        match matches.value_of("HEIGHT")
                     .unwrap_or("540")
                     .parse::<u32>() {
                        Ok(n) => n,
                        Err(_e) => 540,
                     }

    };

    // Initialize SDL
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let joystick_subsystem = sdl_context.joystick().unwrap();
    let mut _joystick = None;

    let window = if window_fullscreen {
        video_subsystem.window("A.L.I.C.E.", window_width, window_height)
                       .fullscreen()
                       .build()
                       .unwrap()
    } else {
        video_subsystem.window("A.L.I.C.E.", window_width, window_height)
                       .build()
                       .unwrap()
    };

    // Hide the cursor
    sdl_context.mouse().show_cursor(false);

    let mut canvas = window.into_canvas().accelerated().build().unwrap();
    let texture_creator = canvas.texture_creator();

    canvas.set_draw_color(sdl2::pixels::Color::RGBA(0, 0, 0, 200));

    let mut timer = sdl_context.timer().unwrap();

    // Load textures
    let mut texture = texture_creator.create_texture(None,
                                                     TextureAccess::Streaming,
                                                     window_width,
                                                     window_height).unwrap();

    // Variables for calculating framerate
    let mut last_frame_end_time = Instant::now();
    let mut _current_fps = 0;
    let mut frames_elapsed = 0;

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        let start_time = Instant::now();

        for event in event_pump.poll_iter() {
            match event {
                Event::JoyDeviceAdded { which, .. } => {
                    println!("Joystick {} connected.", which);
                    match joystick_subsystem.open(which) {
                        Ok(c) => {
                            println!("Success: opened \"{}\"", c.name());
                            _joystick = Some(c);
                            break;
                        }
                        Err(e) => println!("failed: {:?}", e),
                    }
                }
                Event::JoyDeviceRemoved { which, .. } => {
                    println!("Joystick {} disconnected.", which);
                    _joystick = None;
                }
                Event::JoyButtonDown { button_idx, .. } => {
                    println!("Button down {:?}", button_idx);
                }
                Event::JoyAxisMotion {
                    axis_idx,
                    value: val,
                    ..
                } => {
                    let dead_zone = 500;

                    if axis_idx == 0 {
                        // LEFT pressed
                        if val == -32768 {
                            println!("LEFT pressed");
                        }
                        // RIGHT pressed
                        else if val == 32767 {
                            println!("RIGHT pressed");
                        }
                        // LEFT and RIGHT released
                        else if val > -dead_zone && val < dead_zone {
                            println!("LEFT/RIGHT released");
                        }
                    }

                    if axis_idx == 1 {
                        // UP pressed
                        if val == -32768 {
                            println!("UP pressed");
                        }
                        // DOWN pressed
                        else if val == 32767 {
                            println!("DOWN pressed");
                        }
                        // UP and DOWN released
                        else if val > -dead_zone && val < dead_zone {
                            println!("UP/DOWN released");
                        }
                    }
                }

                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        canvas.clear();

        // Copy the texture to the canvas
        canvas.copy(&texture, None, None).unwrap();

        canvas.present();

        // Calculate framerate.
        // NOTE(m): Borrowed heavily from Casey Muratori's Handmade Hero
        // implementation.
        let frame_end_time = Instant::now();
        if (frame_end_time - last_frame_end_time) >= Duration::new(1, 0) {
            last_frame_end_time = frame_end_time;
            _current_fps = frames_elapsed;
            frames_elapsed = 0;
        }
        frames_elapsed = frames_elapsed + 1;

        // Cap framerate.
        let end_time = Instant::now();
        let time_elapsed = end_time - start_time;
        let time_elapsed: u64 =
            time_elapsed.as_secs() *
            BILLION +
            time_elapsed.subsec_nanos() as u64;
        if time_elapsed < FRAME_TIME_NS {
            thread::sleep(Duration::new(0,
                                        (FRAME_TIME_NS - time_elapsed) as u32));
        }
    }
}
