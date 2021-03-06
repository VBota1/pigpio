// Copyright (c) 2017 Viorel Bota
// 
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// 
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
// 
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

extern crate rppal;

use std::env;
use std::ffi::OsString;
use rppal::gpio::{GPIO,Mode,Level};

const LAST_PIN_NUMBER: u8 = 27;
const HELP_CMD_STR: &'static str = "help";
const SET_CMD_STR: &'static str = "set";
const GET_CMD_STR: &'static str = "get";
const HIGH_PIN_STATE_STR: &'static str = "high";
const LOW_PIN_STATE_STR: &'static str = "low";
const ALL_PINS_STR: &'static str = "all";
#[cfg(ignore_PATH)]
const APP_CALL_STR: &'static str = "./pi3gpio";
#[cfg(not(ignore_PATH))]
const APP_CALL_STR: &'static str = "pi3gpio";
#[cfg(ignore_PATH)]
const HELP_SUGGEST_STR: &'static str = "For help run \"sudo ./pi3gpio help\"";
#[cfg(not(ignore_PATH))]
const HELP_SUGGEST_STR: &'static str = "For help run \"sudo pi3gpio help\"";

fn main() {
	handle_cmd ( env::args_os() );
}

fn handle_cmd (mut cmd: env::ArgsOs) {
	match cmd.nth(1) {
		Some (action) => {
			if action == OsString::from(HELP_CMD_STR) {
				print_help_text ();
			}
			else if action == OsString::from(SET_CMD_STR) {
				set_pins (cmd); 
			}
			else if action == OsString::from(GET_CMD_STR) {
				read_pins (cmd);
			}
			else {
				panic! ("Unknown command {:?}! \n{}", action, HELP_SUGGEST_STR );
			}
		},
		None => panic! ("No command found! \n{}", HELP_SUGGEST_STR ),
	}
}

fn print_help_text () {
	println! ("SYNTAX: sudo {} command [state] [pins]", APP_CALL_STR );
	println! ("	command: ");
	println! ("		{}	prints the help text", HELP_CMD_STR );
	println! ("		{}	sets the value of the indicated pins to the indicated value", SET_CMD_STR );
	println! ("		{}	prints the state of the indicated pins", GET_CMD_STR );
	println! ("	state: ");
	println! ("		{}	logical 1 for the indicated pins, equivalent voltage 3.3[V]", HIGH_PIN_STATE_STR );
	println! ("		{}	logical 0 for the indicated pins, equivalent voltage 0[V]", LOW_PIN_STATE_STR );
	println! ("	pins: ");
	println! ("		{}	BCM pins between 0 and {}", ALL_PINS_STR, LAST_PIN_NUMBER );
	println! ("Examples:");
	println! ("	sudo {} help ", APP_CALL_STR );
	println! ("	sudo {} get all", APP_CALL_STR );
	println! ("	sudo {} get 4", APP_CALL_STR );
	println! ("	sudo {} get 10 11", APP_CALL_STR );
	println! ("	sudo {} set low all", APP_CALL_STR );
	println! ("	sudo {} set high 12", APP_CALL_STR );
	println! ("	sudo {} set low 2 5 7", APP_CALL_STR );
	#[cfg(not(ignore_PATH))]
	{
		println! ("Access:");
		println! ("	The application needs access to /dev/mem or /dev/gpiomem to control GPIO pins. This is why sudo is required.");
		println! ("	In order to gain access to /dev/mem you need run, only once after installantion, the following commands:");
		println! ("		sudo snap connect pi3gpio:physical-memory-control core:physical-memory-control");
		println! ("		sudo snap connect pi3gpio:physical-memory-observe core:physical-memory-observe");
	}
}

trait ReadState{
	fn read_state (self, gpio: &mut GPIO);
}

impl ReadState for OsString {
	fn read_state (self, gpio: &mut GPIO) {
		match self.to_u8() {
			Ok (pin) => read_pin ( gpio, pin ),
			Err (e) => {
				panic! ("{} \n{}", e,  HELP_SUGGEST_STR );		
			},
		}		
	}
}

fn read_all_pins ( gpio: &mut GPIO ) {
	for pin in 0..(LAST_PIN_NUMBER+1) {
		read_pin ( gpio, pin );
	}
}

fn read_pin ( gpio: &mut GPIO, pin: u8) {
	gpio.set_mode( pin, Mode::Input );
	match gpio.read(pin) {
		Ok(state) =>	println! ("Pin {} = {:?}", pin, state ),
		Err (_) => println! ("Could not read state of pin {}",pin),
	}
}

