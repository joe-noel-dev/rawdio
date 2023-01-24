#[derive(Clone, Copy)]
pub enum BiquadFilterType {
    HighPass,
    LowPass,
    BandPass,
    Notch,
    HighShelf,
    LowShelf,
}
