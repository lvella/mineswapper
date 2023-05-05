# Mineswapper: the non-deterministic minesweeper

![name origin](mineswapper.png?raw=true)

Tired of having to guess in a game of minesweeper? Worry no more!

This is a variation of Minesweeper game where you only lose if the move is
certainly wrong! This turns the game into a 100% logic game with no luck
involved, and makes any ratio of mine/free space playable. This is a screenshot
of a game with 50% mines where I went pretty far:

![game with 50% mines](screenshot.png?raw=true "Ultra hard")

In early beta stage: I don't really trust the correctness of the solver.

## How does it compares to other "never have to guess" minesweepers?

The game
[Mines](https://www.chiark.greenend.org.uk/~sgtatham/puzzles/js/mines.html) from
[Simon Tatham's Portable Puzzle
Collection](https://www.chiark.greenend.org.uk/~sgtatham/puzzles/) is also a
minesweeper variant where you never have to guess to complete a puzzle. There
are others. How does Mineswapper compares to them?

Well, they actually play differently. For the same size and number of mines,
Simon Tatham's Mines actually feels easier for experienced minesweeper players.
That is because that game is fully solved upon the first click, before being
presented to the player, and is rearranged until the solver can find a
configuration with no guesses needed. This means you will always play on a
curated board, where you will often be able to find familiar patterns that
leads to a solution.

Mineswapper feels harder at first, because the board is not curated, and like in
classic minesweeper, you will often get patterns that seems to have no certain
solution. But that is not really true in Mineswapper, and any possible solution
is a valid solution (at least until new clues are revealed). Once you learn to
take full advantage of this non-determinism, you might find Mineswapper actually
easier (but I am not so sure).

What is most interesting, though, is that in Mineswapper you can play with any
ratio of mines to free spaces. I guess the most challenging should be around 50%
of mine occupation, where you have to worry of a long chain of consequences when
committing to a move. This is in contrast to Simon Tatham's Mines, where a 50%
occupation will give you a very easy game where all mines had to be clustered
together outside the playing area, for manageable solvability.

## To do:

- Display why the player lost. This is not trivial in the non-deterministic
variant, as the game will have to prove it is impossible to find a mine
configuration that leaves the field in a consistent configuration.

- Obey the real probabilities of a given configuration to manifest. Right now,
when committing to a given mine configuration as the player opens new spaces, the
chances are not consistent with uniform random distribution of mines.

- Improve performance on the corner cases. This may not be always possible, as
the problem of finding if there is a valid mine configuration for a given
minesweeper field is NP-Hard. If you play normally as you would do in a normal
minesweeper game, the problem is still easy enough that it does not interfere
with the gameplay. But if you start doing crazy things, expect to wait many
minutes of 100% CPU usage for your move to complete.

- Web version using `iced_web`.

- Polish the interface.
  
- Scoreboard? Maybe online? Maybe use denuvo anti-cheat to prevent tampering
  with the scoreboard?