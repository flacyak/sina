# sina

Generate a QR code from the command line and display it in the terminal.
The encoder is implemented from scratch: all 40 versions, four error correction levels and numeric, alphanumeric, byte and Kanji modes.

## Usage

```
sina [OPTIONS] [TEXT]

Options:
  -e, --ec <l|m|q|h>  Error correction level (default: m)
  -p, --plain         Print without ANSI colors
  -h, --help          Show this help
```

Text is read from stdin when omitted:

```
echo "https://example.com" | sina
```
