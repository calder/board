# Fonts

Bundled so the page looks the same on every device and the screenshot tests don't depend on
installed fonts. All three are under the SIL Open Font License 1.1 (license files alongside) and
declare no Reserved Font Names, so they can be subset and redistributed.

| File | Font | Source |
| --- | --- | --- |
| `inter.woff2` | Inter 4.1 (variable) | [rsms/inter](https://github.com/rsms/inter/releases/tag/v4.1) |
| `roboto-mono.woff2` | Roboto Mono (variable) | [google/fonts](https://github.com/google/fonts/tree/main/ofl/robotomono) |
| `noto-color-emoji.woff2` | Noto Color Emoji | [google/fonts](https://github.com/google/fonts/tree/main/ofl/notocoloremoji) |

Inter and Roboto Mono are cut down to Latin, and Noto Color Emoji to the color squares the board
can show:

```sh
latin=U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD
pyftsubset InterVariable.ttf --unicodes=$latin --flavor=woff2 --output-file=inter.woff2
pyftsubset 'RobotoMono[wght].ttf' --unicodes=$latin --flavor=woff2 --output-file=roboto-mono.woff2
pyftsubset NotoColorEmoji-Regular.ttf --unicodes=U+1F7E5-1F7EA,U+2B1B-2B1C,U+FE0F --flavor=woff2 --output-file=noto-color-emoji.woff2
```
