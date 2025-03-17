
function img_to_image_data(img) {
    var canvas = document.createElement('canvas');
    var ctx = canvas.getContext('2d');
    canvas.width = img.width;
    canvas.height = img.height;
    ctx.drawImage(img, 0, 0);

    return ctx.getImageData(0, 0, img.width, img.height);
}

// pub struct Blob {
//     pub x_min: u32,
//     pub y_min: u32,
//     pub x_max: u32,
//     pub y_max: u32,
// }

function new_blob(x_min, y_min, x_max, y_max) {
    return {
        x_min: x_min,
        y_min: y_min,
        x_max: x_max,
        y_max: y_max,
    };
}

export function find_blobs(img) {
    let image_data = img_to_image_data(img);

    let width = img.width;
    let height = img.height;

    let blobs = [];

    let visited = new Array(width * height).fill(false);
    let stack = [];

    function is_magenta(color) {
        return color[0] == 255 && color[1] == 0 && color[2] == 255 && color[3] == 255;
    }

    function visit(x, y) {
        if (x < 0 || x >= width || y < 0 || y >= height) {
            return null;
        }
        let i = x + y * width;

        if (i >= visited.length) {
            return null;
        }

        if (visited[i]) {
            return null;
        }

        visited[i] = true;

        let color = image_data.data.slice(i * 4, i * 4 + 4);

        if (is_magenta(color)) {
            return null;
        }

        return color;
    }

    function push_neighbors(x, y) {
        if (x > 0) {
            stack.push([x - 1, y]);
        }
        if (y > 0) {
            stack.push([x, y - 1]);
        }
        if (x < width - 1) {
            stack.push([x + 1, y]);
        }
        if (y < height - 1) {
            stack.push([x, y + 1]);
        }
    }

    for (let x = 0; x < width; x++) {
        for (let y = 0; y < height; y++) {
            let color = visit(x, y);

            if (!color) {
                continue;
            }

            let blob = new_blob(x, y, x, y);

            stack = [];
            push_neighbors(x, y);

            while (stack.length > 0) {
                let [x, y] = stack.pop();

                let color = visit(x, y);

                if (!color) {
                    continue;
                }

                blob.x_min = Math.min(blob.x_min, x);
                blob.y_min = Math.min(blob.y_min, y);
                blob.x_max = Math.max(blob.x_max, x);
                blob.y_max = Math.max(blob.y_max, y);

                push_neighbors(x, y);
            }

            blobs.push(blob);
        }
    }

    // sort blobs first by y then by x
    blobs.sort((a, b) => {
        if (a.y_min != b.y_min) {
            return a.y_min - b.y_min;
        }
        return a.x_min - b.x_min;
    });

    return blobs;
}