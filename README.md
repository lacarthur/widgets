# widgets

widgets is a small proof of concept bar for Wayland. It is made using the [`iced`](https://iced.rs/) library directly. It is not feature complete at all, but any further development will have to wait until iced supports the `wlr layer shell` protocol, which will probably have to wait until `winit` does. In the meantime, I'm using the Cosmic fork of `iced`, which supports `wlr layer shell` via `sctk`, but as far as I can tell, it does not support multiple windows for it, which I would like for a bar.

Right now, the bar has three different "widgets": 
- a Hyprland workspace display, that communicates with Hyprland via IPC socket,
- a clock, that is also able to display the date,
- a battery display, which displays the status and charge of the battery

![videobar](https://github.com/user-attachments/assets/5831a8f2-df16-47d5-a8d1-5d7ba87c2bd9)
