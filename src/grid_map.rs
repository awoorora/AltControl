use rdev::Key;
use std::collections::HashMap;
use std::sync::OnceLock;

/**
El mucho texto

On Alt+Left Ctrl key combo, open up an overlay. This overlay is a grid as follows:
First row: AQ SQ DQ ... LQ ;Q
Second row: AW SW DW ... LW ;W
...
8th row: AI SI DI ... LI ;I
...
Last row: A/ S/ D/ ... ;/

This means 10 columns and 30 rows.
On a 1920x1080 screen, a cell would take up 192x36 pixels. When moving to a cell, the mouse will move in the middle of it. Assuming each cell is mapped to an X and Y coordinate, [A B] means column A will point to x = 0 * 192, row B means y = 24 * 30 (remember! alphabetical order != qwerty order. Also remember! Only the middle keyboard row (a, s, ..., l, ;) have valid x coords! If the user first inputs a key without an x coord, exit overlay, write a message in console at most)

Basically the rows follow the middle row of the qwerty keyboard (A S D F ... L ;) and the columns follow the entire qwerty layout (q w e r ... p a s ... l ; z x ... m , . /), with Left Ctrl as Left Click and Alt as Right Click
Once the overlay is open, input from the user goes like this:
Optional: The user presses Left Ctrl or Alt to click where the mouse is already at, overlay closes, back to monitoring mode
The user pressed D: the mouse moves to the middle of the D column.
Optional: The user presses Left Ctrl or Alt to click where the mouse is already at (middle of D column), overlay closes, back to monitoring mode
The user presses W: the mouse moves to the specific DW cell, in the middle of it, somewhere in the top left
The user presses Left Ctrl or Alt to click where the mouse is already at (DW cell), overlay closes, back to monitoring mode

If the user presses any key on the left side of the qwerty keyboard, the mouse moves to the center of the first third of the cell. If the user presses any key on the right side of the qwerty keyboard, the mouse moves to the center of the last third of the cell. If the user presses space, the mouse remains in the center of the cell. This assumes the cells are "split" into thirds, one row, three columns, though this is not shown, as it would clutter an already cluttered screen

The overlay automatically closes itself.

Esc closes the overlay
A non-column key pressed at overlay start closes the overlay
Unrecognized keys close the overlay
*/

    pub struct GridEngine{
        pub screen_w: f32,
        pub screen_h: f32
    }

    static COL_MAP: OnceLock<HashMap<Key, usize>> = OnceLock::new();
    static ROW_MAP: OnceLock<HashMap<Key, usize>> = OnceLock::new();

    impl GridEngine{
        pub fn new(w: f32, h: f32) -> Self{
            Self::init_maps();
            GridEngine{screen_w: w, screen_h: h}
        }

        fn init_maps() {
            ROW_MAP.get_or_init(|| {
                let mut m = HashMap::new();
                let row_selector_keys = vec![
                    // QWERTY
                    Key::KeyQ, Key::KeyW, Key::KeyE, Key::KeyR, Key::KeyT, Key::KeyY, Key::KeyU, Key::KeyI, Key::KeyO, Key::KeyP,
                    // ASDF
                    Key::KeyA, Key::KeyS, Key::KeyD, Key::KeyF, Key::KeyG, Key::KeyH, Key::KeyJ, Key::KeyK, Key::KeyL, Key::SemiColon,
                    // ZXCV
                    Key::KeyZ, Key::KeyX, Key::KeyC, Key::KeyV, Key::KeyB, Key::KeyN, Key::KeyM, Key::Comma, Key::Dot, Key::Slash
                ];
                for (index, key) in row_selector_keys.into_iter().enumerate() {
                    m.insert(key, index);
                }
                m
            });

            COL_MAP.get_or_init(|| {
                let mut m = HashMap::new();
                let col_selector_keys = vec![
                    Key::KeyA, Key::KeyS, Key::KeyD, Key::KeyF, Key::KeyG,
                    Key::KeyH, Key::KeyJ, Key::KeyK, Key::KeyL, Key::SemiColon
                ];
                for (index, key) in col_selector_keys.into_iter().enumerate() {
                    m.insert(key, index);
                }
                m
            });
        }

        pub fn get_coords(&self, col_key: Key, row_key: Key) -> Option<(f32, f32)>{
            let col_idx = COL_MAP.get()?.get(&col_key)?;
            let row_idx = ROW_MAP.get()?.get(&row_key)?;

            println!("DEBUG: Col Key: {:?} -> Index: {:?}, Row Key: {:?} -> Index: {:?}", col_key, col_idx, row_key, row_idx);

            let cell_w = self.screen_w / 10.0;
            let cell_h = self.screen_h / 30.0;

            println!("DEBUG: Cell W: {}, Cell H: {}", cell_w, cell_h);

            let x = (*col_idx as f32 * cell_w) + (cell_w / 2.0);
            let y = (*row_idx as f32 * cell_h) + (cell_h / 2.0);

            Some((x,y))
        }
    }