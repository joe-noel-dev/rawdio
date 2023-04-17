/// The type of the biquad filter
///
/// This will determine how the parameters are used to set the coefficients
/// in the filter
#[derive(Clone, Copy)]
pub enum BiquadFilterType {
    /// High pass filter
    ///
    /// This removes low frequencies below the cutoff frequency
    HighPass,

    /// Low pass filter
    ///
    /// This removes high frequencies above the cutoff frequency
    LowPass,

    /// Band pass filter
    ///
    /// This removes all but a band around the centre frequency
    BandPass,

    /// Notch filter
    ///
    /// This removes a band around the centre frequency
    Notch,

    /// High shelf
    ///
    /// This applies a fixed gain to frequencies above the cutoff
    HighShelf,

    /// Low shelf
    ///
    /// This applies a fixed gain to frequencies below the cutoff
    LowShelf,
}
