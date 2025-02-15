from PIL import Image
from PIL.Image import Resampling
import numpy as np
from dataclasses import dataclass
import math
from typing import Literal

__version__ = "0.1.1"

try:
    import colorama  # type: ignore

    colorama.init()
except ImportError:
    colorama = None


def cprint(message: str, color: str | None = None) -> None:
    """Print a message in color. If colorama is not installed, the message will be printed without color."""
    if colorama is not None and color is not None:
        print(
            colorama.Fore.__dict__[color.upper()] + message + colorama.Style.RESET_ALL
        )
    else:
        print(message)


class BlobError(Exception):
    pass


@dataclass
class Blob:
    min_x: int
    min_y: int
    max_x: int
    max_y: int

    @property
    def width(self) -> int:
        return self.max_x - self.min_x + 1

    @property
    def height(self) -> int:
        return self.max_y - self.min_y + 1

    def __str__(self) -> str:
        return f"Blob({self.width}x{self.height} @ {self.min_x},{self.min_y})"

    def __repr__(self) -> str:
        return f"Blob({self.min_x}, {self.min_y}, {self.max_x}, {self.max_y})"

    def overlaps(self, other: "Blob") -> bool:
        return (
            self.min_x <= other.max_x
            and self.max_x >= other.min_x
            and self.min_y <= other.max_y
            and self.max_y >= other.min_y
        )


def find_blobs(image: Image.Image) -> tuple[list[Blob], np.ndarray | None]:
    """Find all disconnected blobs of pixels in an image.
    Also return the background color of the blobs (or None if
    all blobs are rectangles and don't have a background).
    """

    image = image.convert("RGBA")
    image_data = np.array(image)

    blobs = []
    visited = np.zeros((image.height, image.width), dtype=bool)

    background = None  # Try to figure out what's the background color

    def _is_background(color: np.ndarray) -> bool:
        nonlocal background
        if background is None:
            background = color
        return color[3] == 0

    _none = np.array([])

    def _visit(x: int, y: int) -> np.ndarray:
        """Check if a pixel has been visited and mark it as visited."""
        if x < 0 or x >= image.width or y < 0 or y >= image.height:
            return _none

        if visited[y, x]:
            return _none
        visited[y, x] = True

        pixel_color = image_data[y, x, :]

        if _is_background(pixel_color):
            return _none

        return pixel_color

    def _append_neighbours(stack, x, y):
        """Append neighbours to the stack."""
        if x > 0:
            stack.append((x - 1, y))
        if x < image.width - 1:
            stack.append((x + 1, y))
        if y > 0:
            stack.append((x, y - 1))
        if y < image.height - 1:
            stack.append((x, y + 1))

    for y in range(image.height):
        for x in range(image.width):
            pixel_color = _visit(x, y)
            if not pixel_color.any():
                continue

            blob = Blob(x, y, x, y)

            # Flood fill
            stack: list[tuple[int, int]] = []
            _append_neighbours(stack, x, y)

            while stack:
                x, y = stack.pop()

                pixel_color = _visit(x, y)
                if not pixel_color.any():
                    continue

                blob.min_x = min(blob.min_x, x)
                blob.min_y = min(blob.min_y, y)
                blob.max_x = max(blob.max_x, x)
                blob.max_y = max(blob.max_y, y)

                _append_neighbours(stack, x, y)

            blobs.append(blob)

    # sort the blobs from left to right, top to bottom, by min_x, min_y
    blobs.sort(key=lambda b: (b.min_x, b.min_y))

    return blobs, background


