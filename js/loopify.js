// loopify.js
// https://github.com/veltman/loopify

// Available under the MIT license.

// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions.
// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.


export function loopify(uri, cb) {

    var context = new (window.AudioContext || window.webkitAudioContext)();
    var request = new XMLHttpRequest();

    var obj = undefined;

    // If we have not interacted with the page, we can't play audio
    // Try to resume it every 100ms, only once successful we can play
    var can_play = false;
    var want_to_play = false; // if we want to play but can't yet
    var resume_timeout = 100;

    const timeout = (prom, time) => {
        return Promise.race([prom, new Promise((_r, rej) => setTimeout(rej, time))])
    };

    function resume() {
        timeout(context.resume(), resume_timeout).then(() => {

            // Context is resumed! We can play audio now.
            can_play = true;

            // I we want to play, do it now
            if (want_to_play) {
                want_to_play = false;
                if (obj !== undefined) {
                    obj.play();
                }
            }
        }, resume);
    }

    resume();

    request.responseType = "arraybuffer";
    request.open("GET", uri, true);

    // XHR failed
    request.onerror = function () {
        cb(new Error("Couldn't load audio from " + uri));
    };

    // XHR complete
    request.onload = function () {
        if (request.status !== 200) {
            cb(new Error("Couldn't load audio from " + uri));
        } else {
            context.decodeAudioData(request.response, success, function (err) {
                // Audio was bad
                cb(new Error("Couldn't decode audio from " + uri));
            });
        }
    };

    request.send();

    function success(buffer) {

        var SOURCE;
        var GAIN;
        var future_id; // id of the timeout for the next play
        var VOL = 1.0;

        function play(fade_time) {

            if (fade_time === undefined) {
                fade_time = 0.0;
            }

            // We cannot play yet, but maybe this was triggered by our first
            // interaction with the page, and we will be able to play soon.
            // There is a race between call to play and the callback of the
            // resume of the context. We just set a flag here, and return.
            // The resume callback will check this flag and play if needed.
            if (!can_play) {
                // We can't play audio yet
                want_to_play = true;
                return;
            }

            // Stop if it's already playing
            stop();

            // Called at the start of the new segment, 'fade_time' before
            // the end of the previous one
            function playSegment(prev_gain) {
                var now = context.currentTime;

                // Create a new source (can't replay an existing source)
                SOURCE = context.createBufferSource();
                var gain = context.createGain();
                SOURCE.connect(gain).connect(context.destination);
                SOURCE.buffer = buffer;

                // Fade in this segment
                gain.gain.setValueAtTime(0, now);
                gain.gain.linearRampToValueAtTime(VOL, now + fade_time);

                // Crossfade with previous segment if it exists
                if (prev_gain !== undefined) {
                    prev_gain.gain.linearRampToValueAtTime(0, now + fade_time);
                }

                // start source
                SOURCE.start(now);

                return gain;
            }

            // Play segment and recursively schedule the next one
            function recursivePlay(prev_gain) {
                // Play the current segment
                GAIN = playSegment(prev_gain);

                // Schedule ourselves to play the next segment
                future_id = setTimeout(() => {
                    recursivePlay(GAIN);
                }, (buffer.duration - fade_time) * 1000);
            }

            recursivePlay();

        }

        function stop() {

            want_to_play = false;

            // Stop and clear if it's playing
            if (SOURCE) {
                SOURCE.stop();
                SOURCE = null;
                GAIN = null;
            }

            // Clear any future play timeouts
            if (future_id) {
                clearTimeout(future_id);
                future_id = null;
            }

        }

        function playing() {
            return SOURCE !== undefined;
        }

        function volume(vol) {
            if (GAIN) {
                GAIN.gain.setValueAtTime(vol, context.currentTime);
            }
            VOL = vol; // save for future playbacks
        }

        // Return the object to the callback
        obj = {
            play: play,
            stop: stop,
            playing: playing,
            volume: volume,
        }

        cb(null, obj);

    }

}

loopify.version = "0.2";
