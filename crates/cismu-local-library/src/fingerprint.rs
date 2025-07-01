use anyhow::{Result, anyhow};
use std::{fs::File, path::Path};

use symphonia::core::{
    audio::SampleBuffer,
    codecs::DecoderOptions,
    formats::FormatOptions,
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint, // <-- importar Hint
};
use symphonia::default::{get_codecs, get_probe};

use chromaprint::Chromaprint; // Crate chromaprint 0.2.0

pub fn fingerprint_from_file<P: AsRef<Path>>(path: P) -> Result<String> {
    // 1. Abre el archivo y crea el stream de medios
    let file = File::open(&path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    // 2. Prepara un Hint (opcional, pero recomendado) para la detección de formato
    let mut hint = Hint::new();
    if let Some(ext) = path.as_ref().extension().and_then(|s| s.to_str()) {
        hint.with_extension(ext);
    }

    // 3. Auto-detección del formato (ahora con &hint como primer argumento)
    let probed = get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| anyhow!("Error probing format: {}", e))?;
    let mut format = probed.format;

    // 4. Selecciona la primera pista de audio
    let track = format
        .default_track()
        .ok_or_else(|| anyhow!("No se encontró pista de audio"))?;
    let params = &track.codec_params;

    // 5. Crea el decodificador
    let mut decoder = get_codecs()
        .make(params, &DecoderOptions::default())
        .map_err(|e| anyhow!("Error creando decodificador: {}", e))?;

    // 6. Inicializa Chromaprint con sample_rate y canales
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

    // 7. Creamos el SampleBuffer al primer paquete decodificado
    let mut sample_buf: Option<SampleBuffer<i16>> = None;

    // 8. Recorremos los paquetes, decodificamos y alimentamos a Chromaprint
    loop {
        match format.next_packet() {
            Ok(packet) => match decoder.decode(&packet) {
                Ok(audio_buf) => {
                    // Si es el primer paquete, creamos el SampleBuffer
                    if sample_buf.is_none() {
                        let spec = *audio_buf.spec();
                        let duration = audio_buf.capacity() as u64;
                        sample_buf = Some(SampleBuffer::<i16>::new(duration, spec));
                    }
                    // Alimentamos el buffer a Chromaprint
                    let sb = sample_buf.as_mut().unwrap();
                    sb.copy_interleaved_ref(audio_buf); // <- pasar audio_buf directamente
                    if !ctx.feed(sb.samples()) {
                        return Err(anyhow!("Chromaprint feed falló"));
                    }
                }
                Err(err) => {
                    // EOF o error irrelevante
                    use symphonia::core::errors::Error;
                    match err {
                        Error::IoError(_) | Error::DecodeError(_) => break,
                        _ => return Err(anyhow!("Error de decodificación: {}", err)),
                    }
                }
            },
            Err(_) => break, // EOF u otro error de lectura
        }
    }

    // 9. Finaliza y obtiene la huella en Base64
    if !ctx.finish() {
        return Err(anyhow!("Chromaprint finish falló"));
    }
    let fingerprint = ctx
        .fingerprint()
        .ok_or_else(|| anyhow!("No se pudo obtener fingerprint"))?;

    Ok(fingerprint)
}
