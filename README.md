# hanoi64

A just-for-fun Rust take on Tower of Hanoi: the entire game state lives in one 64-bit integer, and every move is a single XOR.

I built it in my spare time to see how compact I could make the state, and as a Rust exercise. Below is how it works and, more interesting than the puzzle itself, the trade-offs behind it.

## Usage

```sh
cargo run --release -- <rings> [-v | --verbose]
```

```text
$ cargo run --release -q -- 3 -v

------------------- STEPS -------------------
Ring 1 from peg 0 to peg 2
Ring 2 from peg 0 to peg 1
Ring 1 from peg 2 to peg 1
Ring 3 from peg 0 to peg 2
Ring 1 from peg 1 to peg 0
Ring 2 from peg 1 to peg 2
Ring 1 from peg 0 to peg 2
------------------ SOLVED!!! ----------------
Moved 3 rings from peg 0 to peg 2 in 7 moves (1.00µs).

$ cargo run --release -q -- 20
Moved 20 rings from peg 0 to peg 2 in 1048575 moves (5.73ms).
```

Towers can have 1 to 20 rings. Verbose mode is limited to 10 rings (see [below](#verbose-mode-stops-at-10-rings)). The library can also be used directly:

```rust
use hanoi64::TowerOfHanoi;

let mut tower = TowerOfHanoi::new(3);
tower.solve(false);
```

## How it works

### Rings on a peg are a set, not a stack

In a legal position, the rings on a peg are always ordered by size, largest at the bottom. So the set of rings on a peg already tells you the whole stack, and there's no need to store their order. Each peg becomes a bitmask: bit `r` is set when ring `r` is on that peg.

### The layout

```text
 bit 63    bits 62..=43    bit 42     bits 41..=22    bit 21     bits 20..=1      bit 0
[  0  ] [  20 ring bits  ] [  0  ] [  20 ring bits  ] [  0  ] [  20 ring bits  ] [  0  ]
   ^            ^             ^            ^             ^            ^             ^
   |            |             |            |             |            |             |
solved        peg 2      peg 2 guard     peg 1      peg 1 guard     peg 0      peg 0 guard
 flag
```

Three 21-bit slots, one per peg, each with a guard bit followed by 20 ring bits. Ring `r` on peg `p` is bit `p * 21 + r`, with ring 1 the smallest. Bit 63 records that the tower has been solved.

### What the operations cost

| Operation | How |
|---|---|
| Top ring of a peg | Shift the peg's slot down, then `trailing_zeros()` |
| Move a ring | One XOR with a two-bit mask: clears the ring's bit on the source peg and sets it on the destination |
| Count the rings | `count_ones()` on everything except the solved flag |
| Start state (`n` rings on peg 0) | `((1 << (n + 1)) - 1) ^ 1`, which sets bits `1..=n` |
| Check the final state | Compare the whole word with the start state shifted to peg 2 (`<< 42`) |

The solver is the classic recursive algorithm. The spare peg is `3 - from - to`, since the pegs are numbered 0, 1 and 2. Every move is checked against the rules, and the final state is checked as a whole, so a bug would panic rather than produce a wrong answer. The tests solve every tower size from 1 to 20.

## Trade-offs and choices

### The state isn't minimal, on purpose

Each ring is on one of 3 pegs, so 20 rings have 3²⁰ ≈ 3.5 billion possible states, which fit in about 32 bits. This layout uses 64, with 60 of them on rings. Here are the options I weighed:

| Encoding | Bits for 20 rings | Max rings in a `u64` | Top ring of a peg | Move |
|---|---|---|---|---|
| **One bit per ring per peg (this one)** | 60 + 4 spare | 20 | shift + `trailing_zeros` | 1 XOR |
| Two-bit peg number per ring | 40 | 32 | lane masks + `trailing_zeros` | clear and set a 2-bit field |
| Base-3 number | 32 | 40 | division and remainder per ring | add or subtract a power of 3 |

The two-bit encoding stays fast but is harder to read. Base-3 is the true minimum, but it turns cheap bit operations into arithmetic. One bit per ring per peg wastes about half the word, and in return every operation is one or two instructions and the code stays easy to follow. For this project, that was the better deal.

### 20 rings is plenty

Solving `n` rings takes 2ⁿ − 1 moves, so each extra ring doubles the work:

| Rings | Moves | Rough time at ~100M moves/s |
|---|---|---|
| 20 | ~1 million | milliseconds |
| 32 | ~4.3 billion | ~40 s |
| 40 | ~1.1 trillion | ~3 hours |
| 64 | ~1.8 × 10¹⁹ | ~6,000 years |

The extra rings a denser encoding could hold would rarely be worth waiting for. Running time is the real limit, not the number of bits.

### Why bother with less than 64 bits at all?

For one board, you wouldn't. A `u64` fits in a register, and narrower integers aren't any faster. Compactness pays off when you store *many* states, for example a visited set for a breadth-first search over every position. 3²⁰ states take about 28 GB at 8 bytes each, but only about 436 MB as a bit array indexed by the base-3 state number. That would be the reason to switch encodings, and this project never stores more than one state.

### Guard bits: convenience first, safety second

The guard bit at the bottom of each slot puts ring `r` at offset `r`, so `trailing_zeros()` returns the ring number directly. It also catches some corruption: a set guard bit makes `trailing_zeros()` return 0, which isn't a valid ring. The check is weak, though. A corrupt peg is reported the same way as an empty one, so a corrupt destination peg wouldn't be caught. The bits are there mainly to keep the slots aligned, and the check is a bonus.

### The solved flag is redundant

Whether the tower is solved can be computed from the ring bits. In fact, since moves never create or remove rings, "solved" means pegs 0 and 1 are empty, so testing the lower 42 bits is as cheap as reading a flag. I kept the flag so that `solve()` sets it only after verifying the final state. The cost is a bit that could disagree with the rings if a future method moved them without clearing it.

### Panics in the library, errors in the CLI

The library panics on impossible input (0 or more than 20 rings) and on rule violations, which would mean a bug in the solver. The command-line front end checks its arguments first, so a user only ever sees an error message and a non-zero exit code, never a panic.

### Verbose mode stops at 10 rings

Verbose mode prints one line per move, so the output doubles with every extra ring. At 10 rings it prints 1,023 lines, which still fits in a terminal and can be read. At 20 it would print over a million. Above 10, the program refuses with an error instead of quietly turning verbose off.

### Recursion is fine here

The recursion is at most 20 calls deep, so stack depth is never a concern. There's a neat alternative that needs no state at all: at step `k` (counting from 1), the ring that moves is `trailing_zeros(k) + 1`, and odd and even rings cycle around the pegs in opposite directions. I stuck with the classic version because the point here was the state, not the move order.

## Project layout

- [`src/lib.rs`](src/lib.rs): the `TowerOfHanoi` type, the bitmap and the solver, with the layout documented in detail.
- [`src/main.rs`](src/main.rs): the command-line front end.
- [`notebooks/hanoi_playground.ipynb`](notebooks/hanoi_playground.ipynb): a Python prototype with plain lists, where I sketched the game before writing it in Rust.

`cargo test` runs the unit tests and the doc example. `cargo doc --open --document-private-items` shows the full documentation, including the private bit helpers.

## Credits

I wrote the library code by hand. The documentation comments and the command-line front end were written with Claude Opus 5.5, and the commit history says which is which.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT), at your option.
