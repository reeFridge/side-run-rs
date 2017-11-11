# side-run-rs

Demo-like project where i'am trying to create something like *co-op-top-down-side-scrolling* game with rust pl.



### Implemented features:
* Game cycle (main game loop)
* Simple scene switcher (just changing handler for game events)
  * Menu scene: setup player config (color, name) and host address:port
  * Play scene: spawn (by `space` key) and control your colored Rect!
* Basic client-server messaging (like events)
  * `connect`, `spawn player`, `update player position` ... no more yet.
* Basic shadow-casting
* Movement, Side-scroll camera
  
**Simple server implementation:** [gist](https://gist.github.com/reeFridge/055fb15bae40056d8b92c73965146c5b)

### Notes:

* Now multiplayer is not primary goal.

### Build & Run

Change version of dependence `pistoncore-input` in Cargo.toml of button_tracker submodule (libs/button_tracker) to 0.19.0
for compatibility with `piston_window` and run: 

`$ cargo run`
