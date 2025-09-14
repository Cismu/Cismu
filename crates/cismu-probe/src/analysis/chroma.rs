use bliss_audio::{
    BlissError,
    decoder::{Decoder, ffmpeg::FFmpegDecoder},
};

use chromaprint::Chromaprint;
use std::path::Path;
use thiserror::Error;

/// Frecuencia de muestreo que `bliss_audio` usa internamente para todos los
/// decodificadores. Debe coincidir con la constante privada `SAMPLE_RATE`
/// de `bliss_audio`. Si la librería cambia este valor en el futuro,
/// actualiza esta constante para evitar fingerprints erróneos.
const BLISS_SAMPLE_RATE: i32 = 22050;

#[derive(Error, Debug, Clone)]
pub enum ChromaprintError {
    #[error("An error occurred while decoding the file.")]
    DecoderError(#[from] BlissError),
    #[error("Chromaprint start failed")]
    StartFailed,
    #[error("Chromaprint feed failed")]
    FeedFailed,
    #[error("Chromaprint finish failed")]
    FinishFailed,
    #[error("The fingerprint could not be obtained.")]
    FingerprintError,
}

/// Convierte una muestra en `f32` del rango [-1.0, 1.0] al rango PCM16 [-32768, 32767].
///
/// Se usa `clamp(-1.0, 1.0)` para asegurar que cualquier pico fuera del rango normalizado
/// (por ejemplo 1.0001 debido a decodificación) no provoque saturación ni overflow al
/// convertir a `i16`. Esta conversión es la que espera Chromaprint.
#[inline]
fn f32_to_i16(s: f32) -> i16 {
    (s.clamp(-1.0, 1.0) * 32767.0) as i16
}

/// Calcula la huella acústica Chromaprint de un archivo de audio.
///
/// 1. Decodifica el archivo completo usando `FFmpegDecoder` de `bliss_audio`,
///    que devuelve muestras `f32` mono a 22050 Hz.
/// 2. Inicializa un contexto Chromaprint con los parámetros adecuados.
/// 3. Convierte las muestras `f32` a PCM16 `i16` usando [`f32_to_i16`].
/// 4. Alimenta Chromaprint con todas las muestras y obtiene la huella.
///
/// # Errores
/// Devuelve un `ChromaprintError` si falla cualquiera de las fases anteriores.
///
/// # Notas
/// - Si `bliss_audio` cambia su frecuencia de muestreo, actualiza `BLISS_SAMPLE_RATE`.
/// - Chromaprint admite `feed` en bloques, pero aquí se envía todo de una vez para simplicidad.
pub fn fingerprint_from_file<P: AsRef<Path>>(path: P) -> Result<String, ChromaprintError> {
    // 1) Decode + resample (mono, 22050 Hz, f32[-1,1])
    let song = FFmpegDecoder::decode(path.as_ref())?;
    if song.sample_array.is_empty() {
        return Err(ChromaprintError::FingerprintError);
    }

    // 2) Inicia Chromaprint con parámetros estándar de bliss
    let mut ctx = Chromaprint::new();
    let channels = 1;
    if !ctx.start(BLISS_SAMPLE_RATE, channels) {
        return Err(ChromaprintError::StartFailed);
    }

    // 3) Convierte f32 -> i16 con clamp rápido + prealocación
    let mut samples_i16 = Vec::<i16>::with_capacity(song.sample_array.len());
    samples_i16.extend(song.sample_array.iter().copied().map(f32_to_i16));

    // 4) Alimenta a Chromaprint con todas las muestras de una vez
    if !ctx.feed(&samples_i16) {
        return Err(ChromaprintError::FeedFailed);
    }

    // (Opcional) liberar RAM del buffer antes del finish
    drop(samples_i16);

    // 5) Finaliza y obtiene la huella
    if !ctx.finish() {
        return Err(ChromaprintError::FinishFailed);
    }

    ctx.fingerprint().ok_or(ChromaprintError::FingerprintError)
}