fn read_pins (mut cmd: env::ArgsOs) {
	let mut gpio = initiate_gpio_handle();
	
	match cmd.nth(0) {
		Some (pin_as_os_string) => {
			if pin_as_os_string == OsString::from(ALL_PINS_STR) {
				read_all_pins ( &mut gpio );
				return;
			}
			else {
				pin_as_os_string.read_state ( &mut gpio );
			}
		}
		None => panic! ("Pin not found! \n{}",  HELP_SUGGEST_STR ),		
	}

	for argument in cmd {
		argument.read_state ( &mut gpio );
	}
	
}	

trait ToU8 {
	fn to_u8 (self) -> Result<u8, &'static str >;
}

impl ToU8 for OsString {
	fn to_u8 (self) -> Result<u8, &'static str> {
		match self.into_string() {
			Ok(x) => match x.parse::<u8>() {
				Ok (y) => return Ok (y),
				Err(_) => return Err("String indicating pin number could not be converted to u8!" ),
			},
			Err(_) => return Err("OsString indicating pin number could not be converted to String!" ),		
		}
	}
}

fn initiate_gpio_handle () -> GPIO {
    let gpio: GPIO;
    match GPIO::new(){
	    Ok (x) => gpio = x,
	    Err (e) => panic! ("Could not access GPIO memory! {}",e),
    }
    
    gpio
}

fn get_pin_state ( argument: Option<OsString>  ) -> Level {
	let mut pin_state = Level::Low;
	match argument {
		Some (state) => {
			if state == OsString::from(HIGH_PIN_STATE_STR) { 
				pin_state = Level::High;
			}
			else if state != OsString::from(LOW_PIN_STATE_STR) {
				panic! ("State {:?} is invalid! \n{}", state, HELP_SUGGEST_STR );
			}
		},
		None => panic! ("State not found! \n{}",  HELP_SUGGEST_STR ),
	}

    pin_state
}

trait SetState{
	fn set_state (self, pin_state: Level ,gpio: &mut GPIO);
}

impl SetState for OsString {
	fn set_state (self, pin_state: Level, gpio: &mut GPIO) {
		match self.to_u8() {
			Ok (pin) => set_pin ( gpio, pin, pin_state ),
			Err (e) => {
				panic! ("{} \n{}", e,  HELP_SUGGEST_STR );		
			},
		}		
	}
}

fn set_all_pins ( gpio: &mut GPIO, pin_state: Level ) {
	for pin in 0..(LAST_PIN_NUMBER+1) {
		set_pin ( gpio, pin, pin_state );
	}
}

fn set_pin ( gpio: &mut GPIO, pin: u8, pin_state: Level ) {
	println! ("Set pin {} to {:?}", pin, pin_state);
	gpio.set_mode( pin, Mode::Output );
	gpio.write ( pin, pin_state );
}

fn set_pins (mut cmd: env::ArgsOs) {
	let mut gpio = initiate_gpio_handle();
	gpio.set_clear_on_drop ( false );

	let pin_state = get_pin_state( cmd.nth(0) );

	match cmd.nth(0) {
		Some (pin_as_os_string) => {
			if pin_as_os_string == OsString::from(ALL_PINS_STR) {
				set_all_pins ( &mut gpio, pin_state );
				return;
			}
			else {
				pin_as_os_string.set_state( pin_state, &mut gpio );
			}
			
		}
		None => panic! ("Pin not found! \n{}",  HELP_SUGGEST_STR ),		
	}

	for argument in cmd {
		argument.set_state( pin_state, &mut gpio );
	}	
}

// For library "rppal" version "0.1.2" the following LICENSE (MIT) applyes:
// "
//   Copyright (c) 2017 Rene van der Meer
// 
//   Permission is hereby granted, free of charge, to any person obtaining a
//   copy of this software and associated documentation files (the "Software"),
//   to deal in the Software without restriction, including without limitation
//   the rights to use, copy, modify, merge, publish, distribute, sublicense,
//   and/or sell copies of the Software, and to permit persons to whom the
//   Software is furnished to do so, subject to the following conditions:
// 
//   The above copyright notice and this permission notice shall be included in
//   all copies or substantial portions of the Software.
// 
//   THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//   IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//   FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
//   THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//   LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
//   FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
//   DEALINGS IN THE SOFTWARE.
// "

// For the "std" rust library the following LINCESE (MIT) applyes:
// "
//   Copyright (c) 2010 The Rust Project Developers
// 
//   Permission is hereby granted, free of charge, to any
//   person obtaining a copy of this software and associated
//   documentation files (the "Software"), to deal in the
//   Software without restriction, including without
//   limitation the rights to use, copy, modify, merge,
//   publish, distribute, sublicense, and/or sell copies of
//   the Software, and to permit persons to whom the Software
//   is furnished to do so, subject to the following
//   conditions:
// 
//   The above copyright notice and this permission notice
//   shall be included in all copies or substantial portions
//   of the Software.
// 
//   THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
//   ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
//   TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
//   PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
//   SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
//   CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
//   OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
//   IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
//   DEALINGS IN THE SOFTWARE.
// "
