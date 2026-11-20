#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell(pub i8, pub i8);

pub const COLUMNS: i8 = 7;
pub const ROWS: i8 = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mirror {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

impl Mirror {
    pub const ALL: [Self; 7] = [
        Self::A,
        Self::B,
        Self::C,
        Self::D,
        Self::E,
        Self::F,
        Self::G,
    ];

    /// 回傳面板格座標，從左上角的 (0, 0) 起向右、向下計數。
    pub const fn cell(self) -> Cell {
        match self {
            Self::A => Cell(2, 1),
            Self::B => Cell(2, 3),
            Self::C => Cell(3, 3),
            Self::D => Cell(5, 3),
            Self::E => Cell(6, 3),
            Self::F => Cell(5, 4),
            Self::G => Cell(6, 5),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Orientation {
    #[default]
    Clear,
    Slash,
    Backslash,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Signal {
    Amber,
    Blue,
}

impl Signal {
    pub const ALL: [Self; 2] = [Self::Amber, Self::Blue];

    /// 回傳左側發射燈的位置，位於網格外一格。
    pub const fn source(self) -> Cell {
        match self {
            Self::Amber => Cell(-1, 1),
            Self::Blue => Cell(-1, 4),
        }
    }

    /// 回傳接收窗的位置，位於網格外一格。
    pub const fn receiver(self) -> Cell {
        match self {
            Self::Amber => Cell(5, -1),
            Self::Blue => Cell(6, ROWS),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SignalBoard {
    orientations: [Orientation; Mirror::ALL.len()],
}

pub struct Route {
    pub points: Vec<Cell>,
    pub received: bool,
}

impl SignalBoard {
    /// 回傳指定鏡片的檔位。
    pub fn orientation(self, mirror: Mirror) -> Orientation {
        self.orientations[mirror as usize]
    }

    /// 將指定鏡片在直通、正斜線、反斜線三個檔位間循環。
    pub fn turn(&mut self, mirror: Mirror) {
        let slot = &mut self.orientations[mirror as usize];
        *slot = match *slot {
            Orientation::Clear => Orientation::Slash,
            Orientation::Slash => Orientation::Backslash,
            Orientation::Backslash => Orientation::Clear,
        };
    }

    /// 從發射燈沿格線追蹤光路，記錄轉折與離開網格的位置。
    pub fn trace(self, signal: Signal) -> Route {
        let source = signal.source();
        let mut points = vec![source];
        let (mut x, mut y) = (source.0 + 1, source.1);
        let (mut dx, mut dy) = (1, 0);
        for _ in 0..(i16::from(COLUMNS) * i16::from(ROWS) * 4 + 1) {
            if !(0..COLUMNS).contains(&x) || !(0..ROWS).contains(&y) {
                let end = Cell(x, y);
                points.push(end);
                return Route {
                    points,
                    received: end == signal.receiver(),
                };
            }
            if let Some(mirror) = Mirror::ALL
                .into_iter()
                .find(|mirror| mirror.cell() == Cell(x, y))
            {
                match self.orientation(mirror) {
                    Orientation::Clear => {}
                    Orientation::Slash => {
                        points.push(Cell(x, y));
                        (dx, dy) = (-dy, -dx);
                    }
                    Orientation::Backslash => {
                        points.push(Cell(x, y));
                        (dx, dy) = (dy, dx);
                    }
                }
            }
            x += dx;
            y += dy;
        }
        Route {
            points,
            received: false,
        }
    }

    /// 判定兩束光是否在同一組鏡片檔位下抵達各自的接收窗。
    pub fn is_aligned(self) -> bool {
        Signal::ALL
            .into_iter()
            .all(|signal| self.trace(signal).received)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_beams_travel_straight_and_each_mirror_cycles() {
        let mut board = SignalBoard::default();
        for signal in Signal::ALL {
            let route = board.trace(signal);
            assert_eq!(
                route.points,
                [signal.source(), Cell(COLUMNS, signal.source().1)]
            );
            assert!(!route.received);
        }
        for mirror in Mirror::ALL {
            for expected in [
                Orientation::Slash,
                Orientation::Backslash,
                Orientation::Clear,
            ] {
                board.turn(mirror);
                assert_eq!(board.orientation(mirror), expected);
            }
        }
        assert_eq!(board, SignalBoard::default());
    }

    #[test]
    fn two_receivers_share_exactly_one_solution() {
        use Orientation::{Backslash, Clear, Slash};
        let choices = [Clear, Slash, Backslash];
        let mut received = [0; 3];
        for mut code in 0..3_usize.pow(Mirror::ALL.len() as u32) {
            let board = SignalBoard {
                orientations: Mirror::ALL.map(|_| {
                    let choice = choices[code % 3];
                    code /= 3;
                    choice
                }),
            };
            let [amber, blue] = Signal::ALL.map(|signal| board.trace(signal).received);
            received[0] += usize::from(amber);
            received[1] += usize::from(blue);
            if amber && blue {
                received[2] += 1;
                assert_eq!(
                    board.orientations,
                    [Backslash, Backslash, Clear, Slash, Backslash, Slash, Clear]
                );
            }
        }
        assert_eq!(received, [27, 27, 1]);
    }
}
