# `signals/processor`

this module is worlds in blocks v2, where the result of said world is instantly calculated, because it was converted to a list of instructions beforehand.

more details are in code comments

## notes

`input[n]` = result of `Instruction::SummonInput { id: n }`
`output[n]` = `memory[n]` at the end of execution

the way it works is when executing, we only have two worry about two things:

- `Memory`: it's what it sounds like
- `Instructions`: the list of instructions, most likely a `Vec<Instruction>`, that generates all the outputs required

the complexity is mostly when generating said instructions

- `Equation`: a mathematical way to represent whether any block in a given world will resolve to true or false. the only variables in an `Equation` are the inputs

## the way it works

`world_to_instructions` has functions that handle all aspect of the world-to-instruction pipeline, so this part probably doesn't apply to you if that's all you need.

required steps for foreigns (and processor blocks once they're implemented):

- `Equation::map_foreigns`: recursively goes through every foreign in an equation tree and allows you to return any Equation in their place
- `Equation::map_inputs`: same thing but for inputs. you can combine these two to expand foreigns into regular everyday Equations

possible and recommended optimizations:

- `Equation::simplify`: simplifies the equation (duh) and solves it if it's just `Equation::Const`s
- `program::shared_recognititon`: recognizes equations that are calculated multiple times and makes it so they're calculated once and copied everywhere else they're needed

to turn `Equation`s into instructions:

- `Equation::to_insts`: pushes the list of instructions into the vec given. also does some light optimizations along the way (and recognition and such)
