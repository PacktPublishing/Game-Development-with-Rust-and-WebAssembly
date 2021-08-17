use std::rc::Rc;
use web_sys::HtmlImageElement;

use crate::engine::{Image, Point, Rect, SpriteSheet};
use crate::game::{Barrier, Obstacle, Platform};

const LOW_PLATFORM: i16 = 420;
const HIGH_PLATFORM: i16 = 375;
const FIRST_PLATFORM: i16 = 370;

fn create_platform_bounding_boxes(destination_box: &Rect) -> Vec<Rect> {
    const X_OFFSET: i16 = 60;
    const END_HEIGHT: i16 = 54;
    let destination_box = destination_box;
    let bounding_box_one = Rect::new(destination_box.position, X_OFFSET, END_HEIGHT);

    let bounding_box_two = Rect::new_from_x_y(
        destination_box.x() + X_OFFSET,
        destination_box.y(),
        destination_box.width - (X_OFFSET * 2),
        destination_box.height,
    );

    let bounding_box_three = Rect::new_from_x_y(
        destination_box.x() + destination_box.width - X_OFFSET,
        destination_box.y(),
        X_OFFSET,
        END_HEIGHT,
    );

    vec![bounding_box_one, bounding_box_two, bounding_box_three]
}

pub fn rock_and_platform(
    stone: HtmlImageElement,
    sprite_sheet: Rc<SpriteSheet>,
    offset_x: i16,
) -> Vec<Box<dyn Obstacle>> {
    let mut platform = Platform::new(
        sprite_sheet.clone(),
        Point {
            x: offset_x + FIRST_PLATFORM,
            y: LOW_PLATFORM,
        },
        vec![
            "13.png".to_string(),
            "14.png".to_string(),
            "15.png".to_string(),
        ],
    );

    platform.set_bounding_boxes(create_platform_bounding_boxes(&platform.destination_box()));

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
                x: offset_x + FIRST_PLATFORM,
                y: LOW_PLATFORM,
            },
            vec![
                "13.png".to_string(),
                "14.png".to_string(),
                "15.png".to_string(),
            ],
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
