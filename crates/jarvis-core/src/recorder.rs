mod pvrecorder;

// mod cpal;
// mod portaudio;

use once_cell::sync::OnceCell;

use crate::{config, config::structs::RecorderType, DB};

static RECORDER_TYPE: OnceCell<RecorderType> = OnceCell::new();
static FRAME_LENGTH: OnceCell<u32> = OnceCell::new();

pub fn init() -> Result<(), ()> {
    // set default recorder type
    // @TODO. Make it configurable?
    RECORDER_TYPE.set(config::DEFAULT_RECORDER_TYPE).unwrap();

    // some info
    info!("Loading recorder ...");
    info!("Available audio_devices are:\n{:?}", get_audio_devices());

    // load given recorder
    match RECORDER_TYPE.get().unwrap() {
        RecorderType::PvRecorder => {
            // Init Pv Recorder
            info!("Initializing PvRecorder recording backend.");
            FRAME_LENGTH.set(512u32).unwrap(); // pvrecorder requires frame buffer of 512
            let selected_microphone = get_selected_microphone_index();
            match pvrecorder::init_microphone(
                selected_microphone,
                FRAME_LENGTH.get().unwrap().to_owned(),
            ) {
                false => {
                    error!("Recorder initialization failed.");

                    return Err(());
                }
                _ => {
                    info!(
                        "Recorder initialization success. Listening to microphone ({}): {}",
                        selected_microphone,
                        get_audio_device_name(selected_microphone)
                    );
                }
            }
        }
        RecorderType::PortAudio => {
            // Init PortAudio
            info!("Initializing PortAudio recording backend");
            error!("PortAudio backend not yet implemented"); return Err(());
            // match portaudio::init_microphone(get_selected_microphone_index(), FRAME_LENGTH.load(Ordering::SeqCst)) {
            //     false => {
            //         // Switch to PortAudio recorder
            //         error!("PortAudio audio backend failed.");
            //     },
            //     _ => ()
            // }
        }
        RecorderType::Cpal => {
            // Init CPAL
            info!("Initializing CPAL recording backend");
            error!("CPAL backend not yet implemented"); return Err(());
            // match cpal::init_microphone(get_selected_microphone_index(), FRAME_LENGTH.load(Ordering::SeqCst)) {
            //     false => {
            //         // Switch to CPAL recorder
            //         error!("CPAL audio backend failed.");
            //     },
            //     _ => ()
            // }
        }
    }

    Ok(())
}

pub fn read_microphone(frame_buffer: &mut [i16]) {
    match RECORDER_TYPE.get().unwrap() {
        RecorderType::PvRecorder => {
            pvrecorder::read_microphone(frame_buffer);
        }
        RecorderType::PortAudio => {
            frame_buffer.fill(0); // FIX: was todo!() — PortAudio not impl
            // portaudio::read_microphone(frame_buffer);
        }
        RecorderType::Cpal => {
            // cpal::read_microphone(frame_buffer);
            panic!("Cpal should be used via callback assignment");
        }
    }
}

pub fn start_recording() -> Result<(), ()> {
    match RECORDER_TYPE.get().unwrap() {
        RecorderType::PvRecorder => {
            return pvrecorder::start_recording(
                get_selected_microphone_index(),
                FRAME_LENGTH.get().unwrap().to_owned(),
            );
        }
        RecorderType::PortAudio => {
            return Err(()); // FIX: PortAudio start_recording not impl
            // portaudio::start_recording(get_selected_microphone_index(), FRAME_LENGTH.load(Ordering::SeqCst));
        }
        RecorderType::Cpal => {
            return Err(()); // FIX: CPAL start_recording not impl
            // cpal::start_recording(get_selected_microphone_index(), FRAME_LENGTH.load(Ordering::SeqCst));
        }
    }
}

pub fn stop_recording() -> Result<(), ()> {
    match RECORDER_TYPE.get().unwrap() {
        RecorderType::PvRecorder => pvrecorder::stop_recording(),
        RecorderType::PortAudio => {
            return Err(()); // FIX: PortAudio stop_recording not impl
            // portaudio::stop_recording();
        }
        RecorderType::Cpal => {
            return Err(()); // FIX: CPAL stop_recording not impl
            // cpal::stop_recording();
        }
    }
}

pub fn get_selected_microphone_index() -> i32 {
    let idx = DB.get().unwrap().read().microphone;

    if idx > 0 {
        // validate that this microphone is actually in the list
        let devices = get_audio_devices();
        if (idx as usize) >= devices.len() {
            warn!("Microphone index {} not found ({} available), falling back to default", 
                idx, devices.len());
            return -1;
        }
    }
    
    idx
}

pub fn get_audio_devices() -> Vec<String> {
    match RECORDER_TYPE.get() {
        Some(RecorderType::PvRecorder) => pvrecorder::list_audio_devices(),
        Some(RecorderType::PortAudio) => {
            return vec![]; // FIX: PortAudio list_audio_devices not impl
        }
        Some(RecorderType::Cpal) => {
            return vec![]; // FIX: CPAL list_audio_devices not impl
        }
        None => {
            // not initialized yet, default to pvrecorder
            pvrecorder::list_audio_devices()
        }
    }
}

pub fn get_audio_device_name(idx: i32) -> String {
    match RECORDER_TYPE.get() {
        Some(RecorderType::PvRecorder) => pvrecorder::get_audio_device_name(idx),
        Some(RecorderType::PortAudio) => {
            return String::new(); // FIX: PortAudio get_audio_device_name
        }
        Some(RecorderType::Cpal) => {
            return String::new(); // FIX: CPAL get_audio_device_name
        }
        None => {
            // not initialized yet, default to pvrecorder
            pvrecorder::get_audio_device_name(idx)
        }
    }
}
