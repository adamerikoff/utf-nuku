# utf-nuku

*nuku* (抜く) is Japanese for "to pull out": utf-nuku pulls the bits out of every character.

A UTF-8 / UTF-16 / UTF-32 encoder and decoder written from scratch in Rust, which can show every step bit by bit.

- **utf-nuku-core**: the library, with no dependencies
- **utf-nuku**: the CLI, which works with Unix pipes
- **utf-nuku-gui**: a desktop viewer built with Dioxus

## How UTF-8 works

Every character has a number, its **code point** (`é` = `1110_1001`). UTF-8 stores that number in 1–4 bytes. The first bits of each byte are a **header** that says what kind of byte it is:

| Bytes | Code point range | Lead byte | Continuation bytes | Data bits |
|---|---|---|---|---|
| 1 | up to `0x7F` | `0xxxxxxx` | — | 7 |
| 2 | up to `0x7FF` | `110xxxxx` | `10xxxxxx` | 11 |
| 3 | up to `0xFFFF` | `1110xxxx` | `10xxxxxx` ×2 | 16 |
| 4 | up to `0x10FFFF` | `11110xxx` | `10xxxxxx` ×3 | 21 |

The 4 bit operations used:
- `>>` (shift right) drops bits on the right
- `<<` (shift left) makes room on the right
- `&` (and) with a mask keeps only some bits
- `|` (or) glues bits together

**Encode `é`** (code point `000_1110_1001`, needs 2 bytes):
```
                 simple                          bit operation
split            0_0011 and 10_1001              lead = cp >> 6,  continuation = cp & 11_1111
lead byte        110 + 0_0011  = 1100_0011       (110 << 5) | 0_0011
continuation     10  + 10_1001 = 1010_1001       (10 << 6)  | 10_1001
```

**Decode `[1100_0011, 1010_1001]`:** the same steps in reverse.
```
                 simple                          bit operation
lead byte        1100_0011 = 110 + 0_0011        data = 1100_0011 & 1_1111   (110 → 2 bytes)
continuation     1010_1001 = 10  + 10_1001       data = 1010_1001 & 11_1111
join             0_0011 + 10_1001 = é            (0_0011 << 6) | 10_1001 = 000_1110_1001
```
`+` here means "put next to each other", not addition.

The decoder rejects:
- bytes in the wrong place
- input that ends in the middle of a character
- overlong encodings (a character not written in its shortest form)
- surrogates (`0xD800`–`0xDFFF`)
- numbers above `0x10FFFF`

**UTF-16** uses 16-bit units. A character up to `0xFFFF` is one unit. Above that, subtract `0x10000` and split the remaining 20 bits into a **surrogate pair**: `110110` + 10 bits, then `110111` + 10 bits.
**UTF-32** stores every code point as is, in one 32-bit unit.

## CLI

```sh
cargo install utf-nuku                     # from crates.io, installs the `utf-nuku` command
cargo install --path crates/utf-nuku-cli   # or from this repo
# or without installing: cargo run -q --bin utf-nuku -- <command> …
```

| Command | What it does |
|---|---|
| `utf-nuku encode` | text from stdin → encoded bytes to stdout |
| `utf-nuku decode` | encoded bytes from stdin → text to stdout |

| Option | Values | Default |
|---|---|---|
| `-e, --encoding` | `utf8`, `utf16`, `utf32` | `utf8` |
| `-b, --byte-order` | `be` (left byte first), `le` (right byte first); ignored for utf8 | `be` |
| `-v, --verbose` | print every step to stderr | off |

stdout gets only the data and stderr gets the steps and errors, so `> file` and `| xxd` always get clean bytes. Use `echo -n`; plain `echo` adds a newline that gets encoded too.

### Encode

**Reading `xxd` output:** each line is the offset, then the bytes in hex, then an ASCII preview. The preview shows only plain ASCII bytes and prints `.` for every other byte, so `H......` = `H` + 2 bytes of `é` + 4 bytes of `😀`. All 7 bytes are in the hex part:
```
48           → H    1 byte
c3 a9        → é    2 bytes
f0 9f 98 80  → 😀   4 bytes
```

