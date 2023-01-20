#[derive(Clone, Copy)]
pub enum FilterType {
    HighPass,
    LowPass,
    BandPass,
    Notch,
    HighShelf,
    LowShelf,
}
