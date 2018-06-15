extern crate rand;
extern crate sdl2;
extern crate clap;

use std::path::Path;
use std::process;

use sdl2::event::Event;
use sdl2::image::{LoadTexture, INIT_PNG};
use sdl2::keyboard::Keycode;
use sdl2::rect::Point;
use sdl2::rect::Rect;
use std::thread;
use std::time::{Duration, Instant};
use clap::{Arg, App};

use rand::Rng;

const SPRITE_SIZE: u32 = 38;
const SPRITE_ZOOM: u32 = 4;
const SPRITE_REST_OFFSET: i32 = 4 * SPRITE_SIZE as i32;

const SPRITE_NORTH_OFFSET: i32 = 0;
const SPRITE_WEST_OFFSET: i32 = 1 * SPRITE_SIZE as i32;
const SPRITE_SOUTH_OFFSET: i32 = 2 * SPRITE_SIZE as i32;
const SPRITE_EAST_OFFSET: i32 = 3 * SPRITE_SIZE as i32;

const MOVEMENT_SPEED: f32 = 300.0;
const ANIMATION_SPEED: i32 = 100;
const FRAMES_PER_ANIM: i32 = 4;

const FACING_NORTH: u32 = 0;
const FACING_WEST: u32 = 1;
const FACING_SOUTH: u32 = 2;
const FACING_EAST: u32 = 3;

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
                               .help("Sets the horizontal resolution.")
                               .index(1))
                          .arg(Arg::with_name("HEIGHT")
                               .help("Sets the vertical resolution.")
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
    let _image_context = sdl2::image::init(INIT_PNG).unwrap();
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
    let player_texture = texture_creator
        .load_texture(Path::new("assets/bunny.png"))
        .unwrap();
    let mut background_texture = texture_creator
        .load_texture(Path::new("assets/background.png"))
        .unwrap();
    background_texture.set_alpha_mod(220);
    let carrot_texture = texture_creator
        .load_texture(Path::new("assets/carrot.png"))
        .unwrap();

    let mut player_x: f32 = window_width as f32 / 2.0;
    let mut player_y: f32 = window_height as f32 / 2.0;

    let mut d_player_x = 0.0;
    let mut d_player_y = 0.0;
    let mut player_direction = FACING_EAST;

    let mut source_rect = Rect::new(
        0,
        SPRITE_REST_OFFSET + (SPRITE_SIZE * player_direction) as i32,
        SPRITE_SIZE,
        SPRITE_SIZE,
    );
    let mut dest_rect = Rect::from_center(
        Point::new(player_x as i32, player_y as i32),
        SPRITE_SIZE * SPRITE_ZOOM,
        SPRITE_SIZE * SPRITE_ZOOM,
    );

    // Generate a random carrot
    let carrot_x: i32 = rand::thread_rng().gen_range(64,
                                                     window_width as i32 - 64);
    let carrot_y: i32 = rand::thread_rng().gen_range(64,
                                                     window_height as i32 - 64);

    let carrot_rect = Rect::from_center(Point::new(carrot_x, carrot_y), 64, 64);

    // Variables for calculating framerate
    let mut last_frame_end_time = Instant::now();
    let mut current_fps = 0;
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
                            source_rect.set_y(SPRITE_WEST_OFFSET);
                            d_player_x = -MOVEMENT_SPEED;
                            player_direction = FACING_WEST;
                        }
                        // RIGHT pressed
                        else if val == 32767 {
                            source_rect.set_y(SPRITE_EAST_OFFSET);
                            d_player_x = MOVEMENT_SPEED;
                            player_direction = FACING_EAST;
                        }
                        // LEFT and RIGHT released
                        else if val > -dead_zone && val < dead_zone {
                            d_player_x = 0.0;
                        }
                    }

                    if axis_idx == 1 {
                        // UP pressed
                        if val == -32768 {
                            source_rect.set_y(SPRITE_NORTH_OFFSET);
                            d_player_y = -MOVEMENT_SPEED;
                            player_direction = FACING_NORTH;
                        }
                        // DOWN pressed
                        else if val == 32767 {
                            source_rect.set_y(SPRITE_SOUTH_OFFSET);
                            d_player_y = MOVEMENT_SPEED;
                            player_direction = FACING_SOUTH;
                        }
                        // UP and DOWN released
                        else if val > -dead_zone && val < dead_zone {
                            d_player_y = 0.0;
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

        // Move player
        let dt_for_frame = 1.0 / TARGET_FRAME_RATE as f32;

        let new_player_x = player_x + (dt_for_frame * d_player_x);

        if new_player_x >= 32.0 && new_player_x <= window_width as f32 - 32.0 {
            player_x = new_player_x;
        } else {
            d_player_x = 0.0;
            source_rect.set_x(0);
        }

        let new_player_y = player_y + (dt_for_frame * d_player_y);
        if new_player_y >= 42.0 && new_player_y <= window_height as f32 - 42.0 {
            player_y = new_player_y;
        } else {
            d_player_y = 0.0;
            source_rect.set_x(0);
        }

        dest_rect.center_on(Point::new(player_x as i32, player_y as i32));

        // Walking / eating animation
        let ticks = timer.ticks() as i32;

        if !(d_player_x == 0.0) || !(d_player_y == 0.0) {
            source_rect.set_x(SPRITE_SIZE as i32 *
                ((ticks / ANIMATION_SPEED) % FRAMES_PER_ANIM));
        } else {
            // Stop animation
            source_rect.set_x(0);
            source_rect.set_y(SPRITE_REST_OFFSET +
                (SPRITE_SIZE * player_direction) as i32);
        }

        canvas.clear();

        // Copy the background frame to the canvas
        canvas.copy(&background_texture, None, None).unwrap();

        // Copy the carrot to the canvas
        canvas.copy(&carrot_texture, None, carrot_rect).unwrap();

        // Copy the player frame to the canvas
        canvas
            .copy(&player_texture, Some(source_rect), Some(dest_rect))
            .unwrap();

        canvas.present();

        // Calculate framerate.
        // NOTE(m): Borrowed heavily from Casey Muratori's Handmade Hero
        // implementation.
        let frame_end_time = Instant::now();
        if (frame_end_time - last_frame_end_time) >= Duration::new(1, 0) {
            last_frame_end_time = frame_end_time;
            current_fps = frames_elapsed;
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
