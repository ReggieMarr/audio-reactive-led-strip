use std::thread;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
extern crate portaudio;
use portaudio::{
    PortAudio,
    Stream,
    NonBlocking,
    Input,
    StreamParameters,
    InputStreamSettings,
    InputStreamCallbackArgs,
    Continue,
};
use num::complex::Complex;
use rustfft::FFTplanner;
use glium::*;//{
    // Display,
    // Vertex,
    // Surface,
    // VertexBuffer,
    // Program,
    // DrawParameters,
    // Blend,
// };

use crate::device_modules::config::*;


#[derive(Copy, Clone)]
pub struct Scalar {
    pub v: f32
}
implement_vertex!(Scalar, v);

#[derive(Copy, Clone)]
pub struct Vec2 {
    pub vec: [f32; 2],
}
implement_vertex!(Vec2, vec);

#[derive(Copy, Clone, Debug)]
pub struct Vec4 {
    pub vec: [f32; 4],
}
implement_vertex!(Vec4, vec);


pub type MultiBuffer = Arc<Vec<Mutex<AudioBuffer>>>;
pub type PortAudioStream = Stream<NonBlocking, Input<f32>>;
// pub type ReceiveType = mpsc::Receiver<mpsc>;

pub struct AudioBuffer {
    pub rendered: bool,
    pub analytic: Vec<Vec4>,
}

pub const SAMPLE_RATE: f64 = 44_100.0;
const CHANNELS: i32 = 2;
const INTERLEAVED: bool = true;

