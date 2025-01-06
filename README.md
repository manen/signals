# `signals`

silly signal game

## roadmap

scrollable component

before we do that:

- [ ] stores, also just like think about component lifecycles
- [ ] cache function, only rerun something if the inputs change

- [ ] rendering to images (so the scrolled element gets clipped by scrollable bounds)
- [ ] rendering images

---

also a processor block \
not for use by the user directly, but foreigns are really computationally expensive, especially if there's multiple layers of foreigns \
if when placing a foreign, we didn't fully simulate the world inside as is, but only placed a processor with the instructions on how to produce the outputs based on the inputs, it'd be much faster. \
for some components (like memory) processors would yield incorrect results, so we have to give the user the option to place the foreign as a foreign, but otherwise we should only place a processor
