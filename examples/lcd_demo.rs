use std::{error::Error, thread::sleep, time::Duration};

use char_lcd_rgb_i2c::{CharLCDRGBI2C, LcdError};

fn main() -> Result<(), Box<dyn Error>> {
    println!("Setting up I2C and RGB1602 LCD...");

    // Create LCD object (16 columns, 2 rows)
    let mut lcd = CharLCDRGBI2C::new(16, 2)?;

    hello_world_demo(&mut lcd)?;
    led_demo(&mut lcd)?;
    backlight_demo(&mut lcd)?;

    Ok(())
}

fn hello_world_demo(lcd: &mut CharLCDRGBI2C) -> Result<(), LcdError> {
    println!("Starting Hello World demo");

    lcd.message("Hello World!")?;
    Ok(())
}

fn backlight_demo(lcd: &mut CharLCDRGBI2C) -> Result<(), LcdError> {
    println!("Starting Backlight demo");

    println!("Turning backlight OFF");
    lcd.set_backlight(false)?;
    sleep(Duration::from_secs(2));

    lcd.message("Backlight ON")?;
    println!("Turning backlight ON");
    lcd.set_backlight(true)?;
    sleep(Duration::from_secs(2));

    println!("Turning backlight OFF again");
    lcd.set_backlight(false)?;
    lcd.clear()?;

    Ok(())
}

fn led_demo(lcd: &mut CharLCDRGBI2C) -> Result<(), LcdError> {
    println!("Starting RGB LED Demo");

    // Define a map of colors
    let color_map: &[(&str, [u8; 3])] = &[
        ("Red", [100, 0, 0]),
        ("Green", [0, 100, 0]),
        ("Blue", [0, 0, 100]),
        ("Purple", [50, 0, 50]),
        ("Cyan", [0, 50, 50]),
        ("Yellow", [50, 50, 0]),
        ("White (dim)", [50, 50, 50]),
    ];

    for (color, color_values) in color_map.iter() {
        println!(
            "Setting color to: R={}, G={}, B={} - {}",
            color_values[0], color_values[1], color_values[2], color
        );
        lcd.set_color(color_values[0], color_values[1], color_values[2])?;
        sleep(Duration::from_secs(1));
    }

    // Turn off all LEDs
    lcd.set_color(0, 0, 0)?;

    Ok(())
}
