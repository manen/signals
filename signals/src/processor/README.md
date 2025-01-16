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
