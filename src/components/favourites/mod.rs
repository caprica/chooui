mod render;

pub(crate) struct FavouritesView {
    pub(crate) is_active: bool,
}

impl FavouritesView {
    pub(crate) fn new() -> Self {
        Self { is_active: false }
    }
}
