// Layout
//  bit 63    bits 62..=43    bit 42     bits 41..=22    bit 21     bits 20..=1      bit 0
// [  0  ] [  20 ring bits  ] [  0  ] [  20 ring bits  ] [  0  ] [  20 ring bits  ] [  0  ]
//    ^            ^             ^            ^             ^            ^             ^
//    |            |             |            |             |            |             |
// solved        peg 2      peg 2 guard     peg 1      peg 1 guard     peg 0      peg 0 guard
//  flag
pub struct TowerOfHanoi(u64);

impl TowerOfHanoi {
    pub fn new(number_of_rings: u8) -> Self {
        match number_of_rings {
            0 => panic!("Number of rings must be greater than 0"),
            x if x > 20 => panic!("Max 20 rings allowed (requested: {})", x),
            _ => TowerOfHanoi(Self::full_tower(number_of_rings)),
        }
    }

    fn full_tower(rings: u8) -> u64 {
        debug_assert!(
            rings > 0,
            "full_tower requires at least 1 ring, got {rings}"
        );
        ((1 << (rings + 1)) - 1) ^ 1
    }

    fn get_solved(&self) -> bool {
        self.0 >> 63 == 1
    }

    fn set_solved(&mut self) {
        self.0 |= 1 << 63;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Number of rings must be greater than 0")]
    fn new_panics_with_zero_rings() {
        TowerOfHanoi::new(0);
    }

    #[test]
    #[should_panic(expected = "Max 20 rings allowed")]
    fn new_panics_with_more_than_20_rings() {
        TowerOfHanoi::new(21);
    }

    #[test]
    fn new_works_with_any_1_to_20_rings() {
        for rings in 1..=20 {
            TowerOfHanoi::new(rings);
        }
    }
}
