# Non-deterministic Minesweeper

Tired of having to guess in a game of minesweeper? Worry no more!

This is a variation of Minesweeper game where you only lose if the move is certainly wrong!
This turns the game into a 100% logic game with no luck involved, and makes any ratio of
mine/free space playable. This is a screenshot of a game with 50% mines where I went pretty
far:

![game with 50% mines](screenshot.png?raw=true "Ultra hard")

It pretty much works, but there is still a few things to be done.

## To do:

- Display why the player lost. This is not trivial in the non-deterministic
variant, as the game will have to prove it is impossible to find a mine
configuration that leaves the field in a consistent configuration.

- Obey the real probabilities of a given configuration to manifest. Right
now, when commiting to a given mine configuration as the player opens new
spaces, the chances are not consistent with uniform random distribution
of mines.

- Improve performance on the corner cases. This may not be always possible,
as the problem of finding if there is a valid mine configuration for a
given minesweeper field is NP-Hard. If you play normally as you
would do in a normal minesweeper game, the problem is still easy enough
that it does not interfere with the game play. But if you start doing crazy
things, expect to wait many minutes of 100% CPU usage for your move to complete.
