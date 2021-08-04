use std::rc::Rc;
use web_sys::HtmlImageElement;

use crate::engine::{Image, Point, SpriteSheet};
use crate::game::{Barrier, Obstacle, Platform};

pub fn rock_and_platform(
    stone: HtmlImageElement,
    sprite_sheet: Rc<SpriteSheet>,
    offset_x: i16,
) -> Vec<Box<dyn Obstacle>> {
    vec![
        Box::new(Barrier::new(Image::new(
            stone,
            Point {
                x: offset_x + 150,
                y: 546,
            },
        ))),
        Box::new(Platform::new(
            sprite_sheet.clone(),
            Point {
                x: offset_x + 200,
                y: 400,
            },
            vec![
                "13.png".to_string(),
                "14.png".to_string(),
                "15.png".to_string(),
            ],
        )),
    ]
}
