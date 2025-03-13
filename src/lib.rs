use std::{error::Error, fmt, thread::sleep, time::Duration};

use mcp230xx::{Direction, Level, Mcp23017, Mcp230xx, PullUp};
use rppal::i2c::I2c;

pub const ADDR: u8 = 0x20; // Default I2C address

// MCP23017 registers
pub const IODIRA: u8 = 0x00; // I/O direction register for Port A
pub const IODIRB: u8 = 0x01; // I/O direction register for Port B
pub const GPIOA: u8 = 0x12; // GPIO register for Port A
pub const GPIOB: u8 = 0x13; // GPIO register for Port B

// MCP23017 pin mappings based on Python library
pub const LCD_RS: Mcp23017 = Mcp23017::B7; // Register Select (RS)
pub const LCD_E: Mcp23017 = Mcp23017::B5; // Enable (E)
pub const LCD_D4: Mcp23017 = Mcp23017::B4;
pub const LCD_D5: Mcp23017 = Mcp23017::B3;
pub const LCD_D6: Mcp23017 = Mcp23017::B2;
pub const LCD_D7: Mcp23017 = Mcp23017::B1;
pub const LCD_RW: Mcp23017 = Mcp23017::B6; // Read/Write (RW)

// MCP23017 pins for RGB LED
pub const RGB_RED: Mcp23017 = Mcp23017::A6;
pub const RGB_GREEN: Mcp23017 = Mcp23017::A7;
pub const RGB_BLUE: Mcp23017 = Mcp23017::B0;
pub const LCD_BACKLIGHT: Mcp23017 = Mcp23017::A5;

// MCP23017 pins for Buttons
pub const BTN_LEFT: Mcp23017 = Mcp23017::A4;
pub const BTN_UP: Mcp23017 = Mcp23017::A3;
pub const BTN_DOWN: Mcp23017 = Mcp23017::A2;
pub const BTN_RIGHT: Mcp23017 = Mcp23017::A1;
pub const BTN_SELECT: Mcp23017 = Mcp23017::A0;

// LCD commands
pub const LCD_CLEARDISPLAY: u8 = 0x01;
pub const LCD_RETURNHOME: u8 = 0x02;
pub const LCD_ENTRYMODESET: u8 = 0x04;
pub const LCD_DISPLAYCONTROL: u8 = 0x08;
pub const LCD_CURSORSHIFT: u8 = 0x10;
pub const LCD_FUNCTIONSET: u8 = 0x20;
pub const LCD_SETCGRAMADDR: u8 = 0x40;
pub const LCD_SETDDRAMADDR: u8 = 0x80;

// Entry flags
pub const LCD_ENTRYLEFT: u8 = 0x02;
pub const LCD_ENTRYSHIFTDECREMENT: u8 = 0x00;

// Control flags
pub const LCD_DISPLAYON: u8 = 0x04;
pub const LCD_CURSORON: u8 = 0x02;
pub const LCD_CURSOROFF: u8 = 0x00;
pub const LCD_BLINKON: u8 = 0x01;
pub const LCD_BLINKOFF: u8 = 0x00;

// Move flags
pub const LCD_DISPLAYMOVE: u8 = 0x08;
pub const LCD_MOVERIGHT: u8 = 0x04;
pub const LCD_MOVELEFT: u8 = 0x00;

// Function set flags
pub const LCD_4BITMODE: u8 = 0x00;
pub const LCD_2LINE: u8 = 0x08;
pub const LCD_1LINE: u8 = 0x00;
pub const LCD_5X8DOTS: u8 = 0x00;

// Direction constants
pub const LEFT_TO_RIGHT: usize = 0;
pub const RIGHT_TO_LEFT: usize = 1;

// Row offset addresses for different LCD lines
pub const LCD_ROW_OFFSETS: [u8; 4] = [0x00, 0x40, 0x14, 0x54];

// Custom error type
#[derive(Debug)]
pub enum LcdError {
    I2c(rppal::i2c::Error),
    Mcp(String),
    Other(String),
}

impl From<rppal::i2c::Error> for LcdError {
    fn from(err: rppal::i2c::Error) -> Self {
        LcdError::I2c(err)
    }
}

impl fmt::Display for LcdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LcdError::I2c(err) => write!(f, "I2C error: {}", err),
            LcdError::Mcp(msg) => write!(f, "MCP23017 error: {}", msg),
            LcdError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for LcdError {}

