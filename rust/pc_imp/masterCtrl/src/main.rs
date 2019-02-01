#[allow(dead_code)]

use std::net::{Ipv4Addr};
use std::any::Any;
#[macro_use]
extern crate serde_derive;

extern crate serde;
use serde::{Serialize,Deserialize};
use serde_json::{Result,Value};



#[derive(Debug,PartialEq,Eq)]
enum DeviceType {
    ESP8266,
    RASPBERRY_PI,
    BLINKSTICK
}
#[derive(Debug,PartialEq,Eq)]
enum StatusType {
    ERROR,
    OK
}

#[allow(dead_code)]
#[derive(Debug)]
struct Devicecfg {
    use_gui : bool,
    //Whether or not to display a PyQtGraph GUI plot of visualization
    display_fps : bool,
    //Whether to display the FPS when running (can reduce performance)
    pixel_num : u8,
    //Number of pixels in the LED strip (must match ESP8266 firmware)
    gamma_table_path : String,
    //Location of the gamma correction table"
    mic_rate : u32,
    //Sampling frequency of the microphone in Hz
    fps : u8,
    //Desired refresh rate of the visualization (frames per second)
    min_led_fps : u32,
    //Frequencies below this value will be removed during audio processing
    max_led_fps : u32,
    //Frequencies above this value will be removed during audio processing
    device_type : DeviceType
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
            device_type : DeviceType::ESP8266
        }
    }
}

trait DeviceTypeInheritance {
    fn transmit_type(&self);
}

#[derive(Debug)]
struct RaspberryPiCfg {
    generic_specs : Devicecfg,
    //configurable specs that are not specific to the type of device
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
            generic_specs : Devicecfg::default(),
            led_pin : 18,
            led_freq_hz : 800_000,
            led_dma : 5,
            brightness : 255,
            led_invert : true,
            software_gamma_correction : true
        }
    }
}

impl DeviceTypeInheritance for RaspberryPiCfg {
    fn transmit_type(&self) {
        self.generic_specs.device_type = DeviceType::RASPBERRY_PI;
    }
}

#[allow(dead_code)]
struct Esp8266Cfg {
    generic_specs : Devicecfg,
    //configurable specs that are not specific to the type of device
    udp_ip : Ipv4Addr,
    //IP address of the ESP8266. Must match IP in ws2812_controller.ino
    udp_port : u16,
    //Port number used for socket communication between Python and ESP8266"
    software_gamma_correction : bool
    //Set to True because Raspberry Pi doesn't use hardware dithering
}

impl Default for Esp8266Cfg {
    fn default() -> Esp8266Cfg {
        Esp8266Cfg {
            generic_specs : Devicecfg::default(),
            udp_ip : Ipv4Addr::new(192, 168, 2, 165),
            udp_port : 7777,
            software_gamma_correction : false
        }
    }
}

impl DeviceTypeInheritance for Esp8266Cfg {
    fn transmit_type(&self) {
        self.generic_specs.device_type = DeviceType::ESP8266;
    }
}

#[allow(dead_code)]
struct BlinkstickCfg {
    generic_specs : Devicecfg,
    //configurable specs that are not specific to the type of device
    software_gamma_correction : bool
    //Set to True because BlinkstickCfg doesn't use hardware dithering
}

impl Default for BlinkstickCfg {
    fn default() -> BlinkstickCfg {
        BlinkstickCfg {
            generic_specs : Devicecfg::default(),
            software_gamma_correction : true
        }
    }
}

impl DeviceTypeInheritance for BlinkstickCfg {
    fn transmit_type(&self) {
        self.generic_specs.device_type = DeviceType::BLINKSTICK;
    }
}

fn main() {
    let mut example_settings = Devicecfg::default();
    println!("device type is {:?}",example_settings.device_type);
    let ret = setup_device(DeviceType::ESP8266,&mut example_settings);

    if ret == StatusType::OK {
        println!("device type is {:?}",example_settings.device_type);
    }
    else {
        println!("something went wrong");
    }
    

}

#[allow(unreachable_code)]
fn setup_device(cfg_device:DeviceType,cfg_settings:&mut Devicecfg) -> (StatusType) {

    match cfg_device {
        DeviceType::ESP8266 => {
            cfg_settings.device_type = DeviceType::RASPBERRY_PI;
            return StatusType::OK;
        }
        DeviceType::RASPBERRY_PI => {
            cfg_settings.device_type = DeviceType::RASPBERRY_PI;
            return StatusType::OK;
        }
        DeviceType::BLINKSTICK => {
            cfg_settings.device_type = DeviceType::BLINKSTICK;
            return StatusType::OK;
        }
    }

    return StatusType::ERROR;

}