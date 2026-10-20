# Bundled fonts

Typst compiles with `--font-path` pointing here and `--ignore-system-fonts`, so
every PDF uses these files and nothing else, on any machine.

| Family | Use | Source | Licence |
|---|---|---|---|
| Inter 4.1 | Body text | rsms/inter release, static TTFs | SIL OFL 1.1 (`OFL-Inter.txt`) |
| Inter Tight | Headings, cover | Google Fonts variable font, instanced to static weights with `fontTools varLib.instancer` | SIL OFL 1.1 (`OFL-InterTight.txt`) |
| JetBrains Mono | Code | JetBrains release, static TTFs | SIL OFL 1.1 (`OFL-JetBrainsMono.txt`) |

Static files only: Typst 0.14 doesn't read variable fonts. Weights 400, 500,
600 and 700, upright and italic (JetBrains Mono: 400, 400 italic, 500, 700).