```console
$ echo -n "Hé😀" | utf-nuku encode | xxd
00000000: 48c3 a9f0 9f98 80                        H......

$ echo -n "Hé😀" | utf-nuku encode | xxd -b
00000000: 01001000 11000011 10101001 11110000 10011111 10011000  H.....
00000006: 10000000                                               .

$ echo -n "Hé😀" | utf-nuku encode -e utf16 | xxd
00000000: 0048 00e9 d83d de00                      .H...=..

$ echo -n "Hé😀" | utf-nuku encode -e utf16 -b le | xxd
00000000: 4800 e900 3dd8 00de                      H...=...

$ echo -n "Hé😀" | utf-nuku encode -e utf32 | xxd
00000000: 0000 0048 0000 00e9 0001 f600            ...H........
```

### File round trip

```console
$ echo -n "Hé😀" | utf-nuku encode -e utf16 > out.bin
$ xxd out.bin
00000000: 0048 00e9 d83d de00                      .H...=..
$ utf-nuku decode -e utf16 < out.bin
Hé😀
```

### Every step (`-v`)

UTF-8 encode (the steps go to stderr, the bytes to `xxd`):
```console
$ echo -n "é" | utf-nuku encode -v | xxd
'é'
  code point      000_1110_1001
  bytes needed    2
  lead byte       110 + 0_0011 → 1100_0011
  continuation    10 + 10_1001 → 1010_1001
00000000: c3a9                                     ..
```

UTF-16 encode into a file, then decode it:
```console
$ echo -n "😀" | utf-nuku encode -e utf16 -v > out.bin
'😀'
  code point      0_0001_1111_0110_0000_0000
  units needed    2
  − 0x10000       0000_1111_0110_0000_0000
  high surrogate  110110 + 00_0011_1101 → 1101_1000_0011_1101
  low surrogate   110111 + 10_0000_0000 → 1101_1110_0000_0000

$ utf-nuku decode -e utf16 -v < out.bin
'😀'
  high surrogate  1101_1000_0011_1101 → 110110 + 00_0011_1101
  units needed    2
  low surrogate   1101_1110_0000_0000 → 110111 + 10_0000_0000
  joined 10 | 10  0000_1111_0110_0000_0000
  + 0x10000       0_0001_1111_0110_0000_0000
😀
```

UTF-32:
```console
$ echo -n "😀" | utf-nuku encode -e utf32 -v > /dev/null
'😀'
  code point      0_0001_1111_0110_0000_0000
  unit            0000_0000_0000_0001_1111_0110_0000_0000
```

## GUI

```sh
cargo run -p utf-nuku-gui
```

- **Encode / Decode** switch. Encode reads the box as text; Decode reads bytes as hex (`48 c3 a9`), bits (`01001000 11000011`), or a mix.
- **UTF-8 / UTF-16 / UTF-32** and **Big / Little endian** choices.
- The result **updates as you type**.
- **All bytes:** every byte in one box, as **Hex** or **Bits**, with a **Copy** button. The copied text can be pasted back in Decode mode.
- **Per char:** each character with its bytes. Click one (here or in the all-bytes box) to highlight its bytes and see its steps.
- **Steps:** header bits in **orange**, data bits in **blue**, e.g. `[110][00011] = c3`.

Encoding `Hé😀` in UTF-8, with `é` selected:

![Encode Hé😀 in UTF-8](https://gitlab.com/adamerikoff/utf-nuku/-/raw/main/docs/screenshots/encode_utf8.svg)

`😀` in UTF-16, with its surrogate pair steps:

![😀 in UTF-16](https://gitlab.com/adamerikoff/utf-nuku/-/raw/main/docs/screenshots/encode_utf16_pair.svg)

Decoding bytes typed as hex, with `😀` selected:

![Decode hex](https://gitlab.com/adamerikoff/utf-nuku/-/raw/main/docs/screenshots/decode_hex.svg)

## License

Licensed under the [Apache License 2.0](LICENSE). If you use or redistribute utf-nuku, keep the [NOTICE](NOTICE) file with it, which credits the author.

## Credits

Written by **Adam Erikuly**. The GUI parts of the code were written with help from **Claude** (Anthropic), used as a pair-programming assistant.
