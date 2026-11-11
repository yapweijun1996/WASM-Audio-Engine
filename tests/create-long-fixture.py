"""Create a one-hour mono PCM16 silence WAV without overwriting an existing file."""

import argparse
import wave


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("path", help="Destination for a new WAV file")
    args = parser.parse_args()
    with open(args.path, "xb") as destination:
        with wave.open(destination, "wb") as output:
            output.setnchannels(1)
            output.setsampwidth(2)
            output.setframerate(16000)
            remaining = 16000 * 3600
            while remaining:
                frames = min(remaining, 8192)
                output.writeframesraw(bytes(frames * 2))
                remaining -= frames


if __name__ == "__main__":
    main()
