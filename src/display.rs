use embedded_graphics::mono_font::{ascii::FONT_10X20, MonoTextStyleBuilder};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::text::Text;
use esp_idf_hal::i2c::*;
use profont::PROFONT_24_POINT;
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, Ssd1306};

pub fn show_welcome(
    display: &mut Ssd1306<
        I2CInterface<I2cDriver>,
        DisplaySize128x32,
        BufferedGraphicsMode<DisplaySize128x32>,
    >,
) {
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(BinaryColor::On)
        .build();

    let _ = Text::new("^._.^", Point::new(0, 20), text_style).draw(display);

    display.flush().unwrap();
}

pub fn display_update_with_text(
    display: &mut Ssd1306<
        I2CInterface<I2cDriver>,
        DisplaySize128x32,
        BufferedGraphicsMode<DisplaySize128x32>,
    >,
    text_to_display: &str,
) {
    let text_style_big = MonoTextStyleBuilder::new()
        .font(&PROFONT_24_POINT)
        .text_color(BinaryColor::On)
        .build();

    let _ = display.clear(BinaryColor::Off);

    let _ = Text::new(&text_to_display, Point::new(24, 28), text_style_big).draw(display);

    display.flush().unwrap();
}