pub struct CharLCDRGBI2C {
    mcp: Mcp230xx<I2c, Mcp23017>,
    columns: usize,
    lines: usize,
    backlight: bool,    // Backlight status
    rgb: [Mcp23017; 3], // RGB pins
    color_value: [u8; 3],
    display_control: u8,
    display_mode: u8,
    display_function: u8,
    row: usize,
    column: usize,
    column_align: bool,
    message: String,
    direction: usize,
}

impl CharLCDRGBI2C {
    pub fn new(columns: usize, lines: usize) -> Result<Self, LcdError> {
        // Initialize I2C
        let i2c = I2c::new()?;

        // Use map_err for the MCP error conversion
        let mcp = Mcp230xx::<I2c, Mcp23017>::new_default(i2c)
            .map_err(|e| LcdError::Mcp(format!("{:?}", e)))?;

        let mut lcd = CharLCDRGBI2C {
            mcp,
            columns,
            lines,
            backlight: true,
            rgb: [RGB_RED, RGB_GREEN, RGB_BLUE],
            color_value: [0, 0, 0],
            display_control: 0,
            display_mode: 0,
            display_function: 0,
            row: 0,
            column: 0,
            column_align: false,
            message: String::new(),
            direction: 0, // Assuming 0 for LEFT_TO_RIGHT
        };

        lcd.setup_pins()?;
        lcd.initialize()?;

        Ok(lcd)
    }

    fn setup_pins(&mut self) -> Result<(), LcdError> {
        // Set LCD control pins as outputs
        for pin in [LCD_RS, LCD_E, LCD_D4, LCD_D5, LCD_D6, LCD_D7, LCD_RW] {
            self.mcp.set_direction(pin, Direction::Output)?;
        }

        // Set RGB LED pins as outputs
        for pin in [RGB_RED, RGB_GREEN, RGB_BLUE] {
            self.mcp.set_direction(pin, Direction::Output)?;
        }

        // Set Button pins as inputs with pull-up
        for pin in [BTN_LEFT, BTN_UP, BTN_DOWN, BTN_RIGHT, BTN_SELECT] {
            self.mcp.set_direction(pin, Direction::Input)?;
            self.mcp.set_pull_up(pin, PullUp::Enabled)?;
        }

        Ok(())
    }

    fn initialize(&mut self) -> Result<(), LcdError> {
        // Wait for LCD to be ready
        sleep(Duration::from_millis(50));

        // Pull RS low to begin commands
        self.mcp.set_output_latch(LCD_RS, Level::Low)?;
        self.mcp.set_output_latch(LCD_E, Level::Low)?;
        self.mcp.set_output_latch(LCD_RW, Level::Low)?;

        // 4-bit mode initialization sequence
        self.write4bits(0x03)?;
        sleep(Duration::from_millis(5));
        self.write4bits(0x03)?;
        sleep(Duration::from_millis(5));
        self.write4bits(0x03)?;
        sleep(Duration::from_millis(1));
        self.write4bits(0x02)?; // Set to 4-bit mode
        sleep(Duration::from_millis(1));

        // Initialize display control
        self.display_control = LCD_DISPLAYON | LCD_CURSOROFF | LCD_BLINKOFF;
        self.display_function = LCD_4BITMODE | LCD_1LINE | LCD_2LINE | LCD_5X8DOTS;
        self.display_mode = LCD_ENTRYLEFT | LCD_ENTRYSHIFTDECREMENT;

        // Write to display control
        self.write_command(LCD_DISPLAYCONTROL | self.display_control)?;
        // Write to display function
        self.write_command(LCD_FUNCTIONSET | self.display_function)?;
        // Set entry mode
        self.write_command(LCD_ENTRYMODESET | self.display_mode)?;

        // Clear display
        self.clear()?;

        // Initialize tracking variables
        self.row = 0;
        self.column = 0;
        self.column_align = false;
        self.direction = LEFT_TO_RIGHT;
        self.message = String::new();

        // Turn off all RGB LEDs initially
        self.set_color(0, 0, 0)?;

        Ok(())
    }

    fn write4bits(&mut self, value: u8) -> Result<(), LcdError> {
        // Set data pins
        self.mcp.set_output_latch(
            LCD_D4,
            if value & 0x01 > 0 {
                Level::High
            } else {
                Level::Low
            },
        )?;
        self.mcp.set_output_latch(
            LCD_D5,
            if value & 0x02 > 0 {
                Level::High
            } else {
                Level::Low
            },
        )?;
        self.mcp.set_output_latch(
            LCD_D6,
            if value & 0x04 > 0 {
                Level::High
            } else {
                Level::Low
            },
        )?;
        self.mcp.set_output_latch(
            LCD_D7,
            if value & 0x08 > 0 {
                Level::High
            } else {
                Level::Low
            },
        )?;

        // Pulse the enable pin
        self.pulse_enable()?;

        Ok(())
    }

