#!/usr/bin/env python3

from PIL import Image
from PIL.Image import Resampling
import numpy as np
from dataclasses import dataclass, field
import math
from typing import Literal, ClassVar
from enum import Enum

__version__ = "0.2.0"

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

_info = lambda message: cprint("[INFO] " + str(message), "green")  # noqa: E731
_error = lambda message: cprint("[ERROR] " + str(message), "red")  # noqa: E731
_warning = lambda message: cprint("[WARNING] " + str(message), "yellow")  # noqa: E731

# select debug behavior
# _debug = lambda message: cprint("[DEBUG] " + str(message), "magenta")  # noqa: E731
_debug = lambda message: ...  # noqa: E731


class BlobError(Exception):
    pass

class Anchor(Enum):
    TOP_LEFT = "top-left"  # default, top
    TOP_CENTER = "top-center"
    TOP_RIGHT = "top-right"
    CENTER_LEFT = "left-center"  # center-left
    CENTER_CENTER = "center-center"  # center
    CENTER_RIGHT = "right-center"  # center-right
    BOTTOM_LEFT = "bottom-left"
    BOTTOM_CENTER = "bottom-center"
    BOTTOM_RIGHT = "bottom-right"

    def to_int(self) -> int:
        return {
            Anchor.TOP_LEFT: 0,
            Anchor.TOP_CENTER: 1,
            Anchor.TOP_RIGHT: 2,
            Anchor.CENTER_LEFT: 3,
            Anchor.CENTER_CENTER: 4,
            Anchor.CENTER_RIGHT: 5,
            Anchor.BOTTOM_LEFT: 6,
            Anchor.BOTTOM_CENTER: 7,
            Anchor.BOTTOM_RIGHT: 8,
        }[self]

    @classmethod
    def coerce(cls, anchor: "Anchor | str") -> "Anchor":
        if isinstance(anchor, Anchor):
            return anchor
        elif isinstance(anchor, str):
            return Anchor.from_string(anchor)
        else:
            raise ValueError(f"Invalid anchor: {anchor}")

    @classmethod
    def from_string(cls, s: str) -> "Anchor":
        OPTIONS: dict[tuple[str, ...], Anchor] = {
            ("top",): Anchor.TOP_LEFT,
            ("center",): Anchor.TOP_CENTER,
            ("bottom",): Anchor.BOTTOM_LEFT,
            ("left",): Anchor.TOP_LEFT,
            ("right",): Anchor.TOP_RIGHT,
            ("left", "top"): Anchor.TOP_LEFT,
            ("center", "top"): Anchor.TOP_CENTER,
            ("right", "top"): Anchor.TOP_RIGHT,
            ("center", "left"): Anchor.CENTER_LEFT,
            ("center", "center"): Anchor.CENTER_CENTER,
            ("center", "right"): Anchor.CENTER_RIGHT,
            ("bottom", "left"): Anchor.BOTTOM_LEFT,
            ("bottom", "center"): Anchor.BOTTOM_CENTER,
            ("bottom", "right"): Anchor.BOTTOM_RIGHT,
        }

        s = s.lower()
        parts = s.replace("_", "-").split("-")
        if len(parts) == 1:
            key = [parts[0]]
        elif len(parts) == 2:
            key = [parts[0], parts[1]]
            key.sort()
        anchor = OPTIONS.get(tuple(key))

        if anchor is not None:
            return anchor

        raise ValueError(f"Invalid anchor: {s}")

    @property
    def is_top(self) -> bool:
        return "top" in self.value

    @property
    def is_bottom(self) -> bool:
        return "bottom" in self.value

    @property
    def is_left(self) -> bool:
        return "left" in self.value

    @property
    def is_right(self) -> bool:
        return "right" in self.value

    @property
    def is_center(self) -> bool:
        return "center" in self.value


class PadHeight(str, Enum):
    NONE = "none"
    ROW = "row"
    # ALL = "all"

    @classmethod
    def coerce(cls, pad_height: "PadHeight | str") -> "PadHeight":
        if isinstance(pad_height, PadHeight):
            return pad_height
        elif isinstance(pad_height, str):
            return PadHeight.from_string(pad_height)
        else:
            raise ValueError(f"Invalid pad_height: {pad_height}")

    @classmethod
    def from_string(cls, s: str) -> "PadHeight":
        s = s.lower()
        if s in ("none", "row", "all"):
            return PadHeight(s)
        raise ValueError(f"Invalid pad_height: {s}")

