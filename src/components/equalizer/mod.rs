mod event;
mod render;

const BANDS: usize = 18;
pub(crate) const MIN_AMP: f64 = -20.0;
pub(crate) const MAX_AMP: f64 = 20.0;
const AMP_STEP: f64 = 1.0;

/// Represents which equalizer control is currently selected.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum EqualizerSelection {
    Preamp,
    Band(usize),
}

pub(crate) struct EqualizerView {
    pub(crate) is_active: bool,
    selected: EqualizerSelection,
}

impl EqualizerView {
    pub(crate) fn new() -> Self {
        Self {
            is_active: false,
            selected: EqualizerSelection::Preamp,
        }
    }

    /// Returns the currently selected band index (0 = preamp, 1-18 = bands).
    pub(crate) fn selected_index(&self) -> usize {
        match self.selected {
            EqualizerSelection::Preamp => 0,
            EqualizerSelection::Band(i) => i + 1,
        }
    }

    /// Navigate to the next control (preamp -> band 0 -> band 1 -> ... -> band 17 -> preamp).
    fn goto_next(&mut self) {
        self.selected = match self.selected {
            EqualizerSelection::Preamp => EqualizerSelection::Band(0),
            EqualizerSelection::Band(i) if i < BANDS - 1 => EqualizerSelection::Band(i + 1),
            EqualizerSelection::Band(_) => EqualizerSelection::Preamp,
        };
    }

    /// Navigate to the previous control.
    fn goto_previous(&mut self) {
        self.selected = match self.selected {
            EqualizerSelection::Preamp => EqualizerSelection::Band(BANDS - 1),
            EqualizerSelection::Band(0) => EqualizerSelection::Preamp,
            EqualizerSelection::Band(i) => EqualizerSelection::Band(i - 1),
        };
    }

    /// Navigate to the first control (preamp).
    fn goto_first(&mut self) {
        self.selected = EqualizerSelection::Preamp;
    }

    /// Navigate to the last control (last band).
    fn goto_last(&mut self) {
        self.selected = EqualizerSelection::Band(BANDS - 1);
    }

    /// Adjust the currently selected amp by the given delta.
    /// Returns the new value after adjustment.
    fn adjust_amp(&mut self, delta: f64) -> f64 {
        // Note: This method doesn't persist the value - that's handled by the caller
        // via the Equalizer model. This is just for navigation state.
        delta
    }

    /// Get the current selection for rendering highlights.
    pub(crate) fn selection(&self) -> EqualizerSelection {
        self.selected
    }
}
