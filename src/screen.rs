/*
 * Copyright (c) 2014 Mathias Hällman
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

extern crate term;

use std::cmp;

#[deriving(PartialEq)]
pub struct Size(pub u16, pub u16);

impl Size {
  fn from_cell(Cell(row, col): Cell) -> Size {
    Size(row, col)
  }
}

pub struct Cell(pub u16, pub u16);

impl Cell {
  fn within(self, size: Size) -> Option<Cell> {
    let Cell(cell_row, cell_col) = self;
    let Size(end_row, end_col) = size;
    if cell_row < end_row && cell_col < end_col { Some(self) } else { None }
  }

  fn from_size(Size(row, col): Size) -> Cell {
    Cell(row, col)
  }
}

impl Add<Cell, Cell> for Cell {
  fn add(&self, rhs: &Cell) -> Cell {
    let Cell(r1, c1) = *self;
    let Cell(r2, c2) = *rhs;
    Cell(r1 + r2, c1 + c2)
  }
}

impl Sub<Cell, Cell> for Cell {
  fn sub(&self, rhs: &Cell) -> Cell {
    let Cell(r1, c1) = *self;
    let Cell(r2, c2) = *rhs;
    Cell(cmp::max(r1 as i16 - r2 as i16, 0) as u16,
         cmp::max(c1 as i16 - c2 as i16, 0) as u16)
  }
}

/*
 * Iterates over a region of the screen, defined by a starting cell and a size.
 */
pub struct CellIterator {
  next_cell: Option<Cell>,
  size: Size,
  width: u16,
}

impl CellIterator {
  pub fn new(start: Cell, size: Size) -> CellIterator {
    let Size(_, rel_end_col) = size;
    let abs_size = Size::from_cell(start + Cell::from_size(size));
    CellIterator {
      next_cell: start.within(abs_size), size: abs_size, width: rel_end_col
    }
  }
}

impl Iterator<Cell> for CellIterator {
  fn next(&mut self) -> Option<Cell> {
    let ret = self.next_cell;
    self.next_cell = self.next_cell.and_then(|cell|
      (cell + Cell(0, 1)).within(self.size).or(
        (cell - Cell(0, self.width - 1) + Cell(1, 0)).within(self.size)));
    return ret;
  }
}

/*
 * Screen is the output surface. You can put characters within its borders and
 * clear it again. Go nuts!
 */
pub struct Screen {
  size: Size,
  terminal: Terminal,
}

impl Drop for Screen {
  fn drop(&mut self) {
    self.terminal.clear();
    self.terminal.show_cursor();
    self.terminal.disable_altscreen();
  }
}

impl Screen {
  pub fn setup() -> Result<Screen, String> {
    Terminal::new().map_or(
      Err(String::from_str("Failed creating a terminal for stdout.")),
      |mut terminal| {
        terminal.enable_altscreen();
        terminal.hide_cursor();
        terminal.clear();
        Ok(Screen { size: Size(0, 0), terminal: terminal })
      })
  }

  pub fn update_size(&mut self) -> bool {
    match term_size::size().map(|(rows, cols)| Size(rows, cols)) {
      None               => return false,
      Some(current_size) => {
        let size_changed = current_size != self.size;
        self.size = current_size;
        return size_changed;
      }
    }
  }

  pub fn size(&self) -> Size {
    self.size
  }

  pub fn clear(&mut self) {
    self.terminal.clear();
  }

  pub fn put(&mut self, position: Cell, character: char,
             fg: color::Color, bg: color::Color) {
    position.within(self.size).map(|Cell(row, col)| {
      self.terminal.set_cursor_position(row, col);
      self.terminal.set_fg(fg);
      self.terminal.set_bg(bg);
      self.terminal.put(character);
    });
  }

  pub fn flush(&mut self) {
    self.terminal.flush();
  }
}

/*
 * Terminal is a simple wrapper that provides some helpful methods for common
 * ouput operations.
 */
