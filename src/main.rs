//! Records a WAV file using the default input device and default input format.
//!
//! The input data is recorded to "$CARGO_MANIFEST_DIR/recorded.wav".
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

use nannou::prelude::*;
use nannou_audio as audio;
use nannou_audio::Buffer;

fn main() {
    nannou::app(model).run();
}

const MAX_BUFFER_SIZE: usize = 1000;

struct Model {
    stream: audio::Stream<CaptureModel>,
    buffer: Arc<RwLock<VecDeque<f32>>>,
}

struct CaptureModel {
    buffer: Arc<RwLock<VecDeque<f32>>>,
}

fn model(app: &App) -> Model {
    // Create a window to receive key pressed events.
    app.new_window()
        .key_pressed(key_pressed)
        .view(view)
        .build()
        .unwrap();

    // Initialise the audio host so we can spawn an audio stream.
    let audio_host = audio::Host::new();

    // Create a writer
    let buffer = Arc::new(RwLock::new(VecDeque::new()));
    let capture_model = CaptureModel {
        buffer: Arc::clone(&buffer),
    };

    let stream = audio_host
        .new_input_stream(capture_model)
        .capture(capture_fn)
        .build()
        .unwrap();

    stream.play().unwrap();

    Model { stream, buffer }
}

// A function that captures the audio from the buffer and
// writes it into the the WavWriter.
fn capture_fn(audio: &mut CaptureModel, buffer: &Buffer) {
    let mut samples_buffer = audio.buffer.write().unwrap();

    for frame in buffer.frames() {
        for sample in frame {
            samples_buffer.push_back(*sample);
            if samples_buffer.len() > MAX_BUFFER_SIZE {
                samples_buffer.pop_front();
            }
        }
    }

    // dbg!("buf size {}", samples_buffer.len());
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    if let Key::Space = key {
        if model.stream.is_paused() {
            model.stream.play().unwrap();
        } else if model.stream.is_playing() {
            model.stream.pause().unwrap();
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    // Get the rectangle (size) of the window
    let win = app.window_rect();

    // Get an object from nannou that let's us easily draw shapes
    let draw = app.draw();

    draw.background().color(BLACK);

    // frame.clear(DIMGRAY);

    let mut points = [Vec2::ZERO; MAX_BUFFER_SIZE];
    let first_index = 0;
    let last_index = points.len() - 1;

    let sample_buf = model.buffer.read().unwrap();

    

    for (index, point_ref) in points.iter_mut().enumerate() {
        // Calculate the `x` position from the index
        // first point in array = -1.0 = left of line
        // last  point in array =  1.0 = right of line
        let x = map_range(index, first_index, last_index, -1.0, 1.0);

        // Calculate the `y` position
        let y = sample_buf.get(index).unwrap_or(&0.0);

        // Convert the above normalized (-1.0 -> 0.0) coordinates to window coordinates
        //     https://guide.nannou.cc/tutorials/basics/window-coordinates
        let window_x = map_range(x, -1.0, 1.0, win.left(), win.right());
        let window_y = map_range(*y, -1.0, 1.0, win.bottom(), win.top());

        *point_ref = pt2(window_x, window_y);
    }

    // dbg!(&points);
    // Fill with black to obscure lines behind this one
    draw.polygon().color(BLACK).points(points);

    // dbg!(win.bottom());
    // dbg!(win.top());

    // dbg!(win.left());
    // dbg!(win.right());
    // Draw white outline
    draw.path()
        .stroke()
        .color(WHITE)
        .stroke_weight(2.0)
        .join_round()
        .caps_round()
        .points(points);

    draw.to_frame(app, &frame).unwrap();
}
