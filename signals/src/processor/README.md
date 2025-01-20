# `signals/processor`

this module is worlds in blocks v2, where the result of said world is instantly calculated, because it was converted to a list of instructions beforehand.

this isn't documentation.

## notes

`input[n]` = result of `Instruction::SummonInput { id: n }`
`output[n]` = `memory[n]` at the end of execution

next up:

- implement foreigns for programification:

CHOOSE A PATH:

1. every programification will include all foreigns inside itself as if they were just in the world
2. foreigns will not be programified, their inputs will be calculated and a special instruction will be called to execute another world

path 1 is probably easier to implement but has two caveats:

if world A depends on world B and world B is changed, both of them need to be reinstructionified \
for highly nested worlds this will probably mean really really really long programs, taking up memory and taking long to regenerate

path 2 is harder to implement and requires designing and implementing a new instruction, but:

worlds will only have to be reprogramified when they change \
but memory is gonna have to be changed in order to allow another program to run (currently the compile-time stack makes this impossible in one segment of memory)

---

go ahead and think bout this one

path 1 is done but for high-complexity worlds we should probably do path 2 instead \
the good thing is we don't really have to hard choose one during development cause why not do both and use both of them for their advantages

but yeah we need the optimizer asap

imean i tried to just add processor blocks (actually just replace foreign functionality if and when possible) but game.rs was really made for foreigns and it didn't play well with processors

so maybe just do the optimizer before

cause all we need for that is a recursive eq function that returns a number representing how complex that eq is \
and we could hash eqs and for high-complexity eqs that appear in more than one places we could somehow store them and then we only have to calculate them once

although i have no clue how i'd add variables to the insanely pure functional eq type \
but yk we'll cross that bridge when we come to it

---

go ahead and think bout this one

---

go ahead and think bout this one

---

go ahead and think bout this one

---

go ahead and think bout this one

---

go ahead and think bout this one
