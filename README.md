# `signals`

silly signal game

## roadmap

scrollable component

before we do that:

- [x] stores, also just like think about component lifecycles
- [x] cache function, only rerun something if the inputs change

- [x] rendering to images (so the scrolled element gets clipped by scrollable bounds)
- [x] rendering images

we have a scrollable component!!

it works on the worlds bar, horizontally. it doesn't work in the debug ui for some fucking reason, check main to see what differences there are in implementation

---

also a processor block \
not for use by the user directly, but foreigns are really computationally expensive, especially if there's multiple layers of foreigns \
if when placing a foreign, we didn't fully simulate the world inside as is, but only placed a processor with the instructions on how to produce the outputs based on the inputs, it'd be much faster. \
for some components (like memory) processors would yield incorrect results, so we have to give the user the option to place the foreign as a foreign, but otherwise we should only place a processor