pub fn init_audio_simple(config: &Devicecfg) -> Result<(PortAudioStream, MultiBuffer), portaudio::Error> {
    let fft_size = 1024;//config.fft_bins as usize;
    //Found that I had to change the buffer size to 512, not sure if this is really 
    //neccessary but for some reason if I don't do this then I only get half the spectrum
    let buffer_size = 256;//config.audio.buffer_size as usize;
    let num_buffers = 16; //config.audio.num_buffers;
    let cutoff = 0.01;
    let q = 0.5;//config.audio.q;
    let pa = PortAudio::new().expect("Unable to init portaudio");

    let def_input = pa.default_input_device().expect("Unable to get default device");
    let input_info = pa.device_info(def_input).expect("Unable to get device info");
    println!("Default input device name: {}", input_info.name);

    let latency = input_info.default_low_input_latency;
    // Set parameters for the stream settings.
    // We pass which mic should be used, how many channels are used,
    // whether all the values of all the channels should be passed in a 
    // single audiobuffer and the latency that should be considered
    let input_params = StreamParameters::<f32>::new(def_input, CHANNELS, INTERLEAVED, latency);

    pa.is_input_format_supported(input_params, SAMPLE_RATE)?;
    // Settings for an inputstream.
    // Here we pass the stream parameters we set before,
    // the sample rate of the mic and the amount values we want 
    let settings = InputStreamSettings::new(input_params, SAMPLE_RATE, buffer_size as u32);

    let mut buffers = Vec::with_capacity(num_buffers);

    for _ in 0..num_buffers {
        buffers.push(Mutex::new(AudioBuffer {
            rendered: true,
            //why is this buffer_size + 3?
            analytic: vec![Vec4 {vec: [0.0, 0.0, 0.0, 0.0]}; buffer_size + 3],
        }));
    }
    //This creates a thread safe reference counting pointer to buffers
    let buffers = Arc::new(buffers);

    // This is a lambda which I want called with the samples
    let (receiver, callback) = {
        let mut buffer_index = 0;
        let (sender, receiver) = mpsc::channel();
        let gain = 1.0;//config.audio.gain;
        let buffers = buffers.clone();
        let mut analytic_buffer = vec![Vec4 {vec: [0.0, 0.0, 0.0, 0.0]}; buffer_size + 3];

        let mut time_index = 0;
        let mut time_ring_buffer = vec![Complex::new(0.0, 0.0); 2 * fft_size];
        // this gets multiplied to convolve stuff
        let mut complex_freq_buffer = vec![Complex::new(0.0f32, 0.0); fft_size];
        let mut complex_analytic_buffer = vec![Complex::new(0.0f32, 0.0); fft_size];

        let mut n = fft_size - buffer_size;
        if n % 2 == 0 {
            if n != 0 {
                n -= 1;
            }
        }
        let analytic = make_analytic(n, fft_size);
        let mut fft_planner = FFTplanner::new(false);
        let fft = fft_planner.plan_fft(fft_size);
        // let mut ifft_planner = FFTplanner::new(true);
        // let ifft = ifft_planner.plan_fft(fft_size);

        let mut prev_input = Complex::new(0.0, 0.0); // sample n-1
        let mut prev_diff = Complex::new(0.0, 0.0); // sample n-1 - sample n-2
        let mut angle_lp = get_lowpass(cutoff, q);
        let mut noise_lp = get_lowpass(0.05, 0.7);

        (receiver, move |InputStreamCallbackArgs { buffer: data, .. }| {
            {
                let (left, right) = time_ring_buffer.split_at_mut(fft_size);
                //This takes the buffer input to the stream and then begins describing the 
                //input using complex values on a unit circle. 
                for ((x, t0), t1) in data.chunks(CHANNELS as usize)
                    .zip(left[time_index..(time_index + buffer_size)].iter_mut())
                    .zip(right[time_index..(time_index + buffer_size)].iter_mut())
                {
                    let mono = Complex::new(gain * (x[0] + x[1]) / 2.0, 0.0);
                    *t0 = mono;
                    *t1 = mono;
                }
            }
            //this updates the time index as we continue to sample the audio stream
            time_index = (time_index + buffer_size as usize) % fft_size;
            //This represents the amplitude of the signal represented as the distance from the origin on a unit circle
            //Here we transform the signal from the time domain to the frequency domain. 
            //Note that humans can only hear sound with a frequency between 20Hz and 20_000Hz
            fft.process(&mut time_ring_buffer[time_index..time_index + fft_size], &mut complex_freq_buffer[..]);

            //the analytic array acts as a filter, removing the negative and dc portions
            //of the signal as well as filtering out the nyquist portion of the signal
            //Also applies the hamming window here 
            for (x, y) in analytic.iter().zip(complex_freq_buffer.iter_mut()) {
                *y = *x * *y;
            }
            let test_freq = complex_freq_buffer.clone();

            // for (freq_idx, freq) in test_freq.iter().take(fft_size/2).enumerate() {
            //     let bin = SAMPLE_RATE as f32 / fft_size as f32;
            //     let freq_mag = f32::sqrt((freq.re as f32).exp2() + (freq.im as f32).exp2())/fft_size as f32;
            //     let freq_val = bin*freq_idx as f32;
            //     println!("{:?}, {:?}", freq_val as f32, (20.0f32*((2.0f32*freq.im/fft_size as f32).abs().log10())));
            // }
            // By applying the inverse fourier transform we transform the signal from the frequency domain back into the 
            // time domain. However now this signal can be represented as a series of points on a unit circle.
            // ifft.process(&mut complex_freq_buffer[..], &mut complex_analytic_buffer[..]);
            analytic_buffer[0] = analytic_buffer[buffer_size];
            analytic_buffer[1] = analytic_buffer[buffer_size + 1];
            analytic_buffer[2] = analytic_buffer[buffer_size + 2];
            let scale = fft_size as f32;
            let freq_res = SAMPLE_RATE as f32 / scale;
            // for (&x, y) in complex_analytic_buffer[fft_size - buffer_size..].iter().zip(analytic_buffer[3..].iter_mut()) {
            //this takes 256 points from the complex_freq_buffer into the analytic_buffer
            // for (&x, y) in complex_freq_buffer[(fft_size - buffer_size)..].iter().zip(analytic_buffer[3..].iter_mut()) {
            let freq_iter = complex_freq_buffer[..(fft_size - buffer_size)].iter().zip(analytic_buffer[3..].iter_mut());
            for (freq_idx, (&x, y)) in freq_iter.enumerate() {
                let diff = x - prev_input; // vector
                prev_input = x;

                let angle = get_angle(diff, prev_diff).abs().log2().max(-1.0e12); // angular velocity (with processing)
                prev_diff = diff;

                let output = angle_lp(angle);

                let sample_freq = freq_res * freq_idx as f32;

                *y = Vec4 { vec: [
                    // save the scaling for later
                    // x.re / scale,
                    // x.im / scale,
                    x.re,
                    x.im,
                    // output.exp2(), // smoothed angular velocity
                    sample_freq, // smoothed angular velocity
                    noise_lp((angle - output).abs()), // average angular noise
                ]};
            }

            //what is rendered and why would dropped represent its inverse ?
            let dropped = {
                let mut buffer = buffers[buffer_index].lock().unwrap();
                let rendered = buffer.rendered;
                buffer.analytic.copy_from_slice(&analytic_buffer);
                buffer.rendered = false;
                !rendered
            };
            // for x in 0..num_buffers {
                // println!("noise {:?}", analytic_buffer[x]);
            // }
            buffer_index = (buffer_index + 1) % num_buffers;
            if dropped {
                // what does sender do generally ?
                sender.send(()).ok();
            }
            Continue
        })
    };

    // Registers the callback with PortAudio
    let mut stream = pa.open_non_blocking_stream(settings, callback)?;

    Ok((stream, buffers))
}