struct Terminal {
  terminal: Box<term::Terminal<term::WriterWrapper> + Send>,
}

impl Terminal {
  pub fn new() -> Option<Terminal> {
    term::stdout().map(|terminal| Terminal { terminal: terminal })
  }

  pub fn set_fg(&mut self, fg: color::Color) {
    self.terminal.fg(fg.to_term_color()).unwrap();
  }

  pub fn set_bg(&mut self, bg: color::Color) {
    self.terminal.bg(bg.to_term_color()).unwrap();
  }

  pub fn clear(&mut self) {
    (write!(self.terminal, "\x1B[2J")).unwrap();
  }

  pub fn enable_altscreen(&mut self) {
    (write!(self.terminal, "\x1B7\x1B[?47h")).unwrap();
  }

  pub fn disable_altscreen(&mut self) {
    (write!(self.terminal, "\x1B[?47l\x1B8")).unwrap();
  }

  pub fn hide_cursor(&mut self) {
    (write!(self.terminal, "\x1B[?25l")).unwrap();
  }

  pub fn show_cursor(&mut self) {
    (write!(self.terminal, "\x1B[?25h")).unwrap();
  }

  pub fn set_cursor_position(&mut self, row: u16, col: u16) {
    // add (1, 1) becase terminal row/col is one-indexed
    (write!(self.terminal, "\x1B[{};{}H", row + 1, col + 1)).unwrap();
  }

  pub fn put(&mut self, character: char) {
    (write!(self.terminal, "{}", character)).unwrap();
  }

  pub fn flush(&mut self) {
    self.terminal.flush().unwrap();
  }
}

/*
 * Color values for terminal output.
 */
#[allow(dead_code)]  // colors are not used much yet
pub mod color {
  extern crate term;

  pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
  }

  impl Color {
    pub fn to_term_color(&self) -> term::color::Color {
      match *self {
        Black         => term::color::BLACK,
        Red           => term::color::RED,
        Green         => term::color::GREEN,
        Yellow        => term::color::YELLOW,
        Blue          => term::color::BLUE,
        Magenta       => term::color::MAGENTA,
        Cyan          => term::color::CYAN,
        White         => term::color::WHITE,
        BrightBlack   => term::color::BRIGHT_BLACK,
        BrightRed     => term::color::BRIGHT_RED,
        BrightGreen   => term::color::BRIGHT_GREEN,
        BrightYellow  => term::color::BRIGHT_YELLOW,
        BrightBlue    => term::color::BRIGHT_BLUE,
        BrightMagenta => term::color::BRIGHT_MAGENTA,
        BrightCyan    => term::color::BRIGHT_CYAN,
        BrightWhite   => term::color::BRIGHT_WHITE,
      }
    }
  }
}

/*
 * Helper module to capture the ugly. Provides a mean to poll the screen size.
 */
mod term_size {
  extern crate libc;

  use self::libc::funcs::bsd44::ioctl;

  const STDOUT_FILENO: libc::c_int = 1;

  #[cfg(target_os = "macos")]
  const TIOCGWINSZ: libc::c_ulong = 0x40087468;
  #[cfg(target_os = "linux")]
  const TIOCGWINSZ: libc::c_int = 0x5413;

  pub fn size() -> Option<(u16, u16)> {
    #[allow(dead_code)]  // not interested in pixel sizes
    struct WinSize {
      rows: libc::c_ushort,  // rows, in screen cells
      cols: libc::c_ushort,  // columns, in screen cells
      h_pixels: libc::c_ushort,  // horizontal size, pixels
      v_pixels: libc::c_ushort,  // vertical size, pixels
    }

    unsafe {
      let mut size = WinSize { rows: 0, cols: 0, h_pixels: 0, v_pixels: 0 };
      match ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut size) {
        0 => Some((size.rows as u16, size.cols as u16)),
        _ => None,
      }
    }
  }
}