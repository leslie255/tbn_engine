use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Rgba {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub const fn to_array_bgra(self) -> [f32; 4] {
        [self.b, self.g, self.r, self.a]
    }

    /// Map all 4 channels.
    #[inline(always)]
    pub fn map(self, mut f: impl FnMut(f32) -> f32) -> Self {
        Self::new(f(self.r), f(self.g), f(self.b), f(self.a))
    }

    /// Map the RGB channels, do nothing on the alpha channel.
    #[inline(always)]
    pub fn map_rgb(self, mut f: impl FnMut(f32) -> f32) -> Self {
        Self::new(f(self.r), f(self.g), f(self.b), self.a)
    }

    #[inline(always)]
    pub fn apply_gamma(self, gamma: f32) -> Self {
        self.map_rgb(|x| x.powf(gamma))
    }

    pub fn linear_to_srgb(self) -> Self {
        self.apply_gamma(2.2).to_array().into()
    }

    pub fn srgb_to_linear(self) -> Self {
        self.apply_gamma(1.0 / 2.2).to_array().into()
    }
}

impl From<[f32; 4]> for Rgba {
    fn from(value: [f32; 4]) -> Self {
        Self::new(value[0], value[1], value[2], value[3])
    }
}

impl From<(f32, f32, f32, f32)> for Rgba {
    fn from(value: (f32, f32, f32, f32)) -> Self {
        Self::new(value.0, value.1, value.2, value.3)
    }
}

impl From<Rgba> for [f32; 4] {
    fn from(value: Rgba) -> Self {
        value.to_array()
    }
}

impl From<Rgba> for (f32, f32, f32, f32) {
    fn from(value: Rgba) -> Self {
        (value.r, value.g, value.b, value.a)
    }
}

