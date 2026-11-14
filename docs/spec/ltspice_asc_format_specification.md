# LTspice `.asc` File Format Specification
*Target Audience: LLMs / Code Generators building Rust Parsers & SVG Renderers*

## 1. Overview & General Syntax Rules

LTspice schematic files (`.asc`) are line-oriented plain-text files (encoded in UTF-8 or ASCII). They define circuit connections, SPICE directives, component placement, and graphical primitives.

### Basic Syntax Principles
- **Line-Oriented:** Each line starts with an uppercase command keyword (token), followed by whitespace-separated parameters.
- **Coordinate System:**
  - Standard integer grid system (typically multiples of 16 or 8 units).
  - **Origin $(0,0)$:** Top-left corner.
  - **X-axis:** Increases to the right ($+X$).
  - **Y-axis:** Increases downwards ($+Y$).
- **Stateful Parsing:** Attribute lines (`SYMATTR`) always belong to the most recently declared `SYMBOL` entry above them.

---

## 2. Coordinate System & Transformations

### Component Orientations
Symbols can be rotated and mirrored. The standard orientations are:
- `R0`: Normal (0 degrees rotation)
- `R90`: Rotated 90 degrees clockwise
- `R180`: Rotated 180 degrees
- `R270`: Rotated 270 degrees clockwise (or 90 counter-clockwise)
- `M0`: Mirrored horizontally (flipped along Y-axis)
- `M90`: Mirrored horizontally, then rotated 90 degrees clockwise
- `M180`: Mirrored horizontally, then rotated 180 degrees
- `M270`: Mirrored horizontally, then rotated 270 degrees