def magentify(
    image: Image.Image,
    pad: int = 1,  # padding between blobs
    anchor: Literal["top", "bottom"] = "top",
    pad_height: Literal["none", "row", "all"] = "none",
    verbose: bool = False,
) -> Image.Image:
    if anchor not in ["top", "bottom"]:
        raise ValueError(f"Invalid anchor: {anchor}")

    if pad_height not in ["none", "row", "all"]:
        raise ValueError(f"Invalid pad_height: {pad_height}")

    image = image.convert("RGBA")

    # Find all disconnected blobs of pixels
    blobs, background = find_blobs(image)

    if background is None:
        # I guess all blobs are rectangles and don't have a background
        # Just use transparent background. This is relevant only to
        # the padding of the blobs anyway.
        background = np.zeros(4, dtype=np.uint8)

    # print all blobs
    if verbose:
        print(f"Found {len(blobs)} blobs")
        for i, b in enumerate(blobs):
            print(f"{i:02d}: {b}")

    # Check if any of the blobs are outside the image (this should never happen)
    for i, b in enumerate(blobs):
        if (
            b.min_x < 0
            or b.min_y < 0
            or b.max_x >= image.width
            or b.max_y >= image.height
        ):
            raise BlobError(
                f"{b} ({i}) is outside the image ({image.width}x{image.height})"
            )

    # Check that no two blobs overlap (this also should never happen)
    for i, b1 in enumerate(blobs):
        for j, b2 in enumerate(blobs):
            if i == j:
                continue
            if b1.overlaps(b2):
                raise BlobError(f"{b1} ({i}) and {b2} ({j}) overlap")

    # The blobs will be packed into an NxM grid
    N = math.ceil(math.sqrt(len(blobs)))
    M = math.ceil(len(blobs) / N)
    assert N * M >= len(blobs)

    if verbose:
        print(f"Packing {len(blobs)} blobs into a {N}x{M} grid")

    # Figure out the size of the output image
    # We need to know the widest and tallest blobs in each row and column
    max_heights = np.zeros(M, dtype=int)
    row_widths = np.zeros(N, dtype=int)

    n, m = 0, 0
    for i, b in enumerate(blobs):
        max_heights[m] = max(max_heights[m], b.max_y - b.min_y + 1)
        row_widths[m] += b.max_x - b.min_x + 1
        n += 1
        if n == N:
            n, m = 0, m + 1

    if verbose:
        overall_max_height = sum(max_heights)

        print(f"Row widths: {row_widths}")
        print(f"Max heights: {max_heights} (overall: {overall_max_height})")

    if pad_height == "all":
        # Pad all blobs to the height of the tallest blob in the row. Hence
        # all blobs will be as tall as the tallest blob in the row.
        max_heights = np.full(M, max(max_heights))

    out_shape = (
        sum(max_heights) + (M + 1) * pad,
        max(row_widths) + (N + 1) * pad,
    )

    magenta = np.array(Image.new("RGBA", (1, 1), "#ff00ffff"))

    out_image_data = np.ones((*out_shape, 4), dtype=np.uint8) * magenta

    if verbose:
        print(f"Output image size: {out_shape[0]}x{out_shape[1]}")

    # Copy the blobs into the output image
    image_data = np.array(image)
    n, m = 0, 0  # indices in the blob grid
    k, l = pad, pad  # pixel-level cursor in the output image

    for i, b in enumerate(blobs):
        blob_image_data = image_data[b.min_y : b.max_y + 1, b.min_x : b.max_x + 1, :]

        if anchor == "top":
            k = sum(max_heights[:m]) + (m + 1) * pad
        elif anchor == "bottom":
            delta = max_heights[m] - (b.max_y - b.min_y + 1)
            k = sum(max_heights[:m]) + (m + 1) * pad + delta

        region = (
            slice(k, k + b.max_y - b.min_y + 1),
            slice(l, l + b.max_x - b.min_x + 1),
            slice(None),
        )

        out_image_data[region] = blob_image_data

        if pad_height != "none":
            delta = max_heights[m] - (b.max_y - b.min_y + 1)
            if delta > 0:
                if anchor == "top":
                    new_slice = slice(
                        k + b.max_y - b.min_y + 1,
                        k + b.max_y - b.min_y + 1 + delta,
                    )
                elif anchor == "bottom":
                    new_slice = slice(k - delta, k)

                region = (new_slice, *region[1:])

                out_image_data[region] = background

        # Move the pixel cursor
        l += b.max_x - b.min_x + 1 + pad

        # Keep track of the blob grid coordinates
        n += 1
        if n == N:
            n, m = 0, m + 1
            l = pad

        if i == 9:
            break

    return Image.fromarray(out_image_data, "RGBA")


def upscale(image: Image.Image, factor: int) -> Image.Image:
    return image.resize(
        (image.width * factor, image.height * factor), Resampling.NEAREST
    )


def main() -> None:
    import argparse

    parser = argparse.ArgumentParser(
        description="Take an image of blobs on background, move all the blobs into boxes and set them up on a magenta background.",
    )

    parser.add_argument(
        "-v",
        "--verbose",
        help="Print verbose output.",
        action="store_true",
        default=False,
    )

    parser.add_argument(
        "--upscale", help="Upscale the output image by a factor.", type=int, default=1
    )

    parser.add_argument("--pad", help="Padding between blobs.", type=int, default=1)

    parser.add_argument(
        "--anchor",
        help="Anchor the blobs to the top or bottom of the boxes. Becomes irrelevant if the blobs have the same height.",
        choices=["top", "bottom"],
        default="top",
    )

    parser.add_argument(
        "--version",
        action="version",
        version=f"%(prog)s {__version__}",
    )

    parser.add_argument(
        "--pad-height",
        help="Pad the height of the output image. Each blob will be held at the anchor and padded to the height of the tallest blob in the row.",
        choices=["none", "row", "all"],
        default="none",
    )

    parser.add_argument("input", help="Input image file.")
    parser.add_argument("output", help="Output image file.")

    args = parser.parse_args()

    input_image = Image.open(args.input)
    
    output_image = magentify(
        input_image,
        pad=args.pad,
        anchor=args.anchor,
        pad_height=args.pad_height,
        verbose=args.verbose,
    )

    if args.upscale > 1:
        output_image = upscale(output_image, args.upscale)

    output_image.save(args.output)

if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        cprint(f"{e.__class__.__name__}: {e}", "red")
        exit(1)
