use std::sync::{Arc, Mutex};

const BANDS: usize = 18;

const MIN_AMP: f64 = -20.0;
const MAX_AMP: f64 = 20.0;

pub(crate) struct Equalizer {
    pub(crate) amps: Arc<Mutex<Amps>>,
}

pub(crate) struct Amps {
    pub(crate) preamp: f64,
    pub(crate) gains: [f64; 18],
}

impl Equalizer {
    pub(crate) fn new() -> Self {
        Self {
            amps: Arc::new(Mutex::new(Amps {
                preamp: 0.0,
                gains: [0.0; BANDS],
            })),
        }
    }

    pub(crate) fn set_amps(&mut self, amps: Amps) {
        let mut lock = self.amps.lock().unwrap();
        *lock = amps;
    }

    pub(crate) fn amps(&self) -> Arc<Mutex<Amps>> {
        Arc::clone(&self.amps)
    }

    pub(crate) fn preamp_updated(&self, value: f64) {
        Equalizer::validate_amp(value);

        let mut amps = self.amps.lock().unwrap();
        amps.preamp = value;
    }

    pub(crate) fn amp_updated(&self, index: usize, value: f64) {
        Equalizer::validate_band(index);
        Equalizer::validate_amp(value);

        let mut amps = self.amps.lock().unwrap();
        amps.gains[index] = value;
    }

    fn validate_amp(value: f64) {
        if !(MIN_AMP..=MAX_AMP).contains(&value) {
            panic!("Amp value {} is out of valid range", value);
        }
    }

    fn validate_band(index: usize) {
        if index >= BANDS {
            panic!("Band index {} is out of valid range", index);
        }
    }
}
