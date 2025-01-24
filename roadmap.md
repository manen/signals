# roadmap

## todo

- overhaul `Tool::PlaceForeign`, only allow foreigns with the same `inst_id` to be next to each other
  - (i can pretty much just rewrite it from scratch, `IngameWorld::generate` exists idk why it's all manual)

### - world customization

- custom name, color
- inputs & outputs custom name, inputs custom color
- some block types should have an option to set output color
- ties into [rendering overhaul](#--rendering-overhaul)

### - rendering overhaul

- new worlds_bar, with categories, [named & colored worlds](#--world-customization)
- proper block textures
- have wires adopt the color of wherever they're getting their signal from

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

### backburner

- `-`
  - give the user some way to turn binary numbers into decimal
  - like a tool that can be used to select blocks in a row and turn their on/off value into a number on the screen
  - or something like that

## 1.0

- [better graphics (textures n allat)](#--rendering-overhaul)
- [main menu](#--main-menu)

## post-1.0

- [could](#--cloud)
- [ampap](#--ampap)
