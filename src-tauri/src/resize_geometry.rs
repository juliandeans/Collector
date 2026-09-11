//! Resize calculations use AppKit coordinates (origin at bottom left).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Frame {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Direction {
    pub x: i8,
    pub y: i8,
}

pub(crate) fn hit_test((x, y): (f64, f64), (width, height): (f64, f64)) -> Option<Direction> {
    if x < 0.0 || y < 0.0 || x > width || y > height {
        return None;
    }
    // Matches the visible 16pt grip inset 4pt from the bottom/right edges.
    if x >= width - 20.0 && x <= width - 4.0 && (4.0..=20.0).contains(&y) {
        return Some(Direction { x: 1, y: -1 });
    }
    // Wider corner targets remain reachable inside the rounded window shape.
    let corner_x = if x < 16.0 {
        -1
    } else if x > width - 16.0 {
        1
    } else {
        0
    };
    let corner_y = if y < 16.0 {
        -1
    } else if y > height - 16.0 {
        1
    } else {
        0
    };
    if corner_x != 0 && corner_y != 0 {
        return Some(Direction {
            x: corner_x,
            y: corner_y,
        });
    }
    let direction = Direction {
        x: if x < 6.0 {
            -1
        } else if x > width - 6.0 {
            1
        } else {
            0
        },
        y: if y < 6.0 {
            -1
        } else if y > height - 6.0 {
            1
        } else {
            0
        },
    };
    (direction.x != 0 || direction.y != 0).then_some(direction)
}

pub(crate) fn resize(
    start: Frame,
    direction: Direction,
    delta: (f64, f64),
    min: (f64, f64),
    max: (f64, f64),
) -> Frame {
    fn clamp(value: f64, min: f64, max: f64) -> f64 {
        let min = min.max(1.0);
        let max = if max > 0.0 { max.max(min) } else { f64::MAX };
        value.clamp(min, max)
    }
    let mut frame = start;
    if direction.x != 0 {
        frame.width = clamp(start.width + delta.0 * f64::from(direction.x), min.0, max.0);
        if direction.x < 0 {
            frame.x = start.x + start.width - frame.width;
        }
    }
    if direction.y != 0 {
        frame.height = clamp(
            start.height + delta.1 * f64::from(direction.y),
            min.1,
            max.1,
        );
        if direction.y < 0 {
            frame.y = start.y + start.height - frame.height;
        }
    }
    frame
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grip_and_all_edges_take_resize_but_content_keeps_background_drag() {
        for (point, direction) in [
            ((387.5, 12.5), Direction { x: 1, y: -1 }),
            ((2.0, 400.0), Direction { x: -1, y: 0 }),
            ((398.0, 400.0), Direction { x: 1, y: 0 }),
            ((200.0, 798.0), Direction { x: 0, y: 1 }),
            ((200.0, 2.0), Direction { x: 0, y: -1 }),
            ((8.0, 8.0), Direction { x: -1, y: -1 }),
            ((8.0, 792.0), Direction { x: -1, y: 1 }),
            ((392.0, 792.0), Direction { x: 1, y: 1 }),
        ] {
            assert_eq!(
                hit_test(point, (400.0, 800.0)),
                Some(direction),
                "{point:?}"
            );
        }
        for point in [(200.0, 400.0), (200.0, 35.0), (-1.0, 400.0), (401.0, 400.0)] {
            assert_eq!(hit_test(point, (400.0, 800.0)), None, "{point:?}");
        }
    }

    #[test]
    fn southeast_grip_keeps_top_left_fixed() {
        let start = Frame {
            x: 600.0,
            y: 279.0,
            width: 400.0,
            height: 800.0,
        };
        assert_eq!(
            resize(
                start,
                Direction { x: 1, y: -1 },
                (-40.0, 35.0),
                (1.0, 1.0),
                (2000.0, 2000.0)
            ),
            Frame {
                x: 600.0,
                y: 314.0,
                width: 360.0,
                height: 765.0
            }
        );
    }

    #[test]
    fn eight_directions_keep_the_opposite_edges_fixed() {
        let start = Frame {
            x: -600.0,
            y: -100.0,
            width: 400.0,
            height: 800.0,
        };
        for (x, y, expected) in [
            (
                -1,
                0,
                Frame {
                    x: -570.0,
                    y: -100.0,
                    width: 370.0,
                    height: 800.0,
                },
            ),
            (
                1,
                0,
                Frame {
                    x: -600.0,
                    y: -100.0,
                    width: 430.0,
                    height: 800.0,
                },
            ),
            (
                0,
                -1,
                Frame {
                    x: -600.0,
                    y: -80.0,
                    width: 400.0,
                    height: 780.0,
                },
            ),
            (
                0,
                1,
                Frame {
                    x: -600.0,
                    y: -100.0,
                    width: 400.0,
                    height: 820.0,
                },
            ),
            (
                -1,
                -1,
                Frame {
                    x: -570.0,
                    y: -80.0,
                    width: 370.0,
                    height: 780.0,
                },
            ),
            (
                1,
                -1,
                Frame {
                    x: -600.0,
                    y: -80.0,
                    width: 430.0,
                    height: 780.0,
                },
            ),
            (
                -1,
                1,
                Frame {
                    x: -570.0,
                    y: -100.0,
                    width: 370.0,
                    height: 820.0,
                },
            ),
            (
                1,
                1,
                Frame {
                    x: -600.0,
                    y: -100.0,
                    width: 430.0,
                    height: 820.0,
                },
            ),
        ] {
            assert_eq!(
                resize(
                    start,
                    Direction { x, y },
                    (30.0, 20.0),
                    (1.0, 1.0),
                    (2000.0, 2000.0)
                ),
                expected
            );
        }
    }

    #[test]
    fn size_constraints_do_not_shift_the_fixed_corner() {
        let start = Frame {
            x: 10.0,
            y: 20.0,
            width: 400.0,
            height: 800.0,
        };
        assert_eq!(
            resize(
                start,
                Direction { x: -1, y: -1 },
                (900.0, 900.0),
                (200.0, 300.0),
                (1000.0, 1000.0)
            ),
            Frame {
                x: 210.0,
                y: 520.0,
                width: 200.0,
                height: 300.0
            }
        );
        assert_eq!(
            resize(
                start,
                Direction { x: -1, y: -1 },
                (-900.0, -900.0),
                (200.0, 300.0),
                (500.0, 900.0)
            ),
            Frame {
                x: -90.0,
                y: -80.0,
                width: 500.0,
                height: 900.0
            }
        );
    }
}