### SVG Mapping Formula
To map LTspice coordinates to SVG coordinates:
1. Standard LTspice coordinates already use top-left origin ($+Y$ down), matching native SVG coordinates directly.
2. SVG Transform Matrix for Symbol Placement at $(X_0, Y_0)$ with rotation $\theta$:
   $$\text{transform}=\text{"translate}(X_0, Y_0)\ \text{rotate}(\theta)"$$
   *(Note: Mirrored elements require `scale(-1, 1)` prior to rotation).*

---

## 3. Structural Specification of Keywords

### 3.1 Document & Metadata
* **`VERSION <major>[.<minor>]`**
  - Specifies format revision (e.g., `VERSION 4`).
* **`SHEET <page_num> <width> <height>`**
  - Canvas dimension definition.
  - Example: `SHEET 1 880 680` $\rightarrow$ SVG canvas width 880, height 680.

### 3.2 Electrical Topology
* **`WIRE <x1> <y1> <x2> <y2>`**
  - Conductive wire segment between $(x_1, y_1)$ and $(x_2, y_2)$.
* **`FLAG <x> <y> <net_name>`**
  - Named net terminal or ground node.
  - Net name `0` denotes explicit circuit Ground (GND).
* **`BUSTAP <x> <y>`**
  - Bus connection tap point.

### 3.3 Components & Attributes
* **`SYMBOL <type> <x> <y> <orientation>`**
  - Instantiates a component symbol (e.g., `SYMBOL res 160 96 R0`).
* **`SYMATTR <key> <value>`**
  - Key-value attributes belonging to the active `SYMBOL`.
  - Common Keys:
    - `InstName`: Unique designator (e.g., `R1`, `C3`, `U1`).
    - `Value`: Electrical value (e.g., `10k`, `100n`, `1N4148`).
    - `Value2`: Secondary value string.
    - `SpiceLine` / `SpiceLine2`: Additional SPICE params.

### 3.4 Text & Directives
* **`TEXT <x> <y> <alignment> <size> <content>`**
  - **Prefix conventions:**
    - `!` : SPICE directive (e.g., `TEXT 32 32 Left 2 !.tran 10m`)
    - `;` : Comment text (e.g., `TEXT 32 64 Left 2 ;Power Supply Circuit`)
  - **Alignment tokens:** `Left`, `Right`, `Center`.
  - **Font size values:** Typically integers `1` (small) to `4` (large).

---

## 4. Graphical Primitives & SVG Translation

LTspice supports direct schematic drawings (non-functional visual elements).

### 4.1 Line (`LINE`)
- **Format:** `LINE <style> <x1> <y1> <x2> <y2>`
- **Style Tokens:** `Normal` (solid, thin), `Dotted`, `Dashed`, `Bold` (thick solid).
- **SVG Conversion:**
  ```xml
  <line x1="x1" y1="y1" x2="x2" y2="y2" stroke="black" stroke-width="1" />
  ```
  *(Map `Bold` to `stroke-width="2"`, `Dotted` to `stroke-dasharray="2,2"`)*

### 4.2 Rectangle (`RECTANGLE`)
- **Format:** `RECTANGLE <style> <x1> <y1> <x2> <y2>`
- **Parameters:** Opposing corners $(x_1, y_1)$ and $(x_2, y_2)$.
- **SVG Conversion:**
  - $x = \min(x_1, x_2)$
  - $y = \min(y_1, y_2)$
  - $width = |x_2 - x_1|$
  - $height = |y_2 - y_1|$
  ```xml
  <rect x="x" y="y" width="width" height="height" fill="none" stroke="black" />
  ```

### 4.3 Circle / Ellipse (`CIRCLE`)
- **Format:** `CIRCLE <style> <x1> <y1> <x2> <y2>`
- **Parameters:** Bounding box corners $(x_1, y_1)$ and $(x_2, y_2)$.
- **SVG Conversion:**
  - $cx = \frac{x_1 + x_2}{2}$
  - $cy = \frac{y_1 + y_2}{2}$
  - $rx = \frac{|x_2 - x_1|}{2}$
  - $ry = \frac{|y_2 - y_1|}{2}$
  ```xml
  <ellipse cx="cx" cy="cy" rx="rx" ry="ry" fill="none" stroke="black" />
  ```

### 4.4 Arc / Elliptic Arc (`ARC`)
- **Format:** `ARC <style> <x1> <y1> <x2> <y2> <x3> <y3> <x4> <y4>`
- **Parameters:**
  - $(x_1, y_1), (x_2, y_2)$: Bounding box of the full ellipse.
  - $(x_3, y_3)$: Ray defining the **start point / angle** of the arc.
  - $(x_4, y_4)$: Ray defining the **end point / angle** of the arc.

- **Mathematical Calculation for SVG Path (`d="M ... A ..."`):**
  1. Calculate Center & Radii:
     $$cx = \frac{x_1 + x_2}{2}, \quad cy = \frac{y_1 + y_2}{2}$$
     $$rx = \frac{|x_2 - x_1|}{2}, \quad ry = \frac{|y_2 - y_1|}{2}$$
  2. Angles relative to center:
     $$\theta_1 = \text{atan2}(y_3 - cy, x_3 - cx)$$
     $$\theta_2 = \text{atan2}(y_4 - cy, x_4 - cx)$$
  3. Actual Arc Start/End coordinates on ellipse boundary:
     $$P_{\text{start\_x}} = cx + rx \cdot \cos(\theta_1), \quad P_{\text{start\_y}} = cy + ry \cdot \sin(\theta_1)$$
     $$P_{\text{end\_x}} = cx + rx \cdot \cos(\theta_2), \quad P_{\text{end\_y}} = cy + ry \cdot \sin(\theta_2)$$
  4. Determine `large-arc-flag` and `sweep-flag`:
     - LTspice renders arcs **counter-clockwise** from start ray to end ray.
     - `sweep-flag` = `0` (or `1` depending on coordinate inversion; in SVG standard Y-down space, CCW corresponds to `0`).
     - `large-arc-flag` = `1` if $\Delta\theta > \pi$, else `0`.

- **SVG Conversion:**
  ```xml
  <path d="M P_start_x P_start_y A rx ry 0 largeArcFlag 0 P_end_x P_end_y" fill="none" stroke="black" />
  ```

---

## 5. Rust AST Data Structure Blueprint

When prompting the LLM to generate Rust code, instruct it to use a structure similar to this:

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Schematic {
    pub version: Option<String>,
    pub sheet: Option<Sheet>,
    pub wires: Vec<Wire>,
    pub flags: Vec<Flag>,
    pub symbols: Vec<Symbol>,
    pub texts: Vec<TextDirective>,
    pub graphics: Vec<GraphicPrimitive>,
}

#[derive(Debug, Clone)]
pub struct Sheet {
    pub page: u32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone)]
pub struct Wire {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}

#[derive(Debug, Clone)]
pub struct Flag {
    pub x: i32,
    pub y: i32,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub orientation: String,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct TextDirective {
    pub x: i32,
    pub y: i32,
    pub align: String,
    pub size: i32,
    pub is_command: bool, // true if starts with '!'
    pub text: String,
}

#[derive(Debug, Clone)]
pub enum LineStyle {
    Normal,
    Dotted,
    Dashed,
    Bold,
}

#[derive(Debug, Clone)]
pub enum GraphicPrimitive {
    Line { style: LineStyle, x1: i32, y1: i32, x2: i32, y2: i32 },
    Rectangle { style: LineStyle, x1: i32, y1: i32, x2: i32, y2: i32 },
    Circle { style: LineStyle, x1: i32, y1: i32, x2: i32, y2: i32 },
    Arc { style: LineStyle, x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32, x4: i32, y4: i32 },
}
```

---

## 6. Parsing Strategy Hints for LLM Prompts

1. **Tokenizer Strategy:** Split lines using whitespace (`split_whitespace()`), but treat `SYMATTR` and `TEXT` differently, as their payload content can contain spaces.
2. **State Machine:** Maintain an optional `current_symbol: Option<Symbol>` while iterating lines. When encountering `SYMATTR`, insert the attribute into `current_symbol`. Push `current_symbol` to `schematic.symbols` whenever a new non-attribute token is encountered or at EOF.
3. **Robustness:** Gracefully ignore unknown tokens to maintain forward compatibility with future LTspice extensions.