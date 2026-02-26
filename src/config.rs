use krilla::paint::Fill;
use krilla::color::rgb;
use krilla::num::NormalizedF32;

pub struct Dimensions {
    pub w: f32,
    pub h: f32,
}

#[derive(Clone, Copy)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}


impl From<Color> for Fill {
    fn from(value: Color) -> Self {
        Fill {
            paint: rgb::Color::new(value.r, value.g, value.b).into(),
            opacity: NormalizedF32::new(1.0).unwrap(),
            rule: Default::default(),
        }
    }
}

pub static PAGE_DIM: Dimensions = Dimensions { w: 800.0, h: 450.0 };

pub static BG: Color = Color {r: 100, g: 100, b: 100};
pub static FG: Color = Color {r: 0, g: 0, b: 0};
pub static ACCENT: Color = Color {r: 241, g: 241, b: 241};

pub static MARGIN: f32 = 0.2; // 10% of page width/height

pub static FONT_PATH: &str = "/usr/local/share/fonts/IBMPlexSans-Light.ttf";
pub static CODE_FONT_PATH: &str = "/usr/local/share/fonts/zed-mono-regular.ttf";
pub static FONT_SIZE: f32 = 36.;
