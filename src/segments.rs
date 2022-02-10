use std::rc::Rc;
use web_sys::HtmlImageElement;

use crate::engine::{Image, Point, Rect, SpriteSheet};
use crate::game::{Barrier, Obstacle, Platform};

const LOW_PLATFORM: i16 = 420;
const HIGH_PLATFORM: i16 = 375;
const FIRST_PLATFORM: i16 = 370;

const FLOATING_PLATFORM_SPRITES: [&str; 3] = ["13.png", "14.png", "15.png"];
const PLATFORM_WIDTH: i16 = 384;
const PLATFORM_HEIGHT: i16 = 93;
const PLATFORM_EDGE_WIDTH: i16 = 60;
const PLATFORM_EDGE_HEIGHT: i16 = 54;
const FLOATING_PLATFORM_BOUNDING_BOXES: [Rect; 3] = [
    Rect {
        position: Point { x: 0, y: 0 },
        width: PLATFORM_EDGE_WIDTH,
        height: PLATFORM_EDGE_HEIGHT,
    },
    Rect {
        position: Point {
            x: PLATFORM_EDGE_WIDTH,
            y: 0,
        },
        width: PLATFORM_WIDTH - (PLATFORM_EDGE_WIDTH * 2),
        height: PLATFORM_HEIGHT,
    },
    Rect {
        position: Point {
            x: PLATFORM_WIDTH - PLATFORM_EDGE_WIDTH,
            y: 0,
        },
        width: PLATFORM_EDGE_WIDTH,
        height: PLATFORM_EDGE_HEIGHT,
    },
];

fn create_floating_platform(sprite_sheet: Rc<SpriteSheet>, position: Point) -> Platform {
    Platform::builder(sprite_sheet.clone(), position)
        .with_sprites(&FLOATING_PLATFORM_SPRITES)
        .with_bounding_boxes(&FLOATING_PLATFORM_BOUNDING_BOXES)
        .build()
}

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
        Box::new(create_floating_platform(
            sprite_sheet,
            Point {
                x: offset_x + FIRST_PLATFORM,
                y: LOW_PLATFORM,
            },
        )),
    ]
}

pub fn platform_and_rock(
    stone: HtmlImageElement,
    sprite_sheet: Rc<SpriteSheet>,
    offset_x: i16,
) -> Vec<Box<dyn Obstacle>> {
    vec![
        Box::new(Barrier::new(Image::new(
            stone,
            Point {
                x: offset_x + 400,
                y: 546,
            },
        ))),
        Box::new(create_floating_platform(
            sprite_sheet,
            Point {
                x: offset_x + 200,
                y: HIGH_PLATFORM,
            },
        )),
    ]
}