@dataclass
class Blob:
    min_x: int
    min_y: int
    max_x: int
    max_y: int

    l_pad: int = 0
    r_pad: int = 0
    t_pad: int = 0
    b_pad: int = 0

    @property
    def width(self) -> int:
        return self.max_x - self.min_x + 1  # + self.l_pad + self.r_pad

    @property
    def height(self) -> int:
        return self.max_y - self.min_y + 1  # + self.t_pad + self.b_pad

    @property
    def padded_width(self) -> int:
        return self.width + self.l_pad + self.r_pad

    @property
    def padded_height(self) -> int:
        return self.height + self.t_pad + self.b_pad

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

    def add_pad(self, pad: int) -> None:
        self.l_pad += pad
        self.r_pad += pad
        self.t_pad += pad
        self.b_pad += pad

    @property
    def region(self) -> tuple[slice, slice, slice]:
        # image[b.min_y : b.max_y + 1, b.min_x : b.max_x + 1, :]
        return (
            slice(self.min_y, self.max_y + 1),
            slice(self.min_x, self.max_x + 1),
            slice(None),
        )

@dataclass
class Metablob:
    width: int
    height: int

    l_pad: int = 0
    r_pad: int = 0
    t_pad: int = 0
    b_pad: int = 0

    data: bytes = field(default=b"", init=False)

    MAGIC: ClassVar[bytes] = 0b10101010.to_bytes(1, "big")
    META_LEN: ClassVar[int] = len(MAGIC) + 2  # magic length + data length

    def __post_init__(self) -> None:
        assert self.width > 0, "Width must be greater than 0"
        assert self.height > 0, "Height must be greater than 0"

    @property
    def size(self) -> int:
        return self.width * self.height

    @property
    def padded_width(self) -> int:
        return self.width + self.l_pad + self.r_pad

    @property
    def padded_height(self) -> int:
        return self.height + self.t_pad + self.b_pad

    @classmethod
    def from_available_space(cls, width: int, height: int, pad: int = 0) -> "Metablob":
        _debug(f"from_available_space({width}, {height}, {pad})")
        return cls(width - 2 * pad, height - 2 * pad, pad, pad, pad, pad)

    def __str__(self) -> str:
        data = self.MAGIC + len(self.data).to_bytes(2, "big") + self.data
        return f"Metablob({self.width}x{self.height}, size={self.size // 8}b, data={data!r})"

    def __repr__(self) -> str:
        return f"Metablob({self.width}, {self.height})"

    def shrink(
        self, strategy: Literal["smallest", "height-first"] = "smallest"
    ) -> None:
        shape = (int(self.width), int(self.height))
        n_bits = (len(self.data) + self.META_LEN) * 8

        if strategy == "smallest":

            def shrink(shape: tuple[int, int], limit: int) -> tuple[int, int]:
                wide = shape[0] > shape[1]
                if wide:
                    return (shape[0] - 1, shape[1])
                else:
                    return (shape[0], shape[1] - 1)

        elif strategy == "height-first":

            def shrink(shape: tuple[int, int], limit: int) -> tuple[int, int]:
                new_shape = (shape[0], shape[1] - 1)

                if new_shape[0] * new_shape[1] < limit:
                    # can't shrink height anymore
                    new_shape = (shape[0] - 1, shape[1])

                return new_shape

        while True:
            new_shape = shrink(shape, n_bits)

            if new_shape[0] * new_shape[1] < n_bits:
                break

            shape = new_shape

        self.width = shape[0]
        self.height = shape[1]

    def image_data(self) -> np.ndarray:
        data = self.MAGIC + len(self.data).to_bytes(2, "big") + self.data

        if len(data) > self.size // 8:
            raise ValueError(
                f"Data too large for metablob: {len(data)} > {self.size // 8}"
            )

        out = np.zeros((self.height, self.width, 4), dtype=np.uint8)
        out[..., 3] = 255  # set all to 0

        k, l = 0, 0
        for b in data:
            for i in range(8):
                if b & (1 << i):
                    out[k, l] = [255, 255, 255, 255]
                l += 1
                if l == self.width:
                    l = 0
                    k += 1

        return out


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

MAGENTA = np.array(Image.new("RGBA", (1, 1), "#ff00ffff"))