//this is the init_audio that came from the perceptually meaningful rust project
pub fn init_audio(config: &Devicecfg) -> Result<(PortAudioStream, MultiBuffer), portaudio::Error> {
    let buffer_size = 256;//config.audio.buffer_size as usize;
    let fft_size = 1024;//config.fft_bins as usize;
    let num_buffers = 16; //config.audio.num_buffers;
    let cutoff = 0.01;
    let q = 0.5;//config.audio.q;
    let pa = PortAudio::new().expect("Unable to init portaudio");

    let def_input = pa.default_input_device().expect("Unable to get default device");
    let input_info = pa.device_info(def_input).expect("Unable to get device info");
    println!("Default input device name: {}", input_info.name);

    let latency = input_info.default_low_input_latency;
    // Set parameters for the stream settings.
    // We pass which mic should be used, how many channels are used,
    // whether all the values of all the channels should be passed in a 
    // single audiobuffer and the latency that should be considered
    let input_params = StreamParameters::<f32>::new(def_input, CHANNELS, INTERLEAVED, latency);

    pa.is_input_format_supported(input_params, SAMPLE_RATE)?;
    // Settings for an inputstream.
    // Here we pass the stream parameters we set before,
    // the sample rate of the mic and the amount values we want 
    let settings = InputStreamSettings::new(input_params, SAMPLE_RATE, buffer_size as u32);

    let mut buffers = Vec::with_capacity(num_buffers);

    for _ in 0..num_buffers {
        buffers.push(Mutex::new(AudioBuffer {
            rendered: true,
            //why is this buffer_size + 3?
            analytic: vec![Vec4 {vec: [0.0, 0.0, 0.0, 0.0]}; buffer_size + 3],
        }));
    }
    //This creates a thread safe reference counting pointer to buffers
    let buffers = Arc::new(buffers);

    // This is a lambda which I want called with the samples
    let (receiver, callback) = {
        let mut buffer_index = 0;
        let (sender, receiver) = mpsc::channel();
        let gain = 2.0;//config.audio.gain;
        let buffers = buffers.clone();
        let mut analytic_buffer = vec![Vec4 {vec: [0.0, 0.0, 0.0, 0.0]}; buffer_size + 3];

        let mut time_index = 0;
        let mut time_ring_buffer = vec![Complex::new(0.0, 0.0); 2 * fft_size];
        // this gets multiplied to convolve stuff
        let mut complex_freq_buffer = vec![Complex::new(0.0f32, 0.0); fft_size];
        let mut complex_analytic_buffer = vec![Complex::new(0.0f32, 0.0); fft_size];

        let mut n = fft_size - buffer_size;
        if n % 2 == 0 {
            n -= 1;
        }
        let analytic = make_analytic(n, fft_size);
        let mut fft_planner = FFTplanner::new(false);
        let fft = fft_planner.plan_fft(fft_size);
        // let mut ifft_planner = FFTplanner::new(true);
        // let ifft = ifft_planner.plan_fft(fft_size);

        let mut prev_input = Complex::new(0.0, 0.0); // sample n-1
        let mut prev_diff = Complex::new(0.0, 0.0); // sample n-1 - sample n-2
        let mut angle_lp = get_lowpass(cutoff, q);
        let mut noise_lp = get_lowpass(0.05, 0.7);

        (receiver, move |InputStreamCallbackArgs { buffer: data, .. }| {
            {
                let (left, right) = time_ring_buffer.split_at_mut(fft_size);
                //This takes the buffer input to the stream and then begins describing the 
                //input using complex values on a unit circle. 
                for ((x, t0), t1) in data.chunks(CHANNELS as usize)
                    .zip(left[time_index..(time_index + buffer_size)].iter_mut())
                    .zip(right[time_index..(time_index + buffer_size)].iter_mut())
                {
                    let mono = Complex::new(gain * (x[0] + x[1]) / 2.0, 0.0);
                    *t0 = mono;
                    *t1 = mono;
                }
            }
            time_index = (time_index + buffer_size as usize) % fft_size;
            //This represents the amplitude of the signal represented as the distance from the origin on a unit circle
            //Here we transform the signal from the time domain to the frequency domain. 
            //Note that humans can only hear sound with a frequency between 20Hz and 20_000Hz
            fft.process(&mut time_ring_buffer[time_index..time_index + fft_size], &mut complex_freq_buffer[..]);

            //the analytic array acts as a filter, removing the negative and dc portions
            //of the signal as well as filtering out the nyquist portion of the signal
            for (x, y) in analytic.iter().zip(complex_freq_buffer.iter_mut()) {
                *y = *x * *y;
            }
            // By applying the inverse fourier transform we transform the signal from the frequency domain back into the 
            // time domain. However now this signal can be represented as a series of points on a unit circle.
            // ifft.process(&mut complex_freq_buffer[..], &mut complex_analytic_buffer[..]);

            //here we are filling the start of our array with the ned of the last one so that we have a continuous stream
            analytic_buffer[0] = analytic_buffer[buffer_size];
            analytic_buffer[1] = analytic_buffer[buffer_size + 1];
            analytic_buffer[2] = analytic_buffer[buffer_size + 2];
            let scale = fft_size as f32;
            // for (&x, y) in complex_analytic_buffer[fft_size - buffer_size..].iter().zip(analytic_buffer[3..].iter_mut()) {
            for (&x, y) in complex_freq_buffer[fft_size - buffer_size..].iter().zip(analytic_buffer[3..].iter_mut()) {
                let diff = x - prev_input; // vector
                prev_input = x;

                let angle = get_angle(diff, prev_diff).abs().log2().max(-1.0e12); // angular velocity (with processing)
                prev_diff = diff;

                let output = angle_lp(angle);

                *y = Vec4 { vec: [
                    x.re / scale,
                    x.im / scale,
                    output.exp2(), // smoothed angular velocity
                    noise_lp((angle - output).abs()), // average angular noise
                ]};
            }

            //what is rendered and why would dropped represent its inverse ?
            let dropped = {
                let mut buffer = buffers[buffer_index].lock().unwrap();
                let rendered = buffer.rendered;
                buffer.analytic.copy_from_slice(&analytic_buffer);
                buffer.rendered = false;
                !rendered
            };
            // for x in 0..num_buffers {
                // println!("noise {:?}", analytic_buffer[x]);
            // }
            buffer_index = (buffer_index + 1) % num_buffers;
            if dropped {
                // what does sender do generally ?
                sender.send(()).ok();
            }
            Continue
        })
    };

    // Registers the callback with PortAudio
    let mut stream = pa.open_non_blocking_stream(settings, callback)?;

    Ok((stream, buffers))
}

