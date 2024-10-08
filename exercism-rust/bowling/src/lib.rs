#[derive(Debug, PartialEq)]
pub enum Error {
    NotEnoughPinsLeft,
    GameComplete,
}

#[derive(PartialEq, PartialOrd, Clone, Copy)]
enum SpareOrStrike {
    None,
    Spare,
    Strike,
}

#[derive(PartialEq, PartialOrd)]
enum Throw {
    First,
    Second,
    Third,
    FrameComplete,
}

pub struct BowlingGame {
    score: u16,
    frame: u8,
    pins_left: u16,
    throw: Throw,

    prev: SpareOrStrike,
    prev_prev: SpareOrStrike,
}

impl BowlingGame {
    pub fn new() -> Self {
        BowlingGame {
            score: 0,
            frame: 1,
            pins_left: 10,
            throw: Throw::First,
            prev: SpareOrStrike::None,
            prev_prev: SpareOrStrike::None,
        }
    }

    pub fn roll(&mut self, pins: u16) -> Result<(), Error> {
        if self.throw == Throw::FrameComplete {
            self.throw = Throw::First;
            self.frame += 1;
        }
        if self.frame > 10 {
            return Err(Error::GameComplete);
        }
        if self.pins_left < pins {
            return Err(Error::NotEnoughPinsLeft);
        }

        self.score += pins
            + if self.frame < 10 || self.throw < Throw::Third {
                0 + if self.prev >= SpareOrStrike::Spare
                    && (self.frame < 10 || self.throw == Throw::First)
                {
                    pins
                } else {
                    0
                } + if self.prev_prev == SpareOrStrike::Strike
                    && (self.frame < 10 || self.throw < Throw::Third)
                {
                    pins
                } else {
                    0
                }
            } else {
                0
            };

        let spare_or_strike = self.spare_or_strike(pins);
        assert!(!(spare_or_strike == SpareOrStrike::Spare && self.throw == Throw::First));

        self.throw = self.next_throw(pins, spare_or_strike);

        self.prev_prev = self.prev;
        self.prev = spare_or_strike;

        if self.frame < 10 {
            if self.throw < Throw::FrameComplete {
                self.pins_left -= pins;
            } else {
                self.pins_left = 10;
            }
        } else {
            if spare_or_strike == SpareOrStrike::None {
                self.pins_left -= pins;
            } else {
                self.pins_left = 10;
            }
        }
        Ok(())
    }

    fn spare_or_strike(&self, pins: u16) -> SpareOrStrike {
        if pins == 10 {
            SpareOrStrike::Strike
        } else if pins == self.pins_left {
            SpareOrStrike::Spare
        } else {
            SpareOrStrike::None
        }
    }

    fn next_throw(&self, pins: u16, spare_or_strike: SpareOrStrike) -> Throw {
        match self.throw {
            Throw::First => {
                if spare_or_strike == SpareOrStrike::Strike && self.frame < 10 {
                    Throw::FrameComplete
                } else {
                    Throw::Second
                }
            }
            Throw::Second => {
                if self.frame == 10
                    && (spare_or_strike == SpareOrStrike::Spare
                        || self.prev == SpareOrStrike::Strike)
                {
                    Throw::Third
                } else {
                    Throw::FrameComplete
                }
            }
            Throw::Third => {
                assert!(self.frame == 10);
                Throw::FrameComplete
            }
            Throw::FrameComplete => panic!(),
        }
    }

    pub fn score(&self) -> Option<u16> {
        return if self.frame < 10 {
            None
        } else if self.throw == Throw::FrameComplete {
            Some(self.score)
        } else {
            None
        };
    }
}
