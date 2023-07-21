# EdolView - Image Viewer for Visualization and Analysis

Image viewer for graphics and vision researchers.

* Pixel value analysis: easy to check each pixel value or mean of pixles in area.
* Colormap visualization: colormapping for better visualization
* Flexible visualization configuraion: single channel visualization for multi-channel images, user setting for min max normalization, custom shader.

<img width="641" alt="Screenshot 2023-07-21 175315" src="https://github.com/edoli/edolview/assets/2304569/6dbee8ff-14d8-4fda-bdf1-a5dbc760d7de">
<img width="641" alt="Screenshot 2023-07-21 183216" src="https://github.com/edoli/edolview/assets/2304569/9e781e30-2392-430f-97df-eba17205fe96">

## Installation
Download in [Releases](https://github.com/edoli/edolview/releases).

## How to use
Usage guide of EdolView can be found on [Wiki](https://github.com/edoli/edolview/wiki).

## Supported OS
Currently, EdolView supports **Windows** and **Linux**. I plan to support **macOS** in future.

## Supported formats
EdolView internally uses OpenCV for image loading, so image formats supported by OpenCV should be work on EdolView.
* EXR (*.exr)
* HDR (*.hdr)
* Flow (*.flo)
* PFM (*.pfm)
* PGM (*.pgm), PPM (*.ppm)
* JPG (*.jpg, *.jpeg), PNG (*.png), TIFF (*.tif, *.tiff), BMP (*.bmp)

## License
EdolView is available under the MIT license.