def magentify(
    image: Image.Image,
    pad_magenta: int = 1,  # padding between blobs
    pad_blob: int = 1,  # padding inside the blobs
    anchor: Anchor | Literal["top", "bottom"] = "top",
    pad_height: PadHeight | Literal["none", "row"] = "none",
    metadata: bool = False,
    verbose: bool = False,
    debug: bool = False,
) -> Image.Image:
    anchor = Anchor.coerce(anchor)
    pad_height = PadHeight.coerce(pad_height)

    image = image.convert("RGBA")

    # Find all disconnected blobs of pixels
    blobs, background = find_blobs(image)

    if background is None:
        # I guess all blobs are rectangles and don't have a background
        # Just use transparent background. This is relevant only to
        # the padding of the blobs anyway.
        background = np.zeros(4, dtype=np.uint8)

    # Add padding inside the blobs
    for b in blobs:
        b.add_pad(pad_blob)

    # print all blobs
    if verbose:
        _info(f"Found {len(blobs)} blobs")
        for i, b in enumerate(blobs):
            print(f" {i:02d}: {b}")

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
    I = len(blobs)
    if metadata:
        I += 1  # add a metadata blob

    N = math.ceil(math.sqrt(I))
    M = math.ceil(I / N)
    assert N * M >= I

    if metadata:
        assert N * M >= I + 1

    if verbose:
        _info(f"Packing {len(blobs)} blobs into a {N}x{M} grid")
        if metadata:
            _info("Adding a metadata blob at the end")

    # Figure out the size of the output image
    # We need to know the widest and tallest blobs in each row and column
    row_heights = np.zeros(M, dtype=int)
    row_widths = np.zeros(N, dtype=int)
    row_padded_heights = np.zeros(M, dtype=int)
    row_padded_widths = np.zeros(N, dtype=int)

    n, m = 0, 0
    for i, b in enumerate(blobs):
        row_heights[m] = max(row_heights[m], b.height)
        row_padded_heights[m] = max(row_padded_heights[m], b.padded_height)
        row_widths[m] += b.width
        row_padded_widths[m] += b.padded_width
        n += 1
        if n == N:
            n, m = 0, m + 1

    out_shape = (
        sum(row_padded_heights) + (M + 1) * pad_magenta,
        max(row_padded_widths) + (N + 1) * pad_magenta,
    )

    has_extra_metadata_row = metadata and row_padded_heights[-1] == 0
    if has_extra_metadata_row:
        # We don't have enough space for the metadata blob in the last row
        # Add an extra row for the metadata blob
        if verbose:
            _warning("Adding a row for the metadata blob")
        out_shape = (out_shape[0] + row_padded_heights[-2], out_shape[1])

    # Pad all blobs to the height of the tallest blob in the row. Hence
    # all blobs will be as tall as the tallest blob in the row.
    if pad_height == "row":
        if anchor.is_bottom:
            n, m = 0, 0
            for i, b in enumerate(blobs):
                b.t_pad += row_padded_heights[m] - b.padded_height
                n += 1
                if n == N:
                    n, m = 0, m + 1
        elif anchor.is_top:
            n, m = 0, 0
            for i, b in enumerate(blobs):
                b.b_pad += row_padded_heights[m] - b.padded_height
                n += 1
                if n == N:
                    n, m = 0, m + 1

    if verbose:
        overall_max_height = sum(row_heights)

        _debug(f"Row widths: {row_widths}")
        _debug(f"Max heights: {row_heights} (overall: {overall_max_height})")

        overall_max_height = sum(row_padded_heights)
        _debug(f"Row padded widths: {row_padded_widths}")
        _debug(
            f"Row padded heights: {row_padded_heights} (overall: {overall_max_height})"
        )

    # if pad_height == "all":
    #     # Pad all blobs to the height of the tallest blob in the row. Hence
    #     # all blobs will be as tall as the tallest blob in the row.
    #     row_heights = np.full(M, max(row_heights))

    out_image_data = np.ones((*out_shape, 4), dtype=np.uint8) * MAGENTA

    if verbose:
        _info(f"Output image size: {out_shape[0]}x{out_shape[1]}")
        _debug(f"Magenta padding: {pad_magenta}")
        _info(f"Blob padding: {pad_blob}")
        _info(f"Anchor: {anchor.name}")
        _debug(f"Pad height: {pad_height.name}")

    # Copy the blobs into the output image
    image_data = np.array(image)
    n, m = 0, 0  # indices in the blob grid
    k, l = pad_magenta, pad_magenta  # pixel-level cursor in the output image

    for i, b in enumerate(blobs):
        blob_image_data = image_data[b.region]

        # Move the anchor for this particular blob if needed
        kp, lp = k, l
        if anchor.is_bottom:
            delta = row_padded_heights[m] - b.padded_height
            kp = k + delta

        # paste transparency into the entire *padded* blob region
        region = (
            slice(kp, kp + b.padded_height),
            slice(lp, lp + b.padded_width),
            slice(None),
        )

        if debug:
            out_image_data[region] = np.array([0, 0, 255, 128])  # debug
        else:
            out_image_data[region] = background

        region = (
            slice(kp + b.t_pad, kp + b.height + b.t_pad),
            slice(lp + b.l_pad, lp + b.width + b.l_pad),
            slice(None),
        )

        out_image_data[region] = blob_image_data

        if debug:
            # Draw a debug pixel at the anchor point
            out_image_data[k, l] = np.array([0, 0, 255, 255])

            if anchor.is_bottom:
                out_image_data[kp, lp] = np.array([0, 0, 255, 255])

        # Move the pixel cursor
        l += b.width + pad_magenta + b.l_pad + b.r_pad

        # Keep track of the blob grid coordinates
        n += 1
        if n == N:
            n, m = 0, m + 1
            l = pad_magenta
            k += row_padded_heights[m - 1] + pad_magenta

    if metadata:
        if debug:
            out_image_data[k, l] = np.array([0, 0, 255, 255])  # debug

        # Figure out metadata blob size
        metablob = Metablob.from_available_space(
            out_shape[1] - l - pad_magenta,
            out_shape[0] - k - pad_magenta,
            pad=pad_blob,
        )

        # for _ in range(10):
        #     metablob.data += b"hello world"

        metablob.data += pad_blob.to_bytes(1, "big")
        metablob.data += anchor.to_int().to_bytes(1, "big")

        metablob.shrink(
            strategy="height-first" if has_extra_metadata_row else "smallest"
        )

        if verbose:
            _info("Adding metablob")
            _info(metablob)

        # paste transparency into the entire *padded* metablob region
        region = (
            slice(k, k + metablob.padded_height),
            slice(l, l + metablob.padded_width),
            slice(None),
        )

        if debug:
            out_image_data[region] = np.array([0, 0, 255, 128])  # debug
        else:
            out_image_data[region] = background

        region = (
            slice(k + metablob.t_pad, k + metablob.t_pad + metablob.height),
            slice(l + metablob.l_pad, l + metablob.l_pad + metablob.width),
            slice(None),
        )
        try:
            out_image_data[region] = metablob.image_data()
        except ValueError as e:
            pass

    if has_extra_metadata_row:
        # The image might be too large. Try to shrink it from the bottom up
        # to fit the last row of blobs.
        if verbose:
            _warning("Shrinking the image to fit the last row of blobs")
            out_image_data = shrink_image_from_bottom(
                out_image_data, pad_magenta=pad_magenta
            )

    return Image.fromarray(out_image_data, "RGBA")

