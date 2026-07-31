## CONSTS

### COLOR

#### 8bit color

| foreground | foreground light | background | background light |
| ---------- | ---------------- | ---------- | ---------------- |
| MAGENTA    | LIGHT_MAGENTA    | BG_MAGENTA | BG_LIGHT_MAGENTA |
| CYAN       | LIGHT_CYAN       | BG_CYAN    | BG_LIGHT_CYAN    |

...

usage：

```
COLOR.RED + 'lume' + COLOR.RESET
# same as
string.red('lume')
```

#### 256bit color

| foreground | background |
| ---------- | ---------- |
| FG_1       | BG_1       |
| FG_2       | BG_2       |
| ...        | ...        |
| FG_256     | BG_256     |

usage：

```
COLOR.FG_50 + 'lume' + COLOR.RESET
# same as
string.clr('lume',50)
```

#### true color

- by name

| foreground      | background      |
| --------------- | --------------- |
| aliceblue       | BG_aliceblue    |
| BG_antiquewhite | BG_antiquewhite |
| ...             | ...             |
| yellowgreen     | BG_yellowgreen  |

to list the avaluable colors, use：

```
string.colors(false)
```

usage：

```
COLOR.green + 'lume' + COLOR.RESET
# same as
string.color('lume','green')
```

- by hex code

| foreground | background |
| ---------- | ---------- |
| FGX_000000 | BGX_000000 |
| FGX_000001 | BGX_000001 |
| ...        | ...        |
| FGX_ffffff | BGX_ffffff |

usage：

```
COLOR.FGX_aaff22 + 'lume'
# same as
string.color('lume','#aaff22')
```

### MATH

MATH.E
MATH.PHI
MATH.PI

### STYLE

STYLE.BLINK
STYLE.BOLD
STYLE.DIM
STYLE.HIDDEN
STYLE.ITALIC
STYLE.NORMAL
STYLE.RESET
STYLE.RESET_BLINK
STYLE.RESET_BOLD
STYLE.RESET_DIM
STYLE.RESET_HIDDEN
STYLE.RESET_ITALIC
STYLE.RESET_NORMAL
STYLE.RESET_REVERSE
STYLE.RESET_STRIKE
STYLE.RESET_UNDERLINE
STYLE.REVERSE
STYLE.STRIKE
STYLE.UNDERLINE

