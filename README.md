# `signals`

silly signal game

## try out for yourself

release builds for windows and linux are compiled on every commit using github actions. \
[see actions](https://github.com/manen/signals/actions/)

## todo

- cache ingameworld processor runs
- foreign interactive overhaul:
  - overhaul `Tool::PlaceForeign`, only allow foreigns with the same `inst_id` to be next to each other
  - generate a random color based off of instructions
  - proper foreign rendering, just a big number with the inst id and the name of the component, background color, and a little processor icon if it's being processorized
- speaking of, worlds overhaul
  - worlds_bar shouldn't just be a bar, but rather a menu with categories, maybe allow for subcategories with the worlds inside
  - right-click and relative positional menus unfortunately
  - a `...` button on worlds, allowing for delete, rename, recategorize, etc

this is all worth writing down for now but i have plans

also plan for a 1.0 release cause i'll just keep refining everything until i get bored