def shrink_image_from_bottom(image_data: np.ndarray, *, pad_magenta: int) -> np.ndarray:
    """Shrink an image from the bottom up to fit the last row of blobs."""
    height, width, _ = image_data.shape

    # figure out how many rows from the bottom of the immage are entirely magenta
    n_magenta_rows = 0
    for i in range(height - 1, -1, -1):
        if np.all(image_data[i, :, :] == MAGENTA):
            n_magenta_rows += 1
        else:
            break

    n_to_crop = n_magenta_rows - pad_magenta
    if n_to_crop < 0:
        raise ValueError("Not enough magenta rows to shrink the image")
    elif n_to_crop == 0:
        return image_data
    else:
        return image_data[:-n_to_crop, :, :]


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
        type=Anchor.coerce,
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
        type=PadHeight.coerce,
        default="none",
    )

    parser.add_argument(
        "--pad-blob",
        help="Padding inside the blobs.",
        type=int,
        default=0,
    )

    parser.add_argument(
        "--debug",
        help="Debug mode.",
        action="store_true",
        default=False,
    )

    parser.add_argument(
        "--metadata",
        help="Add metadata to the output image.",
        action="store_true",
        default=False,
    )

    parser.add_argument("input", help="Input image file.")
    parser.add_argument("output", help="Output image file.")

    args = parser.parse_args()

    input_image = Image.open(args.input)

    output_image = magentify(
        input_image,
        pad_magenta=args.pad,
        anchor=args.anchor,
        pad_height=args.pad_height,
        pad_blob=args.pad_blob,
        debug=args.debug,
        verbose=args.verbose,
        metadata=args.metadata,
    )

    if args.upscale > 1:
        output_image = upscale(output_image, args.upscale)

    output_image.save(args.output)


if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        _error(f"{e.__class__.__name__}: {e}")
        exit(1)
