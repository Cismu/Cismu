use std::error::Error;
use std::sync::{Arc, Mutex};

use crate::audio_api::audio_destination_node::AudioDestinationNode;
use crate::audio_api::AudioBuffer;

#[derive(Clone)]
pub struct AudioBufferSourceNode {
    buffer: Option<Arc<Mutex<AudioBuffer>>>,
    is_playing: bool,
    // Variables que capturan los valores de atributos y AudioParams
    loop_playback: bool,
    detune: f32,
    loop_start: f64,
    loop_end: f64,
    playback_rate: f32,
    // Variables para los parámetros de reproducción del nodo
    start_time: f64,
    offset: f64,
    duration: f64,
    stop_time: f64,
    // Variables para rastrear el estado de reproducción del nodo
    buffer_time: f64,
    started: bool,
    entered_loop: bool,
    buffer_time_elapsed: f64,
    // Tasa de muestreo del contexto (suponemos 44100 Hz)
    sample_rate: f64,
}

impl AudioBufferSourceNode {
    pub fn new() -> Self {
        Self {
            buffer: None,
            is_playing: false,
            loop_playback: false,
            detune: 0.0,
            loop_start: 0.0,
            loop_end: 0.0,
            playback_rate: 1.0,
            start_time: 0.0,
            offset: 0.0,
            duration: f64::INFINITY,
            stop_time: f64::INFINITY,
            buffer_time: 0.0,
            started: false,
            entered_loop: false,
            buffer_time_elapsed: 0.0,
            sample_rate: 44100.0,
        }
    }

    pub fn set_buffer(&mut self, buffer: Arc<Mutex<AudioBuffer>>) -> Result<(), Box<dyn Error>> {
        // Aquí puedes implementar la lógica de [[buffer set]] si es necesario
        self.buffer = Some(buffer);
        Ok(())
    }

    pub fn set_playback_rate(&mut self, rate: f32) {
        self.playback_rate = rate;
    }

    pub fn set_detune(&mut self, detune: f32) {
        self.detune = detune;
    }

    pub fn set_loop(&mut self, loop_playback: bool) {
        self.loop_playback = loop_playback;
    }

    pub fn set_loop_start(&mut self, start: f64) {
        self.loop_start = start;
    }

    pub fn set_loop_end(&mut self, end: f64) {
        self.loop_end = end;
    }

    pub fn start(
        &mut self,
        when: Option<f64>,
        offset: Option<f64>,
        duration: Option<f64>,
    ) -> Result<(), Box<dyn Error>> {
        if self.started {
            return Err("start() can only be called once".into());
        }
        // Maneja la invocación del método start
        self.start_time = when.unwrap_or(0.0);
        self.offset = offset.unwrap_or(0.0);
        if let Some(dur) = duration {
            self.duration = dur;
        }
        self.started = false; // Se establecerá en true en el proceso de reproducción
        self.is_playing = true;
        Ok(())
    }

    pub fn stop(&mut self, when: Option<f64>) {
        // Maneja la invocación del método stop
        self.stop_time = when.unwrap_or(0.0);
    }

    // Función para obtener la señal de reproducción en una posición dada
    fn playback_signal(&self, channel_data: &[f32], position: f64) -> f32 {
        /*
            Esta función proporciona la señal de reproducción para el buffer,
            mapeando desde una posición de cabezal de reproducción a un valor de
            señal de salida. Si la posición corresponde a una muestra exacta, devuelve
            ese valor; de lo contrario, realiza una interpolación lineal entre las
            muestras vecinas.
        */
        let sample_pos = position * self.sample_rate;
        let index = sample_pos.floor() as usize;
        let alpha = sample_pos - index as f64;

        if index + 1 < channel_data.len() {
            let sample1 = channel_data[index];
            let sample2 = channel_data[index + 1];
            ((1.0 - alpha) as f32) * sample1 + (alpha as f32) * sample2
        } else if index < channel_data.len() {
            channel_data[index]
        } else {
            0.0 // Fuera del rango del buffer
        }
    }

