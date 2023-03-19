# warning-light

Cad models & PCB schematics for a raspberry pi zero W hooked up to a rotating alarm light
and enabled from messages on matrix.

## Specifying matrix credientials

create a file `./.config.json` with the following parameters (example is provided for matrix.org,
you can use any homeserver):

{
	"homeserver_url": "https://matrix.org",
	"username": "@your-username:matrix.org",
	"password": "your password here"
}

these parameters will be included in the binary and used at runtime.

## Compiling

Compilation scripts are located in the `justfile`, you will need [just](https://github.com/casey/just) 
to run them. Otherwise, you can likely figure out how to run them without just by simply reading the file.

to cross compile for rasperry pi zero W:

```
just comple
```

then, edit the `ip` variable of the justfile to the IP address of the pi, and then

```
just transport
```

## systemd interaction

to automatically start the program, you can use systemd services:

```
cd /home/pi/warning-light 
sudo cp warning-light.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable warning-light
sudo systemctl start warning-light
```

## using a different user than `pi`

scripts generally assume the username on the raspberry pi is `pi`. If you have a different user, you
will need to edit `justfile`, as well as `warning-light.service`
