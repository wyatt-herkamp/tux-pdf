# Tux-PDF

A simple to use PDF generator for Rust


PDF Specification: https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/PDF32000_2008.pdf


## Goals

The goal of this library to provide an easy way of generating pdfs. Specifically PDF that contains report, invoice, etc type data.

Allowing to do tables and text formatting.

Without sacrificing all the time in the world to generate a pdf.


## Current State
Early development stage. Not ready for use.

- [x] Basic PDF Creation
- [x] Text Rendering
- [ ] Fonts
  - [X] Custom Fonts
  - [ ] Built In Pdf Fonts (They work however, not metrics are provided meaning text is not correctly positioned)
- [ ] External Objects
  - [X] Images
  - [ ] SVG
  - [ ] Other External Object Types
- [ ] Graphics
  - [x] Lines
  - [ ] Shapes (Circles, Rectangles, etc)
  - [ ] Paths
- [ ] Layers
- [ ] Layouts and Tables
  - [x] Tables (Works but needs to be improved)
  - [ ] Grid Layout And Flex Layout using [Taffy](https://github.com/DioxusLabs/taffy)
- [ ] Wasm Support (Not tested yet would like to have an example web app)

### Known Issues
- [ ] Alpha Values are not supported
- [ ] Built-in Fonts are barely supported
- [ ] Inconsistent shapes system
- [ ] Inconsistent and confusing styling api


## Examples
- [CSV to PDF](examples/csv_to_pdf/main.rs) - A simple example of how to convert a csv file to a pdf
- [Hello World](examples/hello_world/main.rs) - Shows hello world and an image
## Credit

This library is inspired partially by [printpdf](https://github.com/fschutt/printpdf)

Thank you to the authors of [lopdf](https://github.com/J-F-Liu/lopdf/) for doing the heavy lifting of the pdf file format.

## License

Licensed under either of these:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)