    // Función de procesamiento que genera un bloque de audio
    pub fn process(&mut self, number_of_frames: usize, current_time: f64) -> Vec<f32> {
        let mut output = Vec::with_capacity(number_of_frames);

        // Combina los parámetros playbackRate y detune
        let computed_playback_rate = self.playback_rate * 2f32.powf(self.detune / 1200.0);
        let computed_playback_rate = computed_playback_rate as f64;

        // Determina los puntos de bucle según corresponda
        let (actual_loop_start, actual_loop_end) = if self.loop_playback && self.buffer.is_some() {
            let buffer = self.buffer.as_ref().unwrap().lock().unwrap();
            let buffer_duration = buffer.duration();

            if self.loop_start >= 0.0 && self.loop_end > 0.0 && self.loop_start < self.loop_end {
                (self.loop_start, self.loop_end.min(buffer_duration))
            } else {
                (0.0, buffer_duration)
            }
        } else {
            // Si loop es false, eliminamos cualquier registro de haber entrado en el bucle
            self.entered_loop = false;
            (0.0, 0.0)
        };

        // Maneja el caso de buffer nulo
        if self.buffer.is_none() {
            self.stop_time = current_time; // Fuerza salida en silencio
        }

        let dt = 1.0 / self.sample_rate;

        for _ in 0..number_of_frames {
            // Verifica si currentTime y bufferTimeElapsed están dentro del rango de reproducción
            if current_time < self.start_time
                || current_time >= self.stop_time
                || self.buffer_time_elapsed >= self.duration
            {
                output.push(0.0); // Muestra silenciosa
                continue;
            }

            if !self.started {
                // Nota que el buffer ha comenzado a reproducirse y obtiene la posición inicial
                if self.loop_playback
                    && computed_playback_rate >= 0.0
                    && self.offset >= actual_loop_end
                {
                    self.offset = actual_loop_end;
                }
                if computed_playback_rate < 0.0
                    && self.loop_playback
                    && self.offset < actual_loop_start
                {
                    self.offset = actual_loop_start;
                }
                self.buffer_time = self.offset;
                self.started = true;
            }

            // Maneja cálculos relacionados con el bucle
            if self.loop_playback {
                // Determina si la parte en bucle ha sido ingresada por primera vez
                if !self.entered_loop {
                    if self.offset < actual_loop_end && self.buffer_time >= actual_loop_start {
                        // La reproducción comenzó antes o dentro del bucle, y el cabezal está ahora después de loopStart
                        self.entered_loop = true;
                    }
                    if self.offset >= actual_loop_end && self.buffer_time < actual_loop_end {
                        // La reproducción comenzó después del bucle, y el cabezal está ahora antes de loopEnd
                        self.entered_loop = true;
                    }
                }
                // Ajusta las iteraciones del bucle según sea necesario
                if self.entered_loop {
                    let loop_duration = actual_loop_end - actual_loop_start;
                    while self.buffer_time >= actual_loop_end {
                        self.buffer_time -= loop_duration;
                    }
                    while self.buffer_time < actual_loop_start {
                        self.buffer_time += loop_duration;
                    }
                }
            }

            let sample_value = if let Some(buffer) = &self.buffer {
                let mut buffer = buffer.lock().unwrap();
                if self.buffer_time >= 0.0 && self.buffer_time < buffer.duration() {
                    let channel_data = buffer.get_channel_data(0).unwrap();
                    self.playback_signal(channel_data, self.buffer_time)
                } else {
                    0.0 // Fuera del rango del buffer, salida en silencio
                }
            } else {
                0.0 // Buffer nulo, salida en silencio
            };

            output.push(sample_value);

            // Actualiza las variables de tiempo
            self.buffer_time += dt * computed_playback_rate;
            self.buffer_time_elapsed += dt * computed_playback_rate;
            // currentTime += dt; // En este contexto, currentTime se pasa externamente
        }

        if current_time >= self.stop_time {
            // Finaliza el estado de reproducción de este nodo
            self.is_playing = false;
        }

        output
    }

    pub fn connect(&self, destination: &mut AudioDestinationNode) {
        let source = Arc::new(Mutex::new(self.clone()));

        // Añade este AudioBufferSourceNode al nodo de destino
        // destination.add_source(source.clone());
    }
}
