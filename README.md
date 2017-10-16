# side-run-rs

Demo-like project where i'am trying to create something like *co-op-top-down-side-scrolling* game with rust pl. 

### Implemented features:
* Game cycle (main loop)
* Simple scene switcher (just changing handler for game events)
  * Menu scene: setup player config (color, name) and host address:port
  * Play scene: controll your colored Rect!
* Basic client-server messaging (like events)
  * `connect`, `spawn player`, `update player position` ... no more yet.
  
**Simple server implementation:** [gist](https://gist.github.com/reeFridge/055fb15bae40056d8b92c73965146c5b)

### Build & Run

`$ cargo run`
