pub mod types{

    pub struct SpectrogramPoint {
        pub(crate) freq_bin: usize,
        pub(crate) magnitude: f32,
        pub(crate) time_idx: usize,
    }
    pub struct Fingerprint {
        pub(crate) hash: u64,
        pub(crate) time_offset: usize, // The absolute time of the anchor
    }


}