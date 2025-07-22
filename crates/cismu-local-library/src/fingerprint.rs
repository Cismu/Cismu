use anyhow::{Result, anyhow};
use chromaprint::Chromaprint;
use std::{fs::File, path::Path};
use symphonia::core::{
    audio::SampleBuffer, codecs::DecoderOptions, errors::Error as SymphError, formats::FormatOptions,
    io::MediaSourceStream, meta::MetadataOptions, probe::Hint,
};
use symphonia::default::{get_codecs, get_probe};

pub fn fingerprint_from_file<P: AsRef<Path>>(path: P) -> Result<String> {
    // 1. Abre el archivo y crea el stream de medios
    let file = File::open(&path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    // 2. Hint para detección de formato basada en extensión
    let mut hint = Hint::new();
    if let Some(ext) = path.as_ref().extension().and_then(|s| s.to_str()) {
        hint.with_extension(ext);
    }

    // 3. Probar formato
    let probed = get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| anyhow!("Error probing format: {}", e))?;
    let mut format = probed.format;

    // 4. Selecciona la pista de audio
    let track = format
        .default_track()
        .ok_or_else(|| anyhow!("No se encontró pista de audio"))?;
    let params = &track.codec_params;

    // 5. Crea el decodificador
    let mut decoder = get_codecs()
        .make(params, &DecoderOptions::default())
        .map_err(|e| anyhow!("Error creando decodificador: {}", e))?;

    // 6. Inicializa Chromaprint
    let sample_rate = params
        .sample_rate
        .ok_or_else(|| anyhow!("Sample rate desconocido"))? as i32;
    let channels = params
        .channels
        .ok_or_else(|| anyhow!("Canales desconocidos"))?
        .count() as i32;

    let mut ctx = Chromaprint::new();
    if !ctx.start(sample_rate, channels) {
        return Err(anyhow!("Chromaprint start falló"));
    }

    // ────────────────────────────────────────────────
    // LÓGICA DE CORTE EN 120 SEGUNDOS
    // ────────────────────────────────────────────────
    let max_secs = 120;
    let max_samples = sample_rate as usize * channels as usize * max_secs;
    let mut total_samples: usize = 0;
    let mut sample_buf: Option<SampleBuffer<i16>> = None;

    // 7. Recorre paquetes de audio
    loop {
        match format.next_packet() {
            Ok(packet) => match decoder.decode(&packet) {
                Ok(audio_buf) => {
                    // Inicializa el SampleBuffer en el primer paquete
                    if sample_buf.is_none() {
                        let spec = *audio_buf.spec();
                        let capacity = audio_buf.capacity() as u64;
                        sample_buf = Some(SampleBuffer::new(capacity, spec));
                    }
                    let sb = sample_buf.as_mut().unwrap();
                    sb.copy_interleaved_ref(audio_buf);

                    // Alimenta Chromaprint
                    let samples = sb.samples();
                    if !ctx.feed(samples) {
                        return Err(anyhow!("Chromaprint feed falló"));
                    }

                    // Suma y verifica si excedemos los 120 s
                    total_samples += samples.len();
                    if total_samples >= max_samples {
                        break; // ¡suficiente audio procesado!
                    }
                }
                Err(err) => match err {
                    SymphError::IoError(_) | SymphError::DecodeError(_) => break,
                    _ => return Err(anyhow!("Error de decodificación: {}", err)),
                },
            },
            Err(_) => break, // EOF u otro error de lectura
        }
    }

    // 8. Finaliza y obtiene la huella
    if !ctx.finish() {
        return Err(anyhow!("Chromaprint finish falló"));
    }
    let fingerprint = ctx
        .fingerprint()
        .ok_or_else(|| anyhow!("No se pudo obtener fingerprint"))?;

    Ok(fingerprint)
}
