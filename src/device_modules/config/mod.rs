#[allow(dead_code)]

use std::slice::{Iter, IterMut};
use std::net::{IpAddr, Ipv4Addr};
#[macro_use]

use serde::{Serialize,Deserialize};
use serde_json::{Result,Value};

#[derive(Debug)]
pub struct RaspberryPiCfg {
    led_pin : u8,
    //GPIO pin connected to the LED strip pixels (must support PWM)
    led_freq_hz : u32,
    //LED signal frequency in Hz (usually 800kHz)
    led_dma : u8,
    //DMA channel used for generating PWM signal (try 5)
    brightness : u8,
    //Brightness of LED strip between 0 and 255"
    led_invert : bool,
    //Set True if using an inverting logic level converter
    software_gamma_correction : bool
    //Set to True because Raspberry Pi doesn't use hardware dithering
}

impl Default for RaspberryPiCfg {
    fn default() -> RaspberryPiCfg {
        RaspberryPiCfg {
            led_pin : 18,
            led_freq_hz : 800_000,
            led_dma : 5,
            brightness : 255,
            led_invert : true,
            software_gamma_correction : true
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Esp8266Cfg {
    //IP address of the ESP8266. Must match IP in ws2812_controller.ino
    pub udp_ip : IpAddr,
    //Port number used for socket communication between Python and ESP8266"
    pub udp_port : u16,
    //Set to True because Raspberry Pi doesn't use hardware dithering
    software_gamma_correction : bool
}

impl Default for Esp8266Cfg {
    fn default() -> Esp8266Cfg {
        Esp8266Cfg {
            udp_ip : IpAddr::V4(Ipv4Addr::new(192, 168, 2, 165)),
            udp_port : 7777,
            software_gamma_correction : false
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct BlinkstickCfg {
    //Set to True because BlinkstickCfg doesn't use hardware dithering
    software_gamma_correction : bool
}

impl Default for BlinkstickCfg {
    fn default() -> BlinkstickCfg {
        BlinkstickCfg {
            software_gamma_correction : true
        }
    }
}

#[derive(Debug)]
pub enum DeviceType {
    ESP8266,
    RASPBERRY_PI,
    BLINKSTICK
}
#[derive(Debug,PartialEq,Eq)]
pub enum StatusType {
    ERROR,
    OK
}

// macro_rules! field_names_to_key {
    // (struct $name:ident { $($fname:ident : $ftype:ty),* }) => {
        // struct $name {
            // $($fname : $ftype),*
        // }
// 
        // impl $name {
            // fn field_names() -> &'static [&'static str] {
                // static NAMES: &'static [&'static str] = &[$(stringify!($fname)),*];
                // NAMES
            // }
        // }
    // }
// }

#[allow(dead_code)]
#[derive(Debug)]
pub struct Devicecfg {
//Whether or not to display a PyQtGraph GUI plot of visualization
    pub use_gui : bool,
    //Whether to display the FPS when running (can reduce performance)
    pub display_fps : bool,
    //Number of pixels in the LED strip (must match ESP8266 firmware)
    pub pixel_num : u8,
    //Location of the gamma correction table"
    pub gamma_table_path : String,
    //Sampling frequency of the microphone in Hz
    pub mic_rate : u32,
    //Desired refresh rate of the visualization (frames per second)
    pub fps : u8,
    //Frequencies below this value will be removed during audio processing
    pub min_led_fps : u32,
    //Frequencies above this value will be removed during audio processing
    pub max_led_fps : u32,
    pub fft_bins : usize,
    pub device_type : DeviceType,
    pub device_specific_cfg : Esp8266Cfg
}
impl Default for Devicecfg {
    fn default() -> Devicecfg {
        Devicecfg {
            use_gui : true,
            display_fps : true,
            pixel_num : 65,
            gamma_table_path : "directory".to_string(),
            mic_rate : 44_100,
            fps : 60,
            min_led_fps : 200,
            max_led_fps : 12_000,
            fft_bins : 24,
            device_type : DeviceType::ESP8266,
            device_specific_cfg : Esp8266Cfg::default()
        }
    }
}
// impl<'a, T> IntoIterator for &'a mut Devicecfg<> {
    // type Item = &'a mut T;
    // type IntoIter = Iter<'a,T>;
    // fn into_iter(self) -> Iter<'a,T>
// }