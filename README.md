This project consists of a mediocre chess computer and a mediocre chess gui that only
works with this chess computer.

# How to build & run

Both subprojects `/chessian` and `/gui` can be built using `cargo`, to launch
the chess gui running the computer, one could e. g. issue:

```bash
$ git clone https://github.com/***REMOVED***/chessian
$ cd chessian/gui
$ cargo run --release
```

# Features

1. Chess computer
    - correct and fast move generation using
      [a chess library](https://docs.rs/chess/latest/chess/)
    - alpha-beta pruning search using a full q-search to fully evaluate capture
      chains
    - Performance: Reaches depth 5–7 within 5 seconds on my machine, *before*
      performing the q-search also within the same 5 seconds
2. Chess gui
    - play chess against the computer or by yourself
    - automatically evaluate each position
    - control the computers strength
    - freely undo and redo moves
    - keyboard shortcuts:
        - `a` -> toggle auto response by computer
        - `f` -> print current FEN to stdout
        - `m` -> make the engine move
        - `ctrl+z` -> undo the last move
        - `ctrl+y` -> redo the last move
        - `s` -> toggle square names
        - `p` -> toggle pieces
        - `i` -> invert the board
        - `r` -> reset the game
        - `t` -> analyze the whole game
