# LTspice Symbol File (`.asy`) Specification

Diese Spezifikation beschreibt den Aufbau und die Syntax von LTspice-Symboldateien (`.asy`), um einen robusten Parser für das Format implementieren zu können.

---

## 1. Allgemeine Dateistruktur

* **Dateiformat:** Plain-Text-Datei (ASCII oder UTF-8 ohne BOM).
* **Zeilenenden (Line Endings):** Entweder Unix-Style (`\n`) oder Windows-Style (`\r\n`). Der Parser sollte nach dem Einlesen aller Zeilen nachgestellte Carriage Returns (`\r`) entfernen.
* **Struktur:** Die Datei besteht aus einer sequenziellen Liste von Befehlen (Commands). Pro Zeile steht in der Regel genau ein Befehl.
* **Tokenisierung & Parsing-Regeln:** 
  * Zeilen können durch Leerzeichen oder Tabs getrennt sein.
  * Viele Befehle haben eine feste Anzahl an Parametern am Anfang der Zeile (z. B. Koordinaten, Typen). Der restliche Teil einer Zeile ab einem bestimmten Token (z. B. bei Attributwerten oder Texten) gehört oft als kompletter String zum letzten Parameter.

---

## 2. Befehlsreferenz (Lexikalische Elemente)

Jede Zeile beginnt mit einem Schlüsselwort (Command Identifier), gefolgt von Parametern. Schlüsselwörter sind in der Regel groß-/titelgeschrieben (`Version`, `SymbolType`, `LINE`, `PIN`, etc.), sollten aber case-insensitive geparst werden.

### 2.1 Header
* **`Version <number>`**
  * *Beschreibung:* Muss zwingend die **erste Zeile** der Datei sein.
  * *Parameter:* `<number>` (Gleitkomma- oder Integer-Wert, z. B. `4` oder `4.1`).
  * *Beispiel:* `Version 4`

### 2.2 Symboleigenschaften & Typen
* **`SymbolType [CELL | BLOCK]`**
  * *Beschreibung:* Definiert den Typ des Symbols. `CELL` steht für normale Bauteile/Subcircuits, `BLOCK` für hierarchische Symbole (Schaltplan-Blöcke).
  * *Parameter:* Ein einzelnes Keyword (`CELL` oder `BLOCK`).
  * *Beispiel:* `SymbolType CELL`

* **`SYMATTR <key> <value>`**
  * *Beschreibung:* Definiert ein Attribut des Symbols (z. B. Spice-Präfix, Modellname, Beschreibung).
  * *Parameter:* 
    1. `<key>`: Attributname (String ohne Leerzeichen, z. B. `Prefix`, `SpiceModel`, `Value`, `Description`).
    2. `<value>`: Attributwert (kann Leerzeichen enthalten, erstreckt sich bis zum Zeilenende).
  * *Beispiel:* `SYMATTR Prefix R`
  * *Beispiel:* `SYMATTR Description Resistor`

* **`WINDOW <index> <x> <y> <justification> <n>`**
  * *Beschreibung:* Steuert die Sichtbarkeit und Position der Attribut-Anzeige auf dem Symbol im Schaltplan.
  * *Parameter:*
    1. `<index>`: Integer-Index, der sich auf die Reihenfolge oder bestimmte Attribute bezieht.
    2. `<x>`, `<y>`: Integer-Koordinaten der Textbox relativ zum Symbolursprung.
    3. `<justification>`: Ausrichtung (z. B. `Left`, `Right`, `Center`, `Top`, `Bottom`, `VLeft`, `VRight`, etc.).
    4. `<n>`: Integer-Flag für Sichtbarkeit/Größe (häufig `2` für sichtbar, `0` für unsichtbar).
  * *Beispiel:* `WINDOW 0 32 32 Left 2`

---

### 2.3 Grafische Primitive

Grafische Primitive definieren das Aussehen des Symbols im Symbol-Editor und Schaltplan. Koordinaten (`x`, `y`) sind Ganzzahlen im LTspice-internen Koordinatensystem.

* **`LINE [Normal | Wide] <x1> <y1> <x2> <y2> [<linestyle>]`**
  * *Beschreibung:* Zeichnet eine gerade Linie von Startpunkt zu Endpunkt.
  * *Parameter:*
    * Optionaler Linientyp: `Normal` oder `Wide` (Standard ist `Normal`).
    * `<x1>`, `<y1>`: Startpunkt.
    * `<x2>`, `<y2>`: Endpunkt.
    * `<linestyle>`: Optionaler Integer (Standard `0`).
  * *Beispiel:* `LINE Normal -16 16 -16 -16`

* **`RECTANGLE [Normal | Wide] <x1> <y1> <x2> <y2> [<linestyle>]`**
  * *Beschreibung:* Zeichnet ein Rechteck über diagonal gegenüberliegende Ecken.
  * *Parameter:* Entsprechend `LINE`.
  * *Beispiel:* `RECTANGLE Normal -32 -32 32 32`

