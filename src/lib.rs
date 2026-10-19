//! A Tower of Hanoi solver whose whole game state fits in a single `u64`.
//!
//! The puzzle has three pegs (numbered `0`, `1` and `2`) and up to 20 rings.
//! Rings are numbered by size, from `1` (the smallest) to `n` (the largest).
//! The tower starts with every ring on peg `0` and is solved once every ring
//! has been moved to peg `2`. Only the top ring of a peg can be moved, and a
//! ring may never sit on a smaller one.
//!
//! See [`TowerOfHanoi`] for how the state is packed into the bitmap.
//!
//! # Example
//!
//! ```
//! use hanoi64::TowerOfHanoi;
//!
//! let mut tower = TowerOfHanoi::new(3);
//! tower.solve(true); // prints the 7 moves, then "SOLVED!!!"
//! ```

/// The full state of a Tower of Hanoi game, packed into one 64-bit bitmap.
///
/// # Bitmap layout
///
/// The 64 bits are split into three 21-bit peg slots plus one flag bit:
///
/// ```text
///  bit 63    bits 62..=43    bit 42     bits 41..=22    bit 21     bits 20..=1      bit 0
/// [  0  ] [  20 ring bits  ] [  0  ] [  20 ring bits  ] [  0  ] [  20 ring bits  ] [  0  ]
///    ^            ^             ^            ^             ^            ^             ^
///    |            |             |            |             |            |             |
/// solved        peg 2      peg 2 guard     peg 1      peg 1 guard     peg 0      peg 0 guard
///  flag
/// ```
///
/// - **Ring bits.** Each peg has one bit per possible ring. Ring `r` sits on
///   peg `p` exactly when bit `p * 21 + r` is set (see
///   `ring_to_bit`). Every ring is on exactly one peg,
///   so for each ring exactly one of its three bits is set.
/// - **Guard bits** (bits `0`, `21` and `42`). The lowest bit of each slot
///   is never set. It is padding that does two jobs:
///   - Ring `r` sits at offset `r` inside its slot, not `r - 1`. Once a slot
///     is shifted down to bit 0, its `trailing_zeros()` is the number of the
///     smallest ring on that peg, with no adjustment.
///   - It works as a sanity check. If a guard bit is set, `trailing_zeros()`
///     returns `0`, which is not a valid ring number, so the state is
///     rejected instead of being misread (see
///     `get_smallest_ring`).
/// - **Solved flag** (bit `63`). It is set by [`solve`](Self::solve) only
///   after the final state has been checked.
///
/// Within a peg, a lower bit means a smaller ring. A peg's rings always form
/// a valid stack (smaller on top of larger), so the peg's top ring is its
/// lowest set bit.
///
/// # Notable states
///
/// For a tower of `n` rings:
///
/// - The start state is `full_tower(n)`, which has bits
///   `1..=n` set: all rings on peg 0.
/// - The solved state is `full_tower(n) << 42`, which has bits `43..=42 + n`
///   set: all rings on peg 2. After the check passes, bit 63 is set as well.
///
/// For example, with 3 rings the start state is
/// `0b1110` (bits 1, 2 and 3), and the target state before the solved flag is
/// set is `0b1110 << 42`.
pub struct TowerOfHanoi(u64);

impl TowerOfHanoi {
    /// Creates a new, unsolved tower with all `number_of_rings` rings stacked on
    /// peg 0.
    ///
    /// # Panics
    ///
    /// Panics if `number_of_rings` is `0` or greater than `20`, the number of
    /// ring bits in each peg slot.
    pub fn new(number_of_rings: u8) -> Self {
        match number_of_rings {
            0 => panic!("Number of rings must be greater than 0"),
            x if x > 20 => panic!("Max 20 rings allowed (requested: {})", x),
            _ => TowerOfHanoi(Self::full_tower(number_of_rings)),
        }
    }

    /// Puts the tower back in its start state: all rings on peg 0, solved flag
    /// cleared.
    ///
    /// With `Some(n)`, the tower is rebuilt with `n` rings. With `None`, it
    /// keeps its current number of rings, as counted by
    /// `get_rings_num`.
    ///
    /// # Panics
    ///
    /// Panics under the same conditions as [`new`](Self::new), that is, if
    /// `Some(0)` or `Some(n)` with `n > 20` is passed.
    pub fn reset(&mut self, number_of_rings: Option<u8>) {
        *self = Self::new(number_of_rings.unwrap_or(self.get_rings_num()));
    }