// angle between two complex numbers
// scales into [0, 0.5]
fn get_angle(v: Complex<f32>, u: Complex<f32>) -> f32 {
    // 2 atan2(len(len(v)*u − len(u)*v), len(len(v)*u + len(u)*v))
    let len_v_mul_u = v.norm_sqr().sqrt() * u;
    let len_u_mul_v = u.norm_sqr().sqrt() * v;
    let left = (len_v_mul_u - len_u_mul_v).norm_sqr().sqrt(); // this is positive
    let right = (len_v_mul_u + len_u_mul_v).norm_sqr().sqrt(); // this is positive
    left.atan2(right) / ::std::f32::consts::PI
}

// returns biquad lowpass filter
fn get_lowpass(n: f32, q: f32) -> Box<FnMut(f32) -> f32> {
    let k = (0.5 * n * ::std::f32::consts::PI).tan();
    let norm = 1.0 / (1.0 + k / q + k * k);
    let a0 = k * k * norm;
    let a1 = 2.0 * a0;
    let a2 = a0;
    let b1 = 2.0 * (k * k - 1.0) * norm;
    let b2 = (1.0 - k / q + k * k) * norm;

    let mut w1 = 0.0;
    let mut w2 = 0.0;
    // \ y[n]=b_{0}w[n]+b_{1}w[n-1]+b_{2}w[n-2],
    // where
    // w[n]=x[n]-a_{1}w[n-1]-a_{2}w[n-2].
    Box::new(move |x| {
        let w0 = x - b1 * w1 - b2 * w2;
        let y = a0 * w0 + a1 * w1 + a2 * w2;
        w2 = w1;
        w1 = w0;
        y
    })
}

