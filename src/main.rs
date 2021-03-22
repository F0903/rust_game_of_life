use crossterm::{cursor, style, terminal, QueueableCommand, Result};
use std::io::{Stdout, Write};

#[cfg(windows)]
use winapi::{
	shared::minwindef,
	um::{errhandlingapi, processenv, winbase, wincon, wincontypes, winuser},
};

const WIDTH: u16 = 200;
const HEIGHT: u16 = 100;

const SPAWN_CHANCE: f32 = 1f32 / 100f32;

const CELL: &[u8] = &[0xE2, 0x96, 0xA0];

struct Cell(u16, u16);

#[cfg(windows)]
macro_rules! assert_win32_err {
	($res:ident, $msg:ident) => {
		if $res == 0 {
			#[cfg(not(debug_assertions))]
			std::fs::write("./error.txt", &$msg).unwrap();

			let mut out = std::io::stdout();
			out.write_all($msg.as_bytes()).unwrap();
			out.flush().unwrap();
			std::thread::sleep(std::time::Duration::from_millis(3000));

			panic!();
		}
	};

	($res:ident) => {
		if $res == 0 {
			let code = unsafe { errhandlingapi::GetLastError() };
			let errmsg = format!("Win32 error assert failed with code {}.", code);
			assert_win32_err!($res, errmsg);
		}
	};

	{$res:expr;} => {
		if $res == 0 {
			let code = unsafe { errhandlingapi::GetLastError() };
			let errmsg = format!("Win32 call failed with code {} in {} at line {} column {}\nExplanation: {}", code, std::file!(), std::line!(), std::column!(), get_err_desc(code));
			let val = $res;
			assert_win32_err!(val, errmsg);
		}
	};

	{$res:expr; $($resv:expr);+;} => {
		assert_win32_err!{$res;};
		assert_win32_err!{$($resv);+;};
	};
}

#[cfg(windows)]
fn get_err_desc(errcode: minwindef::DWORD) -> String {
	const BUFSIZE: usize = 512;
	let mut buf: [u8; BUFSIZE] = [0; BUFSIZE];
	let count = unsafe {
		winbase::FormatMessageA(
			winbase::FORMAT_MESSAGE_FROM_SYSTEM | winbase::FORMAT_MESSAGE_IGNORE_INSERTS,
			std::ptr::null(),
			errcode,
			0,
			buf.as_mut_ptr() as *mut i8,
			BUFSIZE as u32,
			std::ptr::null_mut(),
		)
	};
	assert_win32_err!(count);
	let string = unsafe { std::str::from_utf8_unchecked(&buf[..count as usize]) };
	String::from(string)
}

#[cfg(windows)]
fn init_window() {
	unsafe {
		let con_win = wincon::GetConsoleWindow();
		assert_win32_err! {
			winuser::ShowScrollBar(con_win, winuser::SB_VERT as i32, 0);
		};

		let stdout = processenv::GetStdHandle(winbase::STD_OUTPUT_HANDLE);

		let info: *mut wincon::CONSOLE_SCREEN_BUFFER_INFO = std::ptr::null_mut();
		assert_win32_err! {
			wincon::GetConsoleScreenBufferInfo(stdout, info);
		}

		let new_size = wincontypes::COORD {
			X: (*info).dwSize.X - 2,
			Y: (*info).dwSize.Y,
		};
		assert_win32_err! {
			wincon::SetConsoleScreenBufferSize(stdout, new_size);
		}
	}
}

fn init_term(term: &mut Stdout) -> Result<()> {
	terminal::enable_raw_mode()?;
	term.queue(terminal::SetSize(WIDTH, HEIGHT))?
		.queue(terminal::Clear(terminal::ClearType::All))?
		.queue(style::SetForegroundColor(style::Color::Green))?
		.queue(style::SetAttribute(style::Attribute::NoBlink))?
		.queue(cursor::Hide)?
		.queue(terminal::DisableLineWrap)?
		.flush()?;
	Ok(())
}

fn draw_cell(term: &mut Stdout, cell: &Cell) -> Result<()> {
	term.queue(cursor::MoveTo(cell.0, cell.1))?
		.write_all(CELL)?;
	term.flush()?;
	Ok(())
}

fn clear_cell(term: &mut Stdout, cell: &Cell) -> Result<()> {
	term.queue(cursor::MoveTo(cell.0, cell.1))?
		.write_all(b" ")?;
	term.flush()?;
	Ok(())
}

//Note: Panics if run with VSCode debugger.
fn main() -> Result<()> {
	let mut terminal = std::io::stdout();
	let mut cells = Vec::<Cell>::new();

	#[cfg(windows)]
	init_window();
	init_term(&mut terminal)?;

	for x in 0..WIDTH {
		for y in 0..HEIGHT {
			let rng = (rand::random::<f32>() * 100f32) as i32;
			let spawn = rng < (SPAWN_CHANCE * 100f32) as i32;
			if !spawn {
				continue;
			}

			let cell = Cell(x, y);
			draw_cell(&mut terminal, &cell)?;
			cells.push(cell);
		}
	}
	loop {
		std::thread::sleep(std::time::Duration::from_millis(1000));
	}
}