    /// Solves the puzzle by moving every ring from peg 0 to peg 2, then sets the
    /// solved flag.
    ///
    /// A tower of `n` rings takes `2^n - 1` moves, which is about one million
    /// for the maximum of 20 rings. When `verbose` is `true`, each move is
    /// printed to stdout, between a header and a closing banner.
    ///
    /// If the tower is already solved (bit 63 is set), this does nothing,
    /// apart from printing a notice when `verbose` is `true`.
    ///
    /// # Invariant checks
    ///
    /// Every single move is checked by
    /// `move_top_ring`. Once the recursion ends, the
    /// whole bitmap must equal `full_tower(n) << 42`. That one comparison
    /// checks that every ring is on peg 2, that pegs 0 and 1 are empty, and
    /// that no guard bit or solved flag was set by mistake. The solved flag is
    /// set only after this check passes.
    ///
    /// # Panics
    ///
    /// Panics if an illegal move is attempted, or if the final state is not
    /// the expected solved state. Neither should happen when starting from a
    /// state built by [`new`](Self::new) or [`reset`](Self::reset).
    pub fn solve(&mut self, verbose: bool) {
        if self.get_solved() {
            if verbose {
                println!("Already solved");
            }
            return;
        }
        if verbose {
            println!("\n------------------- STEPS -------------------");
        }
        self.move_rings(self.get_rings_num(), 0, 2, verbose);
        if self.0 == Self::full_tower(self.get_rings_num()) << 42 {
            self.set_solved();
            if verbose {
                println!("------------------ SOLVED!!! ----------------");
            }
        } else {
            panic!("Solver error: solution not reached!");
        }
    }

    /// Moves the top (smallest) ring of peg `from` onto peg `to`, enforcing the
    /// rules of the game.
    ///
    /// The move is legal when `from` has at least one ring and `to` is either
    /// empty or has a larger ring on top. The two pegs cannot share the same
    /// top ring, because every ring is on exactly one peg.
    ///
    /// A legal move is applied with one XOR against a mask holding two bits:
    /// the ring's bit on `from`, which is known to be set, and its bit on
    /// `to`, which is known to be clear. The XOR clears the first and sets the
    /// second in a single step.
    ///
    /// # Panics
    ///
    /// Panics if `from` has no rings (or its guard bit is set), or if the move
    /// would put a larger ring on top of a smaller one.
    fn move_top_ring(&mut self, from: u8, to: u8, verbose: bool) {
        let Some(from_bit) = self.get_smallest_ring(from) else {
            panic!("Invalid move: no rings on the source peg {from}");
        };
        let lowest_to_bit = self.get_smallest_ring(to);
        match lowest_to_bit {
            Some(x) if x < from_bit => {
                panic!(
                    "Invalid move: attempting to stack ring {} onto {}",
                    from_bit, x
                );
            }
            _ => {
                if verbose {
                    println!("Ring {} from peg {from} to peg {to}", from_bit);
                }
                let mask: u64 = (1 << Self::ring_to_bit(from, from_bit))
                    | (1 << Self::ring_to_bit(to, from_bit));
                self.0 ^= mask;
            }
        }
    }

    /// Moves the top `number` rings from peg `from` to peg `to`, using the
    /// classic recursive algorithm.
    ///
    /// To move `number` rings, it first moves the `number - 1` rings above the
    /// largest one onto the third peg, called the buffer. It then moves the
    /// largest ring to `to`, and finally moves the `number - 1` rings from the
    /// buffer onto `to`. Since the pegs are numbered `0`, `1` and `2`, which add
    /// up to `3`, the buffer is always `3 - from - to`.
    ///
    /// `number` must be at least `1`, and `from` and `to` must be two different
    /// pegs.
    fn move_rings(&mut self, number: u8, from: u8, to: u8, verbose: bool) {
        if number > 1 {
            let buffer = 3 - from - to;
            self.move_rings(number - 1, from, buffer, verbose);
            self.move_top_ring(from, to, verbose);
            self.move_rings(number - 1, buffer, to, verbose);
        } else {
            self.move_top_ring(from, to, verbose);
        }
    }

