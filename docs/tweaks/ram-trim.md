# RAM trim

**What it is.** Calls `EmptyWorkingSet` once on every process gamebooster can open, except the
configured games. Their in-RAM pages move to the standby list or the page file. Free physical
memory is printed before and after. Off by default; there is nothing to undo.

**Windows default.** Windows trims working sets on its own when free memory runs low, and keeps
recently used pages cached (the standby list) because unused RAM is wasted RAM.

**Why it (sometimes) helps.** Only when memory is nearly full. BF6 can use 10 GB or more; on an
8 GB Mac, trimming before you load a map can avoid paging during a match. When RAM is not tight it
does nothing useful and can briefly hurt: the trimmed apps fault their pages back in, which causes
disk activity right when the game is loading.

**Exact setting.** `ram_trim = true` in the config.

## Try and undo by hand

Sysinternals [RAMMap](https://learn.microsoft.com/sysinternals/downloads/rammap): menu
Empty > Empty Working Sets does the same thing. There is no undo; pages come back as apps use them.

## Measure

- Task Manager > Performance > Memory: "In use" and "Available" before and after.
- Resource Monitor > Memory: "Hard Faults/sec" right after a trim shows the cost.
- Compare BF6 map load times and the first minute's frame times in CapFrameX with and without it
  ([how](README.md#how-to-measure-any-tweak)).
