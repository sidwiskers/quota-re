use skia_safe::Color;

pub struct Theme {
    pub bubble_bg: Color,
    pub text: Color,
    pub username: Color,
    pub reply_bg: Color,
    pub reply_bar: Color,
    pub reply_name: Color,
    pub reply_text: Color,
    pub avatar_border: Color,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            bubble_bg: Color::from_rgb(29, 30, 44),
            text: Color::from_rgb(224, 228, 240),
            username: Color::from_rgb(199, 101, 152),
            reply_bg: Color::from_argb(22, 255, 255, 255),
            reply_bar: Color::from_rgb(100, 130, 200),
            reply_name: Color::from_rgb(140, 170, 230),
            reply_text: Color::from_rgb(158, 173, 210),
            avatar_border: Color::from_rgb(33, 42, 73),
        }
    }

    pub fn light() -> Self {
        Self {
            bubble_bg: Color::from_rgb(218, 234, 255),
            text: Color::from_rgb(20, 25, 40),
            username: Color::from_rgb(22, 138, 205),
            reply_bg: Color::from_argb(18, 0, 0, 0),
            reply_bar: Color::from_rgb(80, 120, 200),
            reply_name: Color::from_rgb(60, 100, 200),
            reply_text: Color::from_rgb(80, 90, 120),
            avatar_border: Color::from_rgb(218, 234, 255),
        }
    }
}
