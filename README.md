# `jset-desk`
## Rust crate `jset_desk`
An application for drawing "trippy fractals" by iterating a function and
coloring pixels based on how long the associated complex numbers take to
diverge.

This is a more powerful, desktop version of my
[jset-wasm](https://github.com/d2718/jset-wasm) project, which you can
find in action [here](http://d2718.net/jset/).

### Installation

There's a [static Windows 10
binary](https://d2718.net/jset/jset_desk_win10.exe) you can download.

Otherwise, you can compile it yourself. There aren't many Rust crate
dependencies, but [FLTK](https://www.fltk.org/) is a doozy; you'll need
to install [all of the dependencies of
`fltk-rs`](https://docs.rs/fltk/latest/fltk/index.html#dependencies),
but then it should just work. Remember to build a `--release` version,
or it'll be disappointingly slow.

### Use

Clicking the mouse on the image will cause the image to be recentered at
that point.

Changes to the color map, the iteration parameters, or the image size
will not be reflected until you focus the main window and hit return (or
click to recenter).

Unless you have a recent version of
[ImageMagick](https://imagemagick.org/index.php) installed (and in your
`$PATH` or `%PATH%` or whatever Windows calls it), files will save in
[Netpbm P6 format](https://en.wikipedia.org/wiki/Netpbm) and the file
size will be _huge_. You'll want to convert the image to a .png (or maybe
.webp? I don't understand that format) using ImageMagick or The Gimp or
something.

If you save your image with the view of it scaled to anything other than
1:1, _it will save at that scale_. This is fine if you want to smooth out
the image by making it huge and scaling it down (although just about any
image editor will have a better scaling algorithm), but if you want to save
it at 1:1, make sure you click on 1:1 before you hit save.

You can also save your image parameters: iterator, coefficients, color, etc.,
and then load them later to continue work where you left off. I made the
load button smaller so you'll be less likely to hit it and wipe out any
work you're in the middle of accidentally.

### Roadmap

In no particular order, I'd like to add:

  * saving natively in .png format. It's more complicated than just adding
    crate `png` as a dependency and calling a function, which I guess
    shouldn't be surprising.
  * keyboard shortcuts to change pane focus. FLTK inputs really like to eat
    key events. There has to be a way to do this, I just haven't figured
    out how because the `fltk-rs` documentation seems to assume you're
    already intimately familiar with using the C++ library.
  * ~~saving the image, color, and iteration parameters to a file, (and
    subsequently loading them) so you can resume work on an image, or
    generate a super-high-rez version later if someone likes it. This should
    be easy, but it involves getting _another_ heavyweight dependency
    (`serde`) involved. (I love `serde`, though, don't get me wrong; it's
    just a huge dependency.)~~ Done in 0.2.1.
  * SIMD. The iteration step is already _parallelized_, but I'd like to
    explore speeding it (and color map calculation) further with some
    vectorization. LLVM supposedly does some vector optimization when it
    realizes it can, but I don't know how smart or aggressive or effective
    it is. This is going to take some experimentation and profiling.
  * Once native .png saving is worked out, I'd like to explore saving the
    "image parameters" as an EXIF sidecar to the saved .png, so you'd
    never even have to worry about saving a separate file. You'd only have
    to worry about image manipulation programs stripping your EXIF data.

### Help

If you have any experience with creating FLTK UIs, I'd love to ask you some
questions. If you've ever _themed_ FLTK, just go ahead and make a pull
request. I don't care that this looks like a Win95 app, but it'd probably
be nice if it were slicker.