// FIR analytical signal transform of length n with zero padding to be length m
// real part removes DC and nyquist, imaginary part phase shifts by 90
// should act as bandpass (remove all negative frequencies + DC & nyquist)
fn make_analytic(n: usize, m: usize) -> Vec<Complex<f32>> {
    use ::std::f32::consts::PI;
    assert_eq!(n % 2, 1, "n should be odd");
    assert!(n <= m, "n should be less than or equal to m");
    // let a = 2.0 / n as f32;
    let mut fft_planner = FFTplanner::new(false);
    //this probably doesn't need to be mut
    let mut fft = fft_planner.plan_fft(m);

    let mut impulse = vec![Complex::new(0.0, 0.0); m];
    let mut freqs = impulse.clone();

    let mid = (n - 1) / 2;

    impulse[mid].re = 1.0;
    let re = -1.0 / (mid - 1) as f32;
    for i in 1..mid+1 {
        if i % 2 == 0 {
            impulse[mid + i].re = re;
            impulse[mid - i].re = re;
        } else {
            let im = 2.0 / PI / i as f32;
            impulse[mid + i].im = im;
            impulse[mid - i].im = -im;
        }
        // hamming window
        let k = 0.53836 + 0.46164 * (i as f32 * PI / (mid + 1) as f32).cos();
        impulse[mid + i] = impulse[mid + i].scale(k);
        impulse[mid - i] = impulse[mid - i].scale(k);
    }
    fft.process(&mut impulse, &mut freqs);
    freqs
}

#[test]
fn test_analytic() {
    let m = 1024; // ~ 40hz
    let n = m / 4 * 3 - 1; // overlap 75%
    let freqs = make_analytic(n, m);
    // DC is below -6db
    assert!(10.0 * freqs[0].norm_sqr().log(10.0) < -6.0);
    // 40hz is above 0db
    assert!(10.0 * freqs[1].norm_sqr().log(10.0) > 0.0);
    // -40hz is below -12db
    assert!(10.0 * freqs[m-1].norm_sqr().log(10.0) < -12.0);
    // actually these magnitudes are halved bc passband is +6db
}

#[test]
fn test_lowpass() {
    let mut lp = get_lowpass(0.5, 0.71);
    println!();
    println!("{}", lp(1.0));
    for _ in 0..10 {
        println!("{}", lp(0.0));
    }
    for _ in 0..10 {
        assert!(lp(0.0).abs() < 0.5); // if it's unstable, it'll be huge
    }
}
