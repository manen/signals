# roadmap

## todo

- [x] overhaul `Tool::PlaceForeign`, only allow foreigns with the same `inst_id` to be next to each other
  - (i can pretty much just rewrite it from scratch, `IngameWorld::generate` exists idk why it's all manual)

### - display block

so some interactivity would be nice \
or at least a display

i want to make an 8 segment display

but first i'm thinking two new tools: copy and move

### - world customization && rendering overhaul

- [x] make scrollables only have scrollbars if they need to
- [x] on render, scrollables should clamp their scroll values

- new worlds_bar, with categories, named & colored worlds

- custom name, color
- inputs & outputs custom name, inputs custom color
- some block types should have an option to set output color

rendering overhaul

- proper block textures
- have wires adopt the color of wherever they're getting their signal from

### - prerelease

cleanups and stuff

- proper error block rendering

### - main menu

- a main menu
- game loading from any file
- game saving to any file
- that's pretty much it i think

### - cloud

- i know i know but allowing players to share their worlds with all the other players would be cool i think
- i'm not necessarily talking multiplayer, but world publishing should be a thing

### - ampap

as many platforms as possible

- so the `raylib` crate supports Windows, Linux, macOS, and web!!!
- a web client would be huge

i did a web client test and there's some differences between the platforms obviously \
mostly just mouse input being fucked up \
and i don't want the game to become a patchwork of `if WEB_BUILD { /* some obscure web unfucking code */ }` \
so yeah web build stays in post-1.0

to be able to do web nicely we'd need a central mouse input authority if that makes sense `note: using sui types in game rendering code might make this easier` \
like raylib has pressed, down, released and up \
and we should probably build our own version of that cause pressed and released straightup don't work on the web

and for some fucking reason the mouse x and y were locked to 640 by 480??? i have no clue why that makes any sense

i'll save the web experiment in the branch `web-experiment` just in case

### backburner

- `-`
  - give the user some way to turn binary numbers into decimal
  - like a tool that can be used to select blocks in a row and turn their on/off value into a number on the screen
  - or something like that

- `-`
  - for some reason `x.centered().clickable()` works but `x.clickable().centered()` doesn't

## 1.0

- [everything should work and it shouldn't be a hassle to use](#--prerelease)
- [main menu](#--main-menu)

## post-1.0

- [could](#--cloud)
- [ampap](#--ampap)
