# npuzzle

A solver for the npuzzle problem using A*. To print usage:

```
cargo run -- --help
```

An example in action:
```
cargo run -- -h nilsson -a 4
```
![4x4 solution with nillson heuristic](assets/4x4.gif)

It's also playable as a game: use
```
cargo run -- -a n -m
```
to solve a randomly generated nxn board yourself, using the arrow keys to move.

Several different heuristics are available; nilsson and the custom heuristic are both inadmissible and will work much faster on a 4x4 and are the only practical way to finish a 5x5.
