target := "arm-unknown-linux-gnueabihf"
ip := "192.168.1.105"
output_binary := "$CARGO_TARGET_DIR/" + target + "/release/warning-light"

compile:
	cross build --release --target {{target}} --features pi

transport:
	# expects a directory /home/pi/warning-light to exist
	rsync {{output_binary}} pi@{{ip}}:/home/pi/warning-light/run
	rsync ./warning-light.service pi@{{ip}}:/home/pi/warning-light/warning-light.service