* **`CIRCLE [Normal | Wide] <x1> <y1> <x2> <y2> [<linestyle>]`**
  * *Beschreibung:* Zeichnet einen Kreis oder eine Ellipse, definiert durch ein umschreibendes Bounding-Box-Rechteck.
  * *Parameter:* Entsprechend `LINE`.
  * *Beispiel:* `CIRCLE Normal -8 -8 8 8`

* **`ARC [Normal | Wide] <x1> <y1> <x2> <y2> <x3> <y3> <x4> <y4> [<linestyle>]`**
  * *Beschreibung:* Zeichnet einen Bogen (Arc).
  * *Parameter:*
    * `<x1>, <y1>, <x2>, <y2>`: Bounding-Box der zugrundeliegenden Ellipse.
    * `<x3>, <y3>` und `<x4>, <y4>`: Start- und End-Vektoren zur Bestimmung des Start- und Endwinkels.
  * *Beispiel:* `ARC Normal -16 -16 16 16 0 16 16 0`

* **`TEXT <x> <y> <justification> <n> <ascii_text>`**
  * *Beschreibung:* Platziert statischen Text auf dem Symbol.
  * *Parameter:*
    * `<x>`, `<y>`: Position.
    * `<justification>`: Textausrichtung (`Left`, `Right`, `Center`, `Top`, `Bottom`, `VLeft`, etc.).
    * `<n>`: Skalierungs- oder Stil-Flag.
    * `<ascii_text>`: Der auszugebende Text (Rest der Zeile).
  * *Beispiel:* `TEXT 0 48 Center 2 VCC`

---

### 2.4 Pins und Pin-Attribute

Pins sind essenziell für die elektrische Konnektivität im Schaltplan.

* **`PIN <x> <y> <justification> <offset>`**
  * *Beschreibung:* Definiert einen elektrischen Anschlusspunkt (Pin) an den Koordinaten `(x, y)`. Der Befehl erzeugt implizit einen neuen Pin, auf den sich unmittelbar folgende `PINATTR`-Befehle beziehen.
  * *Parameter:*
    * `<x>`, `<y>`: Pin-Position.
    * `<justification>`: Ausrichtung des Pin-Labels (`LEFT`, `RIGHT`, `TOP`, `BOTTOM`, `VLEFT`, `VRIGHT`, `VTOP`, `VBOTTOM`, `NONE`).
    * `<offset>`: Abstand des Labels vom Pin-Punkt (Integer).
  * *Beispiel:* `PIN 0 64 BOTTOM 8`

* **`PINATTR <attr_name> <value>`**
  * *Beschreibung:* Weist dem **zuletzt definierten PIN** ein Attribut zu.
  * *Wichtige Standard-Attribute:*
    * `PinName`: Name des Pins (z. B. `IN`, `OUT`, `GND`, `1`, `2`).
    * `SpiceOrder`: Numerische Reihenfolge für die Netlist-Zuordnung (Integer, beginnend bei 1).
  * *Beispiel:* `PINATTR PinName A`
  * *Beispiel:* `PINATTR SpiceOrder 1`

---

## 3. Parser-Implementierungsleitfaden (Pseudocode-Struktur)

Ein AST (Abstract Syntax Tree) für eine `.asy`-Datei sollte folgende Strukturen abbilden:

```json
{
  "version": 4.0,
  "symbol_type": "CELL",
  "attributes": {
    "Prefix": "X",
    "SpiceModel": "my_model"
  },
  "windows": [
    { "index": 0, "x": 0, "y": 0, "justification": "Left", "n": 2 }
  ],
  "graphics": [
    { "type": "LINE", "style": "Normal", "x1": 0, "y1": 0, "x2": 10, "y2": 10 }
  ],
  "pins": [
    {
      "x": 0,
      "y": 32,
      "justification": "TOP",
      "offset": 8,
      "attributes": {
        "PinName": "1",
        "SpiceOrder": "1"
      }
    }
  ]
}
```

### Parsing-Logik (State Machine / Schleife):
1. Datei zeilenweise einlesen und Whitespace am Zeilenanfang/-ende bereinigen. Leerzeilen überspringen.
2. Erste Zeile prüfen: Muss mit `Version` beginnen.
3. Für jede Folgezeile das erste Token als Befehl extrahieren (Case-insensitive):
   * `SymbolType`: Wert als `symbol_type` speichern.
   * `SYMATTR`: Nächstes Token ist Key, der Rest der Zeile ist Value.
   * `WINDOW`: Parameter parsen und zur Windows-Liste hinzufügen.
   * `LINE`, `RECTANGLE`, `CIRCLE`, `ARC`: Prüfen, ob das erste Attribut `Normal`/`Wide` ist, Koordinaten einlesen, zur Grafik-Liste hinzufügen.
   * `TEXT`: Koordinaten, Justification, `n` und den restlichen Text-String parsen.
   * `PIN`: Neuen Pin-Datensatz anlegen, Koordinaten, Justification und Offset parsen, als "aktuellen Pin" merken.
   * `PINATTR`: Dem aktuell gemerkten Pin das Attribut (`attr_name` -> `value`) hinzufügen.