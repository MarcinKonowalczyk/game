#!/usr/bin/env python3

from PIL import Image
from PIL.Image import Resampling


def main() -> None:
    import argparse

    parser = argparse.ArgumentParser(
        description="Resample an image by a given factor using nearest neighbor interpolation."
    )

    parser.add_argument(
        "-v",
        "--verbose",
        help="Print verbose output.",
        action="store_true",
        default=False,
    )

    parser.add_argument(
        "factor",
        help="Factor by which to resample the image. Must be a positive float. Factor > 1 will upsample the image, factor < 1 will downsample the image.",
        type=float,
    )

    parser.add_argument("input", help="Input image file.")
    parser.add_argument("output", help="Output image file.")

    args = parser.parse_args()

    if args.factor <= 0:
        raise ValueError("Factor must be a positive float.")

    input_image = Image.open(args.input)

    output_image = input_image.resize(
        # (input_image.width // args.factor, input_image.height // args.factor),
        (int(input_image.width * args.factor), int(input_image.height * args.factor)),
        resample=Resampling.NEAREST,
    )

    output_image.save(args.output)


if __name__ == "__main__":
    main()
