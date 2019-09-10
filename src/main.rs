extern crate serde_derive;
extern crate rand;
use termcolor::{Color, Ansi, ColorChoice, ColorSpec, StandardStream, WriteColor};
use std::io::Write;
use std::option;
use rand::Rng;
use std::net::{UdpSocket, SocketAddr, IpAddr, Ipv4Addr};
use std::{thread, time};
use std::vec::Vec;
pub mod device_modules;
mod audio;
use audio::{init_audio, Vec2};

use device_modules::config::*;
use std::io;
use std::sync::mpsc::*;

mod visualizer;
use visualizer::display;


#[allow(unreachable_code)]
fn setup_device(cfg_settings:&mut Devicecfg) -> (StatusType) {
    let mut input = String::new();
    let mut cfg_complete = false;

    println!("Setup custom config: s");
    println!("Use Default settings: d");
    println!("Quit: q");
    while !cfg_complete {
        match io::stdin().read_line(&mut input) {
            Ok(_number_of_bytes) => {
                match input.trim() {
                    "s" => {unimplemented!()},
                    "d" => {
                        cfg_complete = true;
                        *cfg_settings =  Devicecfg::default();
                    }
                    _  => { 
                        panic!("Unhandled case")
                    },
                    
                };
            }
            Err(error) => println!("error: {}", error),
        }
    }
    return StatusType::ERROR;
}

fn config_mode() {
    let mut device_settings = Devicecfg::default();
    setup_device(& mut device_settings);
    println!("Setup Remote Device ? y/n");
    //TODO make this a macro
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_number_of_bytes) => {
            match input.trim() {
                "y" => {unimplemented!()},
                "n" => {
                },
                _  => { 
                    panic!("Unhandled case")
                },
                
            };
        }
        Err(error) => println!("error: {}", error),
    }

}

fn make_random_led_vec(strip_size : usize) -> Vec<Vec<u8>> {
        let mut test_leds : Vec<Vec<u8>> = Vec::with_capacity(strip_size);
        let mut rng = rand::thread_rng();

        for _ in 0..strip_size {
            let led_idx: Vec<u8> = (0..3).map(|_| {
                rng.gen_range(0,255)
                }).collect();
            test_leds.push(led_idx);
        }
        test_leds
}

struct Pixel {
    //Named with a U because 🇨🇦
    pub colour : [u8;3],
    pub stdout_symbol : String,
    colour_spec : ColorSpec,
}

// impl Pixel {
    // fn new(setup_colour : [u8;3], symbol : Option<String>) -> Pixel {
        // let mut symbol_defined = false;
        // if let Some(x) = symbol {
            // symbol_defined = true;
        // }
                // let std_out = "symbol";
        // match symbol_defined {
            // true => {
                // let std_out = "symbol";
            // }   
            // _ => {
                // let std_out = " ";
            // }
        // }
        // Pixel { 
            // colour : setup_colour,
            // stdout_symbol : std_out,
            // colour_spec : ColorSpec::new().set_fg(Some(Colour::Rgb(setup_colour)))
        // }
    // }
// }

fn main() -> std::io::Result<()> {
    println!("No Config Found! Please setup the device config");
    {
        let esp_if = Devicecfg::default();
        // let esp_addr = SocketAddr::new(esp_if.device_specific_cfg.udp_ip, esp_if.device_specific_cfg.udp_port);
        let esp_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5005);
        let init_strip = make_random_led_vec(25);
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        // writeln!(&mut stdout, "green text!");
        let mut box_buff : termcolor::Buffer;// = "█";
        for led_idx in init_strip {
            // let pixel = Pixel::new(led_idx);
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Rgb(led_idx[0], led_idx[1], led_idx[2]))));
            println!("▀");
            // println!("led_val: {:?}", led_idx);
            update_esp8266(esp_addr, &led_idx)?;
        }
        
        let (mut stream, buffers) = init_audio(&esp_if).unwrap();
        stream.start().expect("Unable to open stream");
        thread::sleep(time::Duration::from_secs(3));
        // let handle = thread::spawn(move || {
        //     let mut index = 0;
        //     while (!buffers[index].lock().unwrap().rendered) {
        //         let mut buffer = buffers[index].lock().unwrap();
        //         // ys_data.copy_from_slice(&buffer.analytic);
        //         buffer.rendered = true;
        //         index = (index + 1) % buffers.len();
        //         for buff_idx in &buffer.analytic {
        //             println!("buf: {:?}", buff_idx);
    
        //         }
        //     }

            
        // });
        display(buffers);
        thread::sleep(time::Duration::from_secs(3));
        // handle.join().unwrap();   
        stream.stop();

    }
    Ok(()) 
}


fn colour_from_vert4(base_hue : f32, decay : f32, desaturation : f32, relative_length : f32, angle : Vec2, position : f32) -> std::io::Result<(f64)> {
    let colour : f64 = 0.0;

    Ok(colour)
}

fn update_esp8266(socket_address : SocketAddr, esp_packet : &[u8]) -> std::io::Result<()> {
    /*
    The ESP8266 will receive and decode the packets to determine what values
    to display on the LED strip. The communication protocol supports LED strips
    with a maximum of 256 LEDs.

        |i|r|g|b|
    where
        i (0 to 255): Index of LED to change (zero-based)
        r (0 to 255): Red value of LED
    The packet encoding scheme is:
        g (0 to 255): Green value of LED
        b (0 to 255): Blue value of LED
    */
    {
        let local_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5010);
        let socket = UdpSocket::bind(local_address)?;
        socket.send_to(esp_packet, socket_address)?;
    }
    Ok(())
}
// Receives a single datagram message on the socket. If `buf` is too small to hold
// the message, it will be cut off.
// let mut buf = [0; 10];
// let (amt, src) = socket.recv_from(&mut buf)?;
// println!("src is {:?}", src);

// Redeclare `buf` as slice of the received data and send reverse data back to origin.
// let buf = &mut buf[..amt];
// buf.reverse();
// println!("buf is {:?} src is {:?}", buf, src);
