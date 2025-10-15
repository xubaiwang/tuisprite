# tuisprite

Create sprites images in your terminal.

> [!NOTE]
> Still a work in progress.

![screenshot](./assets/screenshot.png)

## Usage

- cli
  - `tuisprite` open an empty drawing
  - `tuisprite <path.json>` open drawing at path
- command mode `:<command>`
  - `:w` save
  - `:w <path>` save to path
  - `:q` quit
- script mode `:=<script>` run JavaScript code
  - `:= color = "red"` set color to red
- key bindings
  - `-` decrease size
  - `+/=` increase size
  - `E` erase all