    fn pulse_enable(&mut self) -> Result<(), LcdError> {
        self.mcp.set_output_latch(LCD_E, Level::Low)?;
        sleep(Duration::from_micros(1));
        self.mcp.set_output_latch(LCD_E, Level::High)?;
        sleep(Duration::from_micros(1));
        self.mcp.set_output_latch(LCD_E, Level::Low)?;
        sleep(Duration::from_micros(100)); // Commands need > 37us to settle
        Ok(())
    }

    fn write8(&mut self, value: u8, char_mode: bool) -> Result<(), LcdError> {
        // Set the RS pin based on char_mode
        self.mcp
            .set_output_latch(LCD_RS, if char_mode { Level::High } else { Level::Low })?;

        // Send upper 4 bits
        self.write4bits(value >> 4)?;
        // Send lower 4 bits
        self.write4bits(value & 0x0F)?;
        Ok(())
    }

    fn write_command(&mut self, value: u8) -> Result<(), LcdError> {
        self.write8(value, false)?;
        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), LcdError> {
        self.write_command(LCD_CLEARDISPLAY)?;
        sleep(Duration::from_millis(3));
        self.row = 0;
        self.column = 0;
        Ok(())
    }

    pub fn home(&mut self) -> Result<(), LcdError> {
        self.write_command(LCD_RETURNHOME)?;
        sleep(Duration::from_millis(3));
        self.row = 0;
        self.column = 0;
        Ok(())
    }

    pub fn set_color(&mut self, r: u8, g: u8, b: u8) -> Result<(), LcdError> {
        // Any value > 1 turns LED on (inverse of Python logic)
        // LOW = on for common anode RGB LED
        self.mcp
            .set_output_latch(self.rgb[0], if r > 1 { Level::Low } else { Level::High })?; // R
        self.mcp
            .set_output_latch(self.rgb[1], if g > 1 { Level::Low } else { Level::High })?; // G
        self.mcp
            .set_output_latch(self.rgb[2], if b > 1 { Level::Low } else { Level::High })?; // B

        self.color_value = [r, g, b];
        Ok(())
    }

    pub fn set_cursor(&mut self, col: usize, row: usize) -> Result<(), LcdError> {
        let row_offsets = [0x00, 0x40, 0x14, 0x54]; // For 16x2 or 20x4 LCD

        if row >= self.lines {
            return Err(LcdError::Other(format!(
                "Row {} is invalid for a {}-line display",
                row, self.lines
            )));
        }

        let command = LCD_SETDDRAMADDR | (col as u8 + row_offsets[row]);
        self.write_command(command)?;

        self.row = row;
        self.column = col;
        Ok(())
    }

    pub fn set_backlight(&mut self, on: bool) -> Result<(), LcdError> {
        if on {
            self.mcp.set_direction(LCD_BACKLIGHT, Direction::Output)?;
            self.backlight = true;
            println!("Backlight ON")
        } else {
            self.mcp.set_direction(LCD_BACKLIGHT, Direction::Input)?;
            self.backlight = false;
            println!("Backlight OFF")
        }
        Ok(())
    }

    pub fn cursor_position(&mut self, mut column: usize, mut row: usize) -> Result<(), LcdError> {
        if row >= self.lines {
            row = self.lines - 1;
        }
        if column >= self.columns {
            column = self.columns - 1;
        }
        self.write_command(LCD_SETDDRAMADDR | (column as u8 + LCD_ROW_OFFSETS[row]))?;
        self.row = row;
        self.column = column;
        Ok(())
    }

    pub fn message(&mut self, message: &str) -> Result<(), LcdError> {
        self.message = message.to_string();

        let mut line = self.row;
        let mut initial_character = 0;

        for char in message.chars() {
            if initial_character == 0 {
                let col;
                if self.display_mode & LCD_ENTRYLEFT > 0 {
                    col = self.column;
                } else {
                    col = self.columns - 1 - self.column;
                }
                self.cursor_position(col, line)?;
                initial_character += 1;
            }

            if char == '\n' {
                line += 1;
                let col;
                if self.display_mode & LCD_ENTRYLEFT > 0 {
                    if self.column_align {
                        col = self.column;
                    } else {
                        col = 0;
                    }
                } else {
                    if self.column_align {
                        col = self.column;
                    } else {
                        col = self.columns - 1;
                    }
                }
                self.cursor_position(col, line)?;
            } else {
                self.write8(char as u8, true)?;
            }
        }

        self.column = 0;
        self.row = 0;

        Ok(())
    }
}
