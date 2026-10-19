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

    fn get_smallest_ring(&self, peg: u8) -> Option<u8> {
        debug_assert!(peg <= 2, "peg index out of bounds");
        let shift_bits = peg * 21;
        let peg_rings = (self.0 << (42 - shift_bits)) >> 42;
        let trailing = peg_rings.trailing_zeros() as u8;
        match trailing {
            x if x != 0 && x <= 20 => Some(x),
            _ => None,
        }
    }

    fn get_rings_num(&self) -> u8 {
        (self.0 << 1).count_ones() as u8
    }

    fn ring_to_bit(peg: u8, ring: u8) -> u8 {
        debug_assert!(peg <= 2, "peg index out of bounds");
        debug_assert!(ring > 0 && ring <= 20, "ring index out of bounds");
        peg * 21 + ring
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

    #[test]
    fn ring_to_bit_maps_correctly() {
        // Testing for ANY number of rings, on all pegs
        for x in 1u8..=20 {
            let rings = x;
            let mut tower = TowerOfHanoi::new(rings);
            // Peg 0
            //println!("Peg 0: {:064b}", tower.0);
            for r in (rings + 1)..20 {
                let b = TowerOfHanoi::ring_to_bit(0, r);
                let b_value = (tower.0 >> b) & 1;
                assert_eq!(b_value, 0);
            }
            for r in 1..=rings {
                let b = TowerOfHanoi::ring_to_bit(0, r);
                let b_value = (tower.0 >> b) & 1;
                assert_eq!(b_value, 1);
            }
            // Peg 1
            tower.0 <<= 21;
            //println!("Peg 1: {:064b}", tower.0);
            for r in (rings + 1)..20 {
                let b = TowerOfHanoi::ring_to_bit(1, r);
                let b_value = (tower.0 >> b) & 1;
                assert_eq!(b_value, 0);
            }
            for r in 1..=rings {
                let b = TowerOfHanoi::ring_to_bit(1, r);
                let b_value = (tower.0 >> b) & 1;
                assert_eq!(b_value, 1);
            }
            // Peg 2
            tower.0 <<= 21;
            //println!("Peg 2: {:064b}", tower.0);
            for r in (rings + 1)..20 {
                let b = TowerOfHanoi::ring_to_bit(2, r);
                let b_value = (tower.0 >> b) & 1;
                assert_eq!(b_value, 0);
            }
            for r in 1..=rings {
                let b = TowerOfHanoi::ring_to_bit(2, r);
                let b_value = (tower.0 >> b) & 1;
                assert_eq!(b_value, 1);
            }
        }
    }
}