    /// Returns the bitmap of a full stack of `rings` rings on peg 0, with bits
    /// `1..=rings` set.
    ///
    /// `(1 << (rings + 1)) - 1` sets bits `0..=rings`. The final `^ 1` clears
    /// bit 0, the guard bit of peg 0. Shift the result left by `21 * p` to get
    /// the same stack on peg `p`. For example, `full_tower(n) << 42` is the
    /// solved state.
    ///
    /// `rings` must be in `1..=20`. Only the lower bound is checked here, and
    /// only in debug builds. [`new`](Self::new) enforces both bounds.
    fn full_tower(rings: u8) -> u64 {
        debug_assert!(
            rings > 0,
            "full_tower requires at least 1 ring, got {rings}"
        );
        ((1 << (rings + 1)) - 1) ^ 1
    }

    /// Returns `true` if the solved flag (bit 63) is set.
    fn get_solved(&self) -> bool {
        self.0 >> 63 == 1
    }

    /// Sets the solved flag (bit 63). The ring bits are left untouched.
    fn set_solved(&mut self) {
        self.0 |= 1 << 63;
    }

    /// Returns the number of the smallest ring on `peg`, which is its top ring,
    /// or `None` if the peg is empty.
    ///
    /// The peg's slot is isolated with two shifts. Shifting left by
    /// `42 - 21 * peg` drops every bit above the slot, and shifting right by
    /// `42` then brings the slot's guard bit down to bit 0. What is left is a
    /// 22-bit window:
    ///
    /// ```text
    ///  bit 21                      bits 20..=1          bit 0
    /// [ next slot's guard bit ] [ peg's ring bits ] [ peg's guard bit ]
    /// ```
    ///
    /// Bit 21 of the window is the guard bit of the next peg up, or the solved
    /// flag when `peg == 2`. Thanks to the guard bit at offset 0, the window's
    /// `trailing_zeros()` is the number of the smallest ring. Only `1..=20`
    /// counts as a ring. Every other value means there is no ring to report:
    ///
    /// - `0`: the peg's own guard bit is set, which means the state is corrupt.
    /// - `21`: the ring bits are empty, but the bit above the slot is set (for
    ///   example, peg 2 is empty and the solved flag is on).
    /// - `64`: the whole window is zero, so the peg is empty.
    ///
    /// `peg` must be `0`, `1` or `2`. This is checked in debug builds only.
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

    /// Returns the number of rings in the game, across all pegs.
    ///
    /// Shifting left by one drops the solved flag (bit 63). The remaining set
    /// bits are all ring bits, because the guard bits are always clear and
    /// every ring is on exactly one peg.
    fn get_rings_num(&self) -> u8 {
        (self.0 << 1).count_ones() as u8
    }

    /// Returns the bitmap index of ring `ring` on peg `peg`: `peg * 21 + ring`.
    ///
    /// Each peg's slot is 21 bits wide (one guard bit plus 20 ring bits), so
    /// `peg * 21` is the slot's guard bit. Because rings are numbered from 1,
    /// ring `ring` is at offset `ring` above it. The result is always in
    /// `1..=62`, so it never lands on a guard bit or on the solved flag.
    ///
    /// `peg` must be `0`, `1` or `2`, and `ring` must be in `1..=20`. Both
    /// are checked in debug builds only.
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
    fn get_solved_and_set_solved_both_use_the_highest_bit() {
        let mut tower = TowerOfHanoi::new(5);
        let solved_before = tower.get_solved();
        tower.set_solved();
        let solved_after = tower.get_solved();
        assert!(!solved_before);
        assert!(
            solved_after,
            "Solved state not modified by set_solved(): {:064b}",
            tower.0
        );
    }

    /// Builds a full tower on peg 0, then shifts it by one 21-bit slot at a
    /// time to place it on peg 1 and then on peg 2. On each peg, it checks
    /// that `ring_to_bit` points to set bits for rings `1..=rings` and to
    /// clear bits for the rings above them.
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

    /// Solves towers of every allowed size. `solve` panics if any move is
    /// illegal or if the final state is wrong, so the test passes as long as
    /// nothing panics.
    #[test]
    fn tower_is_correctly_solved() {
        // Test for ANY number of rings
        for rings in 1u8..=20 {
            let mut tower = TowerOfHanoi::new(rings);
            tower.solve(false);
        }
    }
}
