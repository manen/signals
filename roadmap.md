# roadmap

## todo

- [x] overhaul `Tool::PlaceForeign`, only allow foreigns with the same `inst_id` to be next to each other
  - (i can pretty much just rewrite it from scratch, `IngameWorld::generate` exists idk why it's all manual)

### - world customization

this is next up, but first i want to work on `sui` a little more since sure components are cool but there's still a lot to improve upon:

- the event returned from `Layable::pass_event` should be able to be any type. constraints:
  - the returned event type should be constant across a project
  - so i'm thinking a `<T>` in clickable, that sets an `Event` associated type in Layable
  - other layables further up the component tree will adopt `L::Event`
  - for components with more than 1 generic Layable types, it should only work if `L1::Event` == `L2::Event` (the other option is an enum and no thank you) (although we could have some way to make `L1` and `L2` return an enum, but only as an optional extra feature)

- dialog boxes
  - the best way to do this i think is to have two ways of opening dialogues
  - 1: a toggle enable and a floating component, allowing for small dialogs
  - 2: a custom event return type that contains a component and a position, and a custom event type to close said dialog (and save changes ig)

- functional components
  - still a lot to think about w this one
  - but the problem is there's absolutely no input checking to current 'functional components' (regular ass functions that return a component)
  - and sooner or later that'll become a performance problem
  - i'm thinking an `Arg<T>` type, that will only let you see a `&T` when the value inside updates
  - or something like that i'm going to bed

the rest:

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

i did a web client test and there's some differences between the platforms obviously \
mostly just mouse input being fucked up \
and i don't want the game to become a patchwork of `if WEB_BUILD { /* some obscure web unfucking code */ }` \
so yeah web build stays in post-1.0

to be able to do web nicely we'd need a central mouse input authority if that makes sense \
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

- [better graphics (textures n allat)](#--rendering-overhaul)
- [main menu](#--main-menu)

## post-1.0

- [could](#--cloud)
- [ampap](#--ampap)
