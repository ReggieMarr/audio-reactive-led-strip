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

// mod plotting;
// use plotting::*;

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
        let handle = thread::spawn(move || {
            let mut index = 0;
            while (!buffers[index].lock().unwrap().rendered) {
                let mut buffer = buffers[index].lock().unwrap();
                // ys_data.copy_from_slice(&buffer.analytic);
                buffer.rendered = true;
                index = (index + 1) % buffers.len();
                for buff_idx in &buffer.analytic {
                    println!("buf: {:?}", buff_idx);
    
                }
            }

            
        });
        // display(buffers);
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
        r (0 to 255): red value of LED
    The packet encoding scheme is:
        g (0 to 255): green value of LED
        b (0 to 255): blue value of LED
    */
    {
        let local_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5010);
        let socket = UdpSocket::bind(local_address)?;
        socket.send_to(esp_packet, socket_address)?;
    }
    Ok(())
}

/*
    From Wavelength to RGB in Python - https://www.noah.org/wiki/Wavelength_to_RGB_in_Python
    == A few notes about color ==

    Color   Wavelength(nm) Frequency(THz)
    red     620-750        484-400
    Orange  590-620        508-484
    Yellow  570-590        526-508
    green   495-570        606-526
    blue    450-495        668-606
    Violet  380-450        789-668

    f is frequency (cycles per second)
    l (lambda) is wavelength (meters per cycle)
    e is energy (Joules)
    h (Plank's constant) = 6.6260695729 x 10^-34 Joule*seconds
                         = 6.6260695729 x 10^-34 m^2*kg/seconds
    c = 299792458 meters per second
    f = c/l
    l = c/f
    e = h*f
    e = c*h/l

    List of peak frequency responses for each type of 
    photoreceptor cell in the human eye:
        S cone: 437 nm
        M cone: 533 nm
        L cone: 564 nm
        rod:    550 nm in bright daylight, 498 nm when dark adapted. 
                Rods adapt to low light conditions by becoming more sensitive.
                Peak frequency response shifts to 498 nm.
    if wavelength >= 380 and wavelength <= 440:
        attenuation = 0.3 + 0.7 * (wavelength - 380) / (440 - 380)
        R = ((-(wavelength - 440) / (440 - 380)) * attenuation) ** gamma
        G = 0.0
        B = (1.0 * attenuation) ** gamma
    elif wavelength >= 440 and wavelength <= 490:
        R = 0.0
        G = ((wavelength - 440) / (490 - 440)) ** gamma
        B = 1.0
    elif wavelength >= 490 and wavelength <= 510:
        R = 0.0
        G = 1.0
        B = (-(wavelength - 510) / (510 - 490)) ** gamma
    elif wavelength >= 510 and wavelength <= 580:
        R = ((wavelength - 510) / (580 - 510)) ** gamma
        G = 1.0
        B = 0.0
    elif wavelength >= 580 and wavelength <= 645:
        R = 1.0
        G = (-(wavelength - 645) / (645 - 580)) ** gamma
        B = 0.0
    elif wavelength >= 645 and wavelength <= 750:
        attenuation = 0.3 + 0.7 * (750 - wavelength) / (750 - 645)
        R = (1.0 * attenuation) ** gamma
        G = 0.0
        B = 0.0
    else:
        R = 0.0
        G = 0.0
        B = 0.0
    R *= 255
    G *= 255
    B *= 255
    return (int(R), int(G), int(B))
*/


const MINIMUM_VISIBLE_WAVELENGTH :u16 = 380;
const MAXIMUM_VISIBLE_WAVELENGTH :u16 = 740;

fn wavelength_to_rgb(wavelength : f32, gamma : f32) -> std::io::Result<([u8;3])> {
    let red : f32;
    let green : f32;
    let blue : f32;

    /*
        Color 	Wavelength (nm)
        red 	625 - 740
        Orange 	590 - 625
        Yellow 	565 - 590
        green 	520 - 565
        Cyan 	500 - 520
        blue 	435 - 500
        Violet 	380 - 435
    */
    if wavelength > 440.0 && wavelength < 490.0 {
        let attenuation = 0.3 + 0.7*(wavelength 
            - MINIMUM_VISIBLE_WAVELENGTH as f32);
        red = (-wavelength - 440.0) / (440.0 - MINIMUM_VISIBLE_WAVELENGTH as f32)
             * attenuation * gamma;
        green = 0.0;
        blue = 1.0 * attenuation;
    }
    else if wavelength >= 440.0 && wavelength <= 490.0 {
        red = 0.0;
        green = ((wavelength - 440.0) / (490.0 - 440.0)) * gamma;
        blue = 1.0;
    }
    else if wavelength >= 490.0 && wavelength <= 510.0 {
        red = 0.0;
        green = 1.0;
        blue = (-(wavelength - 510.0) / (510.0 - 490.0)) * gamma;
    }
    else if wavelength >= 510.0 && wavelength <= 580.0 {
        red = ((wavelength - 510.0) / (580.0 - 510.0)) * gamma;
        green = 1.0;
        blue = 0.0;
    }
    else if wavelength >= 580.0 && wavelength <= 645.0 {
        red = 1.0;
        green = (-(wavelength - 645.0) / (645.0 - 580.0)) * gamma;
        blue = 0.0;
    }
    else if wavelength >= 645.0 && wavelength 
        <= MAXIMUM_VISIBLE_WAVELENGTH as f32 {
        let attenuation = 0.3 + 0.7 * 
            (MAXIMUM_VISIBLE_WAVELENGTH as f32 - wavelength) 
            / (MAXIMUM_VISIBLE_WAVELENGTH as f32 - 645.0);
        red = (1.0 * attenuation) * gamma;
        green = 0.0;
        blue = 0.0;
    }
    else {
        red = 0.0;
        green = 0.0;
        blue = 0.0;
    }
    let rgb = [(255.0 * red) as u8, (255.0 * green) as u8, (255.0 * blue) as u8];

    Ok(rgb)
}
// Receives a single datagram message on the socket. If `buf` is too small to hold
// the message, it will be cut off.
// let mut buf = [0; 10];
// let (amt, src) = socket.recv_from(&mut buf)?;
// println!("src is {:?}", src);

// redeclare `buf` as slice of the received data and send reverse data back to origin.
// let buf = &mut buf[..amt];
// buf.reverse();
// println!("buf is {:?} src is {:?}", buf, src);