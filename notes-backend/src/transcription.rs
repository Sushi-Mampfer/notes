use sqlx::{query, Pool, Sqlite};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, install_logging_hooks};

// https://codeberg.org/tazz4843/whisper-rs/src/branch/master/examples/basic_use.rs
pub async fn transcribe(pool: Pool<Sqlite>, id: u32, file: String) {
    let samples: Vec<f32> = hound::WavReader::open(file)
        .unwrap()
        .into_samples::<f32>()
        .map(|x| x.unwrap())
        .collect();

    // load a context and model
    let ctx = WhisperContext::new_with_params(
        "ggml-large-v3-q5_0.bin",
        WhisperContextParameters {
            use_gpu: true,
            ..Default::default()
        },
    )
    .expect("failed to load model");
    // create a state attached to the model
    let mut state = ctx.create_state().expect("failed to create state");

    // the sampling strategy will determine how accurate your final output is going to be
    // typically BeamSearch is more accurate at the cost of significantly increased CPU time
    let mut params = FullParams::new(SamplingStrategy::BeamSearch {
        // whisper.cpp defaults to a beam size of 5, a reasonable default
        beam_size: 5,
        // this parameter is currently unused but defaults to -1.0
        patience: -1.0,
    });

    // and set the language to translate to as english
    params.set_language(Some("en"));

    // we also explicitly disable anything that prints to stdout
    // despite all of this you will still get things printing to stdout,
    // be prepared to deal with it
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    // we must convert to 16KHz mono f32 samples for the model
    // some utilities exist for this
    // note that you don't need to use these, you can do it yourself or any other way you want
    // these are just provided for convenience

    
    let samples = whisper_rs::convert_stereo_to_mono_audio(&samples)
        .expect("failed to convert audio data");

    let samples = resample_to_16khz(&samples);

    // now we can run the model
    state
        .full(params, &samples[..])
        .expect("failed to run model");

    // fetch the results
    let mut out = String::new();
    for segment in state.as_iter() {
        out.push_str(&format!(
            "[{} - {}]: {}",
            // these timestamps are in centiseconds (10s of milliseconds)
            segment.start_timestamp(),
            segment.end_timestamp(),
            // this default Display implementation will result in any invalid UTF-8
            // being converted into the Unicode replacement character, U+FFFD
            segment
        ));
    }
    query(
        r#"
        UPDATE entries 
        SET transcript = ?
        WHERE id = ?
    "#,
    )
    .bind(out)
    .bind(id)
    .execute(&pool)
    .await
    .unwrap();
}

// thanks claude
fn resample_to_16khz(samples: &[f32]) -> Vec<f32> {
    let ratio = 44100 as f64 / 16000.0;
    let new_len = (samples.len() as f64 / ratio) as usize;
    
    (0..new_len)
        .map(|i| {
            let pos = i as f64 * ratio;
            let idx = pos as usize;
            let frac = pos - idx as f64;
            
            if idx + 1 < samples.len() {
                // Linear interpolation
                samples[idx] * (1.0 - frac) as f32 + samples[idx + 1] * frac as f32
            } else {
                samples[idx]
            }
        })
        .collect()
}