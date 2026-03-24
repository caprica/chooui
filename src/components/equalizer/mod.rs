mod event;
mod render;

pub(crate) struct EqualizerView {
    pub(crate) is_active: bool,
}

impl EqualizerView {
    pub(crate) fn new() -> Self {
        Self { is_active: false }
    }

    fn goto_next(&mut self) {}

    fn goto_previous(&mut self) {}

    fn goto_first(&mut self) {}

    fn goto_last(&mut self) {}

    fn adjust_amp(&mut self, delta: f64) {}
